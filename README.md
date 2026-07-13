# InheritVault

**A non-custodial on-chain inheritance vault (a "dead-man's switch") for Stellar and Soroban.**

Lock your funds, check in periodically, and if you ever go silent your assets pass
automatically to the people you chose, with no lawyer and no company holding your keys.

[![CI](https://github.com/ishtiaqahmedcoder/InheritVault/actions/workflows/ci.yml/badge.svg)](https://github.com/ishtiaqahmedcoder/InheritVault/actions/workflows/ci.yml)

**Network:** Stellar / Soroban. **License:** MIT.

Full project, build and grant documentation is in [KEEPER.md](KEEPER.md).

## What it does

Each user deploys their own vault contract and deposits USDC or XLM. The contract
tracks one thing: the time since the owner's last check-in.

- While the owner is active they keep full control (deposit, withdraw, change heirs, adjust the schedule).
- If the owner goes silent past `interval + grace`, the named beneficiaries can claim their shares.
- The contract never decides that anyone died. It only measures silence. That simplicity is what makes it auditable.

Funds only ever go to the pre-configured beneficiaries, so `claim` can safely be
permissionless: a keeper bot, an heir, or anyone can trigger it.

## Status

The core vault contract is implemented and covered by an automated test suite that
runs in CI on every push (see the badge above).

| Component | State |
|---|---|
| Inherit Vault contract | Implemented, unit-tested, compiles to WASM |
| Vault factory (multi-vault registry) | Planned (grant scope) |
| Guardians and cancel window | Planned (grant scope) |
| Owner dashboard (live wallet) | Planned (grant scope) |
| Keeper bot, reminders, heir portal | Planned (grant scope) |
| Professional audit and mainnet | Planned (grant scope) |

See [KEEPER.md](KEEPER.md) for the full roadmap and grant plan.

## Contract API

| Function | Who | Purpose |
|---|---|---|
| `init(owner, token, interval, grace, beneficiaries)` | owner | one-time setup |
| `deposit(from, amount)` | anyone | fund the vault |
| `check_in()` | owner | proof of life; resets the countdown |
| `withdraw(to, amount)` | owner | take funds out (also proves life) |
| `set_beneficiaries(list)` | owner | update heirs (shares total 10,000 bps) |
| `set_schedule(interval, grace)` | owner | change the cadence |
| `claim()` | anyone | after the deadline, distribute by share |
| `status`, `deadline`, `is_claimable`, `time_left`, `beneficiaries`, `owner`, `token`, `last_check_in` | anyone | read-only views |

Rules enforced by the contract: shares sum to exactly 10,000 bps; a minimum
interval floor guards against a zero interval (the app recommends long,
real-world intervals); the rounding remainder goes to the last beneficiary so the
vault fully empties; a `claimed` flag prevents any second payout.

## Build and test

Requires Rust 1.85 or newer and the [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli).

```bash
# run the test suite (native, fast)
cargo test

# build the deployable contract
rustup target add wasm32v1-none
stellar contract build
# output: target/wasm32v1-none/release/inherit_vault.wasm
```

## Deploy to testnet

```bash
stellar keys generate --global me --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/inherit_vault.wasm \
  --source me --network testnet
# returns CONTRACT_ID

stellar contract invoke --id <CONTRACT_ID> --source me --network testnet \
  -- init --owner <G...> --token <TOKEN_ID> \
  --interval 7776000 --grace 604800 \
  --beneficiaries '[{"address":"<G...>","share_bps":10000}]'

# interact
stellar contract invoke --id <ID> --source me --network testnet -- check_in
stellar contract invoke --id <ID> --source me --network testnet -- status
stellar contract invoke --id <ID> --source anyone --network testnet -- claim
```

## Repository layout

```
InheritVault/
  contracts/
    inherit-vault/        the vault contract (Rust / Soroban)
      src/lib.rs
      src/test.rs         14 unit tests
      Cargo.toml
  web/                    dashboard and landing (grant scope)
  docs/                   extra docs and assets
  KEEPER.md               full project, build, and grant document
  Cargo.toml              workspace
  README.md
```

## Disclaimer

InheritVault is open-source, non-custodial software. It is not a legal will, a bank,
or an estate-planning or financial service. On-chain inheritance complements, and
does not replace, proper legal documents. Nothing here is legal or financial advice.
