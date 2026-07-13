#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{vec, Address, Env};

// ---------------------------------------------------------------------------
// Test harness
//
// Every test uses a SINGLE `Env`. Soroban objects (Addresses, Vecs) are tied to
// the Env that created them, so all values a test passes into the contract must
// come from `v.env`.
// ---------------------------------------------------------------------------

const DAY: u64 = 86_400;

struct Vault<'a> {
    env: Env,
    client: InheritVaultClient<'a>,
    token: token::TokenClient<'a>,
    token_admin: token::StellarAssetClient<'a>,
    token_addr: Address,
    owner: Address,
    vault_id: Address,
}

fn create_token<'a>(env: &Env, admin: &Address) -> (Address, token::StellarAssetClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let addr = sac.address();
    (addr.clone(), token::StellarAssetClient::new(env, &addr))
}

/// Fresh env with an owner, a mintable test token, and a registered (but not yet
/// initialized) vault. The clock starts at a non-zero timestamp.
fn new_env<'a>() -> Vault<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);

    let owner = Address::generate(&env);
    let (token_addr, token_admin) = create_token(&env, &owner);
    let token = token::TokenClient::new(&env, &token_addr);

    let vault_id = env.register(InheritVault, ());
    let client = InheritVaultClient::new(&env, &vault_id);

    Vault {
        env,
        client,
        token,
        token_admin,
        token_addr,
        owner,
        vault_id,
    }
}

fn bene(addr: &Address, bps: u32) -> Beneficiary {
    Beneficiary {
        address: addr.clone(),
        share_bps: bps,
    }
}

/// Advance the ledger clock by `secs` seconds.
fn advance(env: &Env, secs: u64) {
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + secs);
}

// ---------------------------------------------------------------------------
// 1. Init + views
// ---------------------------------------------------------------------------

#[test]
fn init_sets_state_and_starts_active() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &(7 * DAY),
        &vec![&v.env, bene(&a, 10_000)],
    );

    assert_eq!(v.client.owner(), v.owner);
    assert_eq!(v.client.status(), VaultStatus::Active);
    assert_eq!(v.client.is_claimable(), false);
    assert_eq!(v.client.deadline(), 1_000_000 + 90 * DAY + 7 * DAY);
}

// ---------------------------------------------------------------------------
// 2. Init validation
// ---------------------------------------------------------------------------

#[test]
fn init_rejects_short_interval() {
    let v = new_env();
    let a = Address::generate(&v.env);
    // 1 hour < 1 day minimum.
    let res = v.client.try_init(
        &v.owner,
        &v.token_addr,
        &3_600,
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );
    assert_eq!(res, Err(Ok(Error::IntervalTooShort)));
}

#[test]
fn init_rejects_bad_share_sum() {
    let v = new_env();
    let a = Address::generate(&v.env);
    let b = Address::generate(&v.env);
    // 5000 + 4000 = 9000 != 10000
    let res = v.client.try_init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &0,
        &vec![&v.env, bene(&a, 5_000), bene(&b, 4_000)],
    );
    assert_eq!(res, Err(Ok(Error::InvalidShares)));
}

#[test]
fn cannot_init_twice() {
    let v = new_env();
    let a = Address::generate(&v.env);
    let benes = vec![&v.env, bene(&a, 10_000)];
    v.client
        .init(&v.owner, &v.token_addr, &(90 * DAY), &0, &benes);

    let res = v
        .client
        .try_init(&v.owner, &v.token_addr, &(90 * DAY), &0, &benes);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

// ---------------------------------------------------------------------------
// 3. Deposit
// ---------------------------------------------------------------------------

#[test]
fn deposit_moves_funds_into_vault() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );

    v.token_admin.mint(&v.owner, &10_000);
    v.client.deposit(&v.owner, &10_000);

    assert_eq!(v.token.balance(&v.vault_id), 10_000);
    assert_eq!(v.token.balance(&v.owner), 0);
}

// ---------------------------------------------------------------------------
// 4. Check-in resets the countdown
// ---------------------------------------------------------------------------

#[test]
fn check_in_resets_deadline() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &(7 * DAY),
        &vec![&v.env, bene(&a, 10_000)],
    );

    advance(&v.env, 80 * DAY);
    v.client.check_in();

    let now = v.env.ledger().timestamp();
    assert_eq!(v.client.deadline(), now + 90 * DAY + 7 * DAY);
    assert_eq!(v.client.status(), VaultStatus::Active);
}

// ---------------------------------------------------------------------------
// 5. Claim before deadline fails
// ---------------------------------------------------------------------------

#[test]
fn claim_before_deadline_fails() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &(7 * DAY),
        &vec![&v.env, bene(&a, 10_000)],
    );
    v.token_admin.mint(&v.owner, &10_000);
    v.client.deposit(&v.owner, &10_000);

    let res = v.client.try_claim();
    assert_eq!(res, Err(Ok(Error::NotYetClaimable)));
}

// ---------------------------------------------------------------------------
// 6. Claim after deadline distributes by share
// ---------------------------------------------------------------------------

#[test]
fn claim_after_deadline_distributes_by_share() {
    let v = new_env();
    let a = Address::generate(&v.env);
    let b = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &(7 * DAY),
        &vec![&v.env, bene(&a, 6_000), bene(&b, 4_000)],
    );
    v.token_admin.mint(&v.owner, &10_000);
    v.client.deposit(&v.owner, &10_000);

    // Go silent past the deadline.
    advance(&v.env, 90 * DAY + 7 * DAY + 1);
    assert_eq!(v.client.is_claimable(), true);

    // Permissionless: anyone triggers it.
    v.client.claim();

    assert_eq!(v.token.balance(&a), 6_000);
    assert_eq!(v.token.balance(&b), 4_000);
    assert_eq!(v.token.balance(&v.vault_id), 0);
    assert_eq!(v.client.status(), VaultStatus::Claimed);
}

// ---------------------------------------------------------------------------
// 7. Double claim fails
// ---------------------------------------------------------------------------

#[test]
fn double_claim_fails() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );
    v.token_admin.mint(&v.owner, &5_000);
    v.client.deposit(&v.owner, &5_000);

    advance(&v.env, 90 * DAY + 1);
    v.client.claim();

    let res = v.client.try_claim();
    assert_eq!(res, Err(Ok(Error::AlreadyClaimed)));
}

// ---------------------------------------------------------------------------
// 8. Rounding remainder goes to the last beneficiary (vault fully empties)
// ---------------------------------------------------------------------------

#[test]
fn rounding_remainder_empties_vault() {
    let v = new_env();
    let a = Address::generate(&v.env);
    let b = Address::generate(&v.env);
    let c = Address::generate(&v.env);
    // Three thirds of a number not divisible by 3.
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(30 * DAY),
        &0,
        &vec![&v.env, bene(&a, 3_333), bene(&b, 3_333), bene(&c, 3_334)],
    );
    v.token_admin.mint(&v.owner, &100);
    v.client.deposit(&v.owner, &100);

    advance(&v.env, 30 * DAY + 1);
    v.client.claim();

    // a: 100*3333/10000 = 33, b: 33, c: remainder = 34. Sum = 100.
    assert_eq!(v.token.balance(&a), 33);
    assert_eq!(v.token.balance(&b), 33);
    assert_eq!(v.token.balance(&c), 34);
    assert_eq!(v.token.balance(&v.vault_id), 0);
}

// ---------------------------------------------------------------------------
// 9. Withdraw returns funds and proves life
// ---------------------------------------------------------------------------

#[test]
fn withdraw_returns_funds_and_resets_timer() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(90 * DAY),
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );
    v.token_admin.mint(&v.owner, &10_000);
    v.client.deposit(&v.owner, &10_000);

    advance(&v.env, 50 * DAY);
    v.client.withdraw(&v.owner, &4_000);

    assert_eq!(v.token.balance(&v.owner), 4_000);
    assert_eq!(v.token.balance(&v.vault_id), 6_000);
    // Timer reset by the withdraw.
    let now = v.env.ledger().timestamp();
    assert_eq!(v.client.deadline(), now + 90 * DAY);
}

// ---------------------------------------------------------------------------
// 10. Owner can update beneficiaries; new split is honored on claim
// ---------------------------------------------------------------------------

#[test]
fn updated_beneficiaries_are_honored() {
    let v = new_env();
    let a = Address::generate(&v.env);
    let b = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(30 * DAY),
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );
    v.token_admin.mint(&v.owner, &1_000);
    v.client.deposit(&v.owner, &1_000);

    // Change heirs to a 50/50 split.
    v.client
        .set_beneficiaries(&vec![&v.env, bene(&a, 5_000), bene(&b, 5_000)]);

    advance(&v.env, 30 * DAY + 1);
    v.client.claim();

    assert_eq!(v.token.balance(&a), 500);
    assert_eq!(v.token.balance(&b), 500);
}

// ---------------------------------------------------------------------------
// 11. set_beneficiaries rejects an invalid split
// ---------------------------------------------------------------------------

#[test]
fn set_beneficiaries_rejects_bad_sum() {
    let v = new_env();
    let a = Address::generate(&v.env);
    let b = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(30 * DAY),
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );

    let res = v
        .client
        .try_set_beneficiaries(&vec![&v.env, bene(&a, 5_000), bene(&b, 6_000)]);
    assert_eq!(res, Err(Ok(Error::InvalidShares)));
}

// ---------------------------------------------------------------------------
// 12. Claim with an empty vault fails cleanly
// ---------------------------------------------------------------------------

#[test]
fn claim_empty_vault_fails() {
    let v = new_env();
    let a = Address::generate(&v.env);
    v.client.init(
        &v.owner,
        &v.token_addr,
        &(30 * DAY),
        &0,
        &vec![&v.env, bene(&a, 10_000)],
    );

    advance(&v.env, 30 * DAY + 1);
    let res = v.client.try_claim();
    assert_eq!(res, Err(Ok(Error::NothingToDistribute)));
}
