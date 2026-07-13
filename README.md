# InheritVault

**A non-custodial on-chain inheritance vault ("dead-man's switch") for Stellar / Soroban.**

Lock your funds, check in periodically, and if you ever go silent your assets pass
automatically to the people you chose — no lawyer, no company holding your keys.

[![CI](https://github.com/ishtiaqahmedcoder/InheritVault/actions/workflows/ci.yml/badge.svg)](https://github.com/ishtiaqahmedcoder/InheritVault/actions/workflows/ci.yml)
· **Network:** Stellar / Soroban · **License:** MIT

> Full project, build and grant documentation: **[KEEPER.md](KEEPER.md)**

---

## What it does

Each user deploys **their own vault contract** and deposits USDC/XLM. The contract
tracks one thing: **time since the owner's last check-in.**

- While the owner is active → **full control** (deposit, withdraw, change heirs, adjust schedule).
- If the owner goes silent past `interval + grace` → the named beneficiaries can **claim** their shares.
- The contract never decides that anyone died — it only measures silence. That
  simplicity is what makes it auditable.

Funds only ever go to the pre-configured beneficiaries, so `claim` can safely be
**permissionless** — a keeper bot, an heir, or anyone can trigger it.

## Status

| Component | State |
|---|---|
| **Inherit Vault contract** | ✅ Built · **14/14 tests passing** · compiles to WASM |
| Vault factory (multi-vault registry) | ⬜ Grant scope |
| Guardians + cancel window | ⬜ Grant scope |
| Owner dashboard (live wallet) | ⬜ Grant scope |
| Keeper bot + reminders + heir portal | ⬜ Grant scope |
| Professional audit + mainnet | ⬜ Grant scope |

See [KEEPER.md](KEEPER.md) for the full roadmap and grant plan.

## Contract API

| Function | Who | Purpose |
|---|---|---|
| `init(owner, token, interval, grace, beneficiaries)` | owner | one-time setup |
| `deposit(from, amount)` | anyone | fund the vault |
| `check_in()` | owner | proof-of-life; resets the countdown |
| `withdraw(to, amount)` | owner | take funds out (also proves life) |
| `set_beneficiaries(list)` | owner | update heirs (shares total 10 000 bps) |
| `set_schedule(interval, grace)` | owner | change cadence |
| `claim()` | anyone | after the deadline, distribute by share |
| `status` / `deadline` / `is_claimable` / `time_left` / `beneficiaries` / `owner` / `token` / `last_check_in` | anyone | read-only views |

**Rules:** shares sum to exactly `10_000` bps · minimum interval **1 day** ·
rounding remainder → last beneficiary (vault fully empties) · `claimed` flag blocks
double payout.

## Build & test

Requires Rust ≥ 1.85 and the [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli).

```bash
# run the test suite (native, fast)
cargo test

# build the deployable contract
rustup target add wasm32v1-none      # once
stellar contract build               # -> target/wasm32v1-none/release/inherit_vault.wasm
```

## Deploy to testnet

```bash
stellar keys generate --global me --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/inherit_vault.wasm \
  --source me --network testnet            # -> CONTRACT_ID

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
├── contracts/
│   └── inherit-vault/        # the vault contract (Rust / Soroban)
│       ├── src/lib.rs
│       ├── src/test.rs       # 14 unit tests
│       └── Cargo.toml
├── web/                      # dashboard + landing (grant scope)
├── docs/                     # extra docs / assets
├── KEEPER.md                 # full project + build + grant document
├── Cargo.toml                # workspace
└── README.md
```

## Disclaimer

InheritVault is open-source, non-custodial software — **not** a legal will, a bank,
or an estate-planning or financial service. On-chain inheritance complements, and
does not replace, proper legal documents. Nothing here is legal or financial advice.
