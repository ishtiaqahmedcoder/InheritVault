#![no_std]
//! # InheritVault
//!
//! A non-custodial on-chain **inheritance vault** ("dead-man's switch") for
//! Stellar / Soroban.
//!
//! One vault = one owner. The vault holds the owner's tokens and tracks a single
//! thing: **time since the owner's last check-in.** While the owner keeps checking
//! in they retain full control. If the owner ever goes silent past
//! `interval + grace`, the pre-configured beneficiaries can `claim` their shares,
//! and only their shares, because the split is locked in the contract, not decided
//! by whoever triggers the payout.
//!
//! The contract never decides that anyone died. It only measures silence. That
//! simplicity is what makes it auditable.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Shares are expressed in basis points; they must sum to exactly 100%.
const BPS_DENOM: u32 = 10_000;

/// Minimum check-in interval: 1 day (seconds). Prevents lock-out footguns.
const MIN_INTERVAL: u64 = 86_400;

/// Ledgers per day (~5s close time), used for storage TTL management.
const DAY_IN_LEDGERS: u32 = 17_280;
/// Extend instance storage by ~30 days on each write.
const BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
/// Re-bump when the remaining TTL drops below ~29 days.
const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single heir and their share of the vault, in basis points (1% = 100 bps).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Beneficiary {
    pub address: Address,
    pub share_bps: u32,
}

/// High-level lifecycle state, for the UI and the keeper bot.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultStatus {
    /// Owner is active (or the deadline has not yet passed).
    Active,
    /// Deadline passed, funds can be claimed.
    Claimable,
    /// Funds have been distributed. Terminal.
    Claimed,
}

/// Instance-storage keys.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    Owner,
    Token,
    Interval,
    Grace,
    LastCheckIn,
    Beneficiaries,
    Claimed,
}

/// Contract errors. Returned as `Result::Err` so callers get clean failures.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    IntervalTooShort = 3,
    /// Beneficiary shares do not sum to exactly 10_000 bps.
    InvalidShares = 4,
    NoBeneficiaries = 5,
    /// `claim` called before the deadline.
    NotYetClaimable = 6,
    /// `claim` called on an already-distributed vault.
    AlreadyClaimed = 7,
    InvalidAmount = 8,
    /// The vault holds no balance to distribute.
    NothingToDistribute = 9,
}

#[contract]
pub struct InheritVault;

#[contractimpl]
impl InheritVault {
    // -----------------------------------------------------------------------
    // Setup
    // -----------------------------------------------------------------------

    /// One-time initialization. Sets the owner, the token this vault holds, the
    /// check-in cadence, and the heirs. Starts the countdown from now.
    pub fn init(
        env: Env,
        owner: Address,
        token: Address,
        interval: u64,
        grace: u64,
        beneficiaries: Vec<Beneficiary>,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Owner) {
            return Err(Error::AlreadyInitialized);
        }
        if interval < MIN_INTERVAL {
            return Err(Error::IntervalTooShort);
        }
        validate_beneficiaries(&beneficiaries)?;

        // The owner authorizes creation of their own vault.
        owner.require_auth();

        let store = env.storage().instance();
        store.set(&DataKey::Owner, &owner);
        store.set(&DataKey::Token, &token);
        store.set(&DataKey::Interval, &interval);
        store.set(&DataKey::Grace, &grace);
        store.set(&DataKey::Beneficiaries, &beneficiaries);
        store.set(&DataKey::LastCheckIn, &env.ledger().timestamp());
        store.set(&DataKey::Claimed, &false);

        bump_ttl(&env);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Owner actions (all reset the countdown = proof of life)
    // -----------------------------------------------------------------------

    /// Proof-of-life. Resets the countdown. Owner-only.
    pub fn check_in(env: Env) -> Result<(), Error> {
        let owner = Self::require_owner(&env)?;
        owner.require_auth();
        Self::reset_timer(&env);
        Ok(())
    }

    /// Withdraw funds from the vault back to the owner (or anywhere). Owner-only.
    /// Also counts as proof-of-life.
    pub fn withdraw(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        let owner = Self::require_owner(&env)?;
        owner.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let token = get_token(&env);
        token::TokenClient::new(&env, &token).transfer(
            &env.current_contract_address(),
            &to,
            &amount,
        );
        Self::reset_timer(&env);
        Ok(())
    }

    /// Replace the beneficiary list. Shares must still sum to 10_000 bps.
    /// Owner-only. Also counts as proof-of-life.
    pub fn set_beneficiaries(env: Env, beneficiaries: Vec<Beneficiary>) -> Result<(), Error> {
        let owner = Self::require_owner(&env)?;
        owner.require_auth();
        validate_beneficiaries(&beneficiaries)?;
        env.storage()
            .instance()
            .set(&DataKey::Beneficiaries, &beneficiaries);
        Self::reset_timer(&env);
        Ok(())
    }

    /// Change the check-in cadence. Owner-only. Also counts as proof-of-life.
    pub fn set_schedule(env: Env, interval: u64, grace: u64) -> Result<(), Error> {
        let owner = Self::require_owner(&env)?;
        owner.require_auth();
        if interval < MIN_INTERVAL {
            return Err(Error::IntervalTooShort);
        }
        let store = env.storage().instance();
        store.set(&DataKey::Interval, &interval);
        store.set(&DataKey::Grace, &grace);
        Self::reset_timer(&env);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Funding (anyone) and payout (permissionless)
    // -----------------------------------------------------------------------

    /// Fund the vault. Anyone can deposit; the depositor authorizes the transfer.
    /// Depositing does NOT reset the countdown (only owner actions do).
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        require_initialized(&env)?;
        from.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let token = get_token(&env);
        token::TokenClient::new(&env, &token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );
        bump_ttl(&env);
        Ok(())
    }

    /// Distribute the vault's entire balance to the beneficiaries by share.
    /// **Permissionless**, a keeper bot, an heir, or anyone may call it, but it
    /// only succeeds after the deadline, and funds only ever go to the
    /// pre-configured beneficiaries. Runs exactly once.
    pub fn claim(env: Env) -> Result<(), Error> {
        require_initialized(&env)?;
        if is_claimed(&env) {
            return Err(Error::AlreadyClaimed);
        }
        if env.ledger().timestamp() < deadline_of(&env) {
            return Err(Error::NotYetClaimable);
        }

        let token = get_token(&env);
        let client = token::TokenClient::new(&env, &token);
        let contract = env.current_contract_address();
        let total: i128 = client.balance(&contract);
        if total <= 0 {
            return Err(Error::NothingToDistribute);
        }

        let benes = get_beneficiaries(&env);
        let n = benes.len();
        let mut distributed: i128 = 0;
        for i in 0..n {
            let b = benes.get(i).unwrap();
            // The last beneficiary absorbs the rounding remainder so the vault
            // always fully empties.
            let amount = if i == n - 1 {
                total - distributed
            } else {
                total * (b.share_bps as i128) / (BPS_DENOM as i128)
            };
            if amount > 0 {
                client.transfer(&contract, &b.address, &amount);
            }
            distributed += amount;
        }

        env.storage().instance().set(&DataKey::Claimed, &true);
        bump_ttl(&env);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read-only views
    // -----------------------------------------------------------------------

    /// Timestamp at which the vault becomes claimable.
    pub fn deadline(env: Env) -> u64 {
        deadline_of(&env)
    }

    /// True if funds can be claimed right now.
    pub fn is_claimable(env: Env) -> bool {
        !is_claimed(&env) && env.ledger().timestamp() >= deadline_of(&env)
    }

    /// High-level lifecycle status.
    pub fn status(env: Env) -> VaultStatus {
        if is_claimed(&env) {
            VaultStatus::Claimed
        } else if env.ledger().timestamp() >= deadline_of(&env) {
            VaultStatus::Claimable
        } else {
            VaultStatus::Active
        }
    }

    /// Seconds remaining until the deadline (0 if already reached).
    pub fn time_left(env: Env) -> u64 {
        let now = env.ledger().timestamp();
        let d = deadline_of(&env);
        if now >= d {
            0
        } else {
            d - now
        }
    }

    pub fn beneficiaries(env: Env) -> Vec<Beneficiary> {
        get_beneficiaries(&env)
    }

    pub fn owner(env: Env) -> Address {
        get_owner(&env)
    }

    pub fn token(env: Env) -> Address {
        get_token(&env)
    }

    pub fn last_check_in(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastCheckIn)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn require_owner(env: &Env) -> Result<Address, Error> {
        require_initialized(env)?;
        Ok(get_owner(env))
    }

    fn reset_timer(env: &Env) {
        env.storage()
            .instance()
            .set(&DataKey::LastCheckIn, &env.ledger().timestamp());
        bump_ttl(env);
    }
}

// ---------------------------------------------------------------------------
// Free helper functions
// ---------------------------------------------------------------------------

fn require_initialized(env: &Env) -> Result<(), Error> {
    if env.storage().instance().has(&DataKey::Owner) {
        Ok(())
    } else {
        Err(Error::NotInitialized)
    }
}

fn validate_beneficiaries(benes: &Vec<Beneficiary>) -> Result<(), Error> {
    if benes.is_empty() {
        return Err(Error::NoBeneficiaries);
    }
    let mut sum: u32 = 0;
    for i in 0..benes.len() {
        // saturating_add guards against overflow on hostile input.
        sum = sum.saturating_add(benes.get(i).unwrap().share_bps);
    }
    if sum != BPS_DENOM {
        return Err(Error::InvalidShares);
    }
    Ok(())
}

fn deadline_of(env: &Env) -> u64 {
    let store = env.storage().instance();
    let last: u64 = store.get(&DataKey::LastCheckIn).unwrap_or(0);
    let interval: u64 = store.get(&DataKey::Interval).unwrap_or(0);
    let grace: u64 = store.get(&DataKey::Grace).unwrap_or(0);
    last + interval + grace
}

fn is_claimed(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Claimed)
        .unwrap_or(false)
}

fn get_owner(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Owner).unwrap()
}

fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

fn get_beneficiaries(env: &Env) -> Vec<Beneficiary> {
    env.storage()
        .instance()
        .get(&DataKey::Beneficiaries)
        .unwrap()
}

/// Keep the vault's storage alive. A vault must survive long periods of silence,
/// so we extend the instance TTL on every write.
fn bump_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

mod test;
