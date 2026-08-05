# InheritVault

> **A non-custodial, on-chain inheritance vault (a "dead-man's switch") for Stellar and Soroban.**
> Lock your funds, check in periodically, and if you ever go silent, your assets pass automatically to the people you chose—with no lawyer, no custodians, and no companies holding your keys.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Soroban: 26](https://img.shields.io/badge/Soroban-v26-purple)](https://developers.stellar.org/docs/smart-contracts)
[![Rust: 1.85+](https://img.shields.io/badge/Rust-1.85%2B-orange)](https://www.rust-lang.org/)

---

## What it does

Each user deploys their own personal vault contract and deposits USDC or XLM. The contract tracks one thing: **the time since the owner's last check-in**.

* **Full Control:** While you are active, you keep full control over your funds (deposit, withdraw, change beneficiaries, or adjust the cadence).
* **Automated Payout:** If you go silent past your chosen `interval + grace` period, your beneficiaries can claim their shares.
* **Permissionless Claim:** The contract never decides if anyone has died; it only measures silence. Because the rules and shares are pre-configured in the immutable contract code, the `claim()` execution is completely permissionless—anyone, including a keeper bot or an heir, can trigger the payout.

---

## Features

* **Non-Custodial:** Keys never leave your wallet.
* **Safety Buffer (Grace Period):** A daily reminder grace period ensures that a single missed check-in won't trigger a payout.
* **Custom Splits:** Define multiple heirs with exact percentage splits (expressed on-chain in basis points, summing to exactly 100%).
* **Activity Tracking:** Deposits, withdrawals, and explicit check-ins all reset the timer.
* **Complete Isolation:** Each user has their own contract instance, meaning funds are never pooled or co-mingled.

---

## Tech Stack & Structure

* **Smart Contracts:** Written in Rust for the **Stellar / Soroban** smart contract platform.
* **Frontend:** Interactive mockup dashboard & landing page built with vanilla HTML/JS/CSS.
* **Keeper Bot (Planned):** Node/TS automation to monitor vaults and automatically trigger payouts.

### Repository Layout
```text
InheritVault/
├── contracts/
│   └── inherit-vault/     # The Soroban smart contract (Rust)
│       ├── src/lib.rs     # Core contract logic
│       ├── src/test.rs    # 14 unit tests covering lifecycle and edge cases
│       └── Cargo.toml
├── web/                   # Frontend dashboard and landing page mockup
│   ├── index.html         # Landing page
│   ├── app.html           # Interactive dashboard demo
│   ├── style.css          # Shared stylesheet
│   └── only-logo-t.png    # Assets
└── Cargo.toml             # Workspace configuration
```

---

## Running the Project Locally

### 1. Smart Contract (Rust / Soroban)

#### Prerequisites
* Install Rust ≥ 1.85 from [rustup.rs](https://rustup.rs).
* Add the WebAssembly compilation target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
* Install the **Stellar CLI**:
  ```bash
  cargo install --locked stellar-cli
  ```

#### Running Tests
Run the contract's unit test suite (14 passing tests):
```bash
cargo test
```

#### Building the Contract
To compile the contract to a deployable `.wasm` binary:
```bash
stellar contract build
```
The compiled binary will be generated at `target/wasm32-unknown-unknown/release/inherit_vault.wasm`.

---

### 2. Frontend Dashboard Demo

The [web/](web) directory contains a static client-side mockup of the vault configuration and claiming flow.

#### Running via Laragon (Local Domain)
If you are using Laragon, the project is mapped automatically to:
* **Landing Page:** [http://inheritvault.test/web/index.html](http://inheritvault.test/web/index.html)
* **Dashboard Demo:** [http://inheritvault.test/web/app.html](http://inheritvault.test/web/app.html)

#### Running via a Local HTTP Server
If your local `.test` domain is not resolving, start a quick local server using Python or Node.js in the project root:
* **Node.js (`npx`)**:
  ```bash
  npx -y serve -l 8000 web
  ```
  Then open [http://localhost:8000/index.html](http://localhost:8000/index.html).
* **Python**:
  ```bash
  python -m http.server 8000
  ```
  Then open [http://localhost:8000/web/index.html](http://localhost:8000/web/index.html).

---

## Smart Contract API

| Function | Caller | Purpose |
|---|---|---|
| `init(owner, token, interval, grace, beneficiaries)` | Owner | One-time vault initialization and configuration |
| `deposit(from, amount)` | Anyone | Add funds (USDC/XLM) to the vault |
| `check_in()` | Owner | Proof of life; resets the countdown timer |
| `withdraw(to, amount)` | Owner | Take funds out of the vault (also resets the timer) |
| `set_beneficiaries(list)` | Owner | Update heirs (shares must total exactly 10,000 bps / 100%) |
| `set_schedule(interval, grace)` | Owner | Change check-in interval and grace period |
| `claim()` | Anyone | After silence deadline passes, distribute funds to heirs |

---
