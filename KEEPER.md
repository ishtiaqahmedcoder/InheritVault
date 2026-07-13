# InheritVault, Complete Project, Build & Grant Document

> A non-custodial **"dead-man's switch" for crypto** on Stellar / Soroban.
> Lock your funds, check in periodically, and if you ever go silent your assets
> pass automatically to the people you chose, no lawyer, no company holding your
> keys.

**Network:** Stellar / Soroban, **Contracts:** Rust, **App + bot:** JS/TS
, **Target:** Stellar Community Fund (SCF)
, **Status:** Core vault contract **built, 14/14 tests passing, compiles to WASM** ✅

This single document contains everything:
- **Part A, The Project** (what it is, features, how it works, diagrams, contracts)
- **Part B, Build It Yourself** (step-by-step dev roadmap, commands, tests)
- **Part C, Win the Grant** (SCF strategy, objections, proposal structure)

---

## Contents

**Part A, The Project**
1. [What Keeper is](#1-what-keeper-is)
2. [The problem](#2-the-problem)
3. [The solution](#3-the-solution)
4. [How it works, end to end](#4-how-it-works--end-to-end)
5. [Diagram 1, System architecture](#5-diagram-1--system-architecture)
6. [Diagram 2, Vault lifecycle](#6-diagram-2--vault-lifecycle)
7. [Diagram 3, The claim sequence](#7-diagram-3--the-claim-sequence)
8. [Smart contracts](#8-smart-contracts)
9. [Safety mechanisms](#9-safety-mechanisms)
10. [Tech stack](#10-tech-stack)
11. [Business model](#11-business-model)
12. [Repository layout](#12-repository-layout)

**Part B, Build It Yourself**
13. [Build order overview](#13-build-order-overview)
14. [Phase 0, Setup](#14-phase-0--setup)
15. [Phase 1, Vault contract](#15-phase-1--vault-contract)
16. [Phase 2, Safety features](#16-phase-2--safety-features-)
17. [Phase 3, Factory](#17-phase-3--factory)
18. [Phase 4, Deploy to testnet](#18-phase-4--deploy-to-testnet)
19. [Phase 5, Live frontend demo](#19-phase-5--live-frontend-demo-)
20. [Phase 6, Reminders + keeper bot](#20-phase-6--reminders--keeper-bot)
21. [Phase 7, Traction + polish](#21-phase-7--traction--polish)
22. [Timeline](#22-timeline)

**Part C, Win the Grant**
23. [What SCF judges score](#23-what-scf-judges-score)
24. [Positioning, the winning story](#24-positioning--the-winning-story)
25. [Kill the 4 objections](#25-kill-the-4-objections)
26. [The proposal, exact section order](#26-the-proposal--exact-section-order)
27. [Budget & milestones](#27-budget--milestones)
28. [Pre-submission checklist](#28-pre-submission-checklist)
29. [Risk register](#29-risk-register)

---
---

# PART A, THE PROJECT

## 1. What Keeper is

Each user deploys **their own vault contract** and deposits USDC/XLM. The contract
tracks exactly one thing: **time since the owner's last check-in.**

- While the owner is active → **full control** (deposit, withdraw, change heirs).
- If the owner goes silent past `interval + grace` → the named beneficiaries can
  **claim** their shares.
- The contract **never decides that someone died**, it only measures silence.
  That simplicity is what makes it trustworthy and auditable.

**Core features**
- **Non-custodial**, keys never leave the owner's wallet.
- **Owner-only while active**, permissionless claim only after the deadline.
- **Multiple beneficiaries** with exact percentage splits (basis points).
- **Configurable grace period**, one missed check-in never triggers a payout.
- **Guardians (M-of-N veto)**, optional human confirmation before payout.
- **Cancel window**, a freeze in which a living owner can abort a false trigger.
- **One contract per owner**, funds fully isolated.
- **The dead owner's key is never needed**, funds live in the contract.

---

## 2. The problem

Self-custody has a fatal edge case: **death and disappearance.** A wallet is
controlled entirely by whoever holds the private key. No bank, no reset, no court
order can recover it. When the only person who knows the key is gone, the funds
are frozen on-chain **forever**, visible, but unreachable.

Every existing option is bad:

| Option | Why it fails |
|---|---|
| Write the seed phrase down | Can be found and stolen |
| Tell no one | The funds die with you |
| Give keys to a relative "just in case" | You hand them everything, today |
| A traditional will | Can't move an on-chain wallet; months of probate |

**How big:** 20%+ of all Bitcoin is estimated lost forever; ~60% of holders never
shared keys with family; ~40% store them in unsecured places. The problem is real,
large, and growing with adoption. **That gap is what Keeper fills.**

---

## 3. The solution

A **vault contract per owner** + a **platform** that makes it usable.

- **On-chain** (the trust boundary, no backend can move funds):
  the vault contract + a factory that deploys and indexes vaults.
- **Off-chain** (makes it usable): profiles, reminders, a keeper bot that
  triggers claims, and heir onboarding.

The key line: *the shares are already locked in the contract, so whoever triggers
the payout cannot change who gets what.*

---

## 4. How it works, end to end

**Setup**
1. Owner creates a profile (email + phone).
2. Connects a wallet, deploys a vault via the factory, locks USDC/XLM (e.g. 10,000 USDC).
3. Names beneficiaries with exact shares (Wallet A 50%, Wallet B 50%).
4. Sets a check-in interval (e.g. 90 days) + grace period (e.g. 7 days).

**While active**
5. One check-in resets the countdown.
6. Before each deadline, the platform sends email + phone reminders.
7. Owner can withdraw, top up, or change heirs any time.

**If the owner goes silent**
8. No check-in → the countdown keeps running.
9. Once `interval + grace` elapses → the vault becomes **claimable**
   (optionally after guardians confirm + a cancel window).

**Payout**
10. The keeper bot triggers `claim()` (or an heir, or anyone, it's permissionless).
11. The contract verifies on-chain: *deadline passed? not already claimed?*
12. It transfers each heir their share **directly**, in one atomic transaction.
13. It sets `claimed = true` so it can never run twice.

> **Key nuance:** blockchains have no built-in timer. A transaction must trigger
> `claim()`. The bot automates that trigger so it *feels* automatic, but because
> the shares are pre-locked, the trigger-er cannot change the outcome.

---

## 5. Diagram 1, System architecture

Who talks to whom. On-chain = trust boundary; off-chain = usability layer.

```
                          ┌──────────────────────────┐
       Owner  ──────────▶ │   PLATFORM (web + API)   │ ◀──────────  Heirs
   (browser + wallet)     │   profiles, reminders   │
                          └────────────┬─────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              ▼                        ▼                         ▼
       ┌──────────────┐        ┌──────────────┐          ┌──────────────┐
       │ VAULT FACTORY│        │  KEEPER BOT  │          │  REMINDERS   │
       │  (Soroban)   │        │  (Node/TS)   │          │ email + SMS  │
       └──────┬───────┘        └──────┬───────┘          └──────────────┘
              │ deploys               │ triggers claim()
              ▼                       ▼
       ┌─────────────────────────────────────────────┐
       │      INHERITANCE VAULT  (one per owner)      │
       │      funds, shares, timer, claim          │
       └──────────────┬───────────────────┬───────────┘
                      ▼                   ▼
                Heir wallet A        Heir wallet B

   [ ON-CHAIN  = Factory + Vault ]     [ OFF-CHAIN = Platform + Bot + Reminders ]
```

---

## 6. Diagram 2, Vault lifecycle

The heart of the product. Note the return paths on the right, a living owner can
**always** escape a false trigger.

```
          ┌──────────────────────────────────┐
          │  1, SETUP                        │   actor: Owner
          │  deploy vault, lock funds        │
          │  name heirs, set schedule        │
          └────────────────┬─────────────────┘
                           │  fund + configure
                           ▼
     ┌──────────────────────────────────────┐
     │  2, ACTIVE                           │ ◀────────────┐
     │  owner in full control                │              │  check_in()
     │  deposit / withdraw / change heirs    │ ─────────────┘  resets timer
     └────────────────┬─────────────────────┘
                      │  no check-in past (interval + grace)
                      ▼
     ┌──────────────────────────────────────┐
     │  3, SILENCE                          │ ─── owner checks in ──────┐
     │  deadline passed, countdown finished  │                           │
     └────────────────┬─────────────────────┘                           │
                      │  silence continues                               │
                      ▼                                                  │
     ┌──────────────────────────────────────┐                           │
     │  4, PENDING                          │ ─── guardian veto ────────┤
     │  guardians confirm, cancel window    │                           │
     └────────────────┬─────────────────────┘             back to ACTIVE │
                      │  M-of-N guardians approve                        │
                      ▼                                                  │
     ┌──────────────────────────────────────┐  ◀────────────────────────┘
     │  5, CLAIMABLE                        │
     │  claim() distributes by share         │
     └────────────────┬─────────────────────┘
                      │  atomic transfer
                      ▼
     ┌──────────────────────────────────────┐
     │  6, HEIRS PAID                       │   actor: Heirs
     │  USDC arrives in heir wallets         │
     └──────────────────────────────────────┘
```

**Reading it:** the only path to *Heirs Paid* is prolonged silence **plus**
(optional) guardian approval. From *Silence* or *Pending*, a single owner check-in
or any guardian veto snaps it back to *Active*. That is the "we won't pay a living
owner's heirs" guarantee.

---

## 7. Diagram 3, The claim sequence

What actually happens on payout, one atomic transaction.

```
  Keeper Bot          Vault Contract           USDC Token          Heir A / B
      │                     │                       │                   │
      │──── claim() ───────▶│                       │                   │
      │                     │ check: deadline passed?                   │
      │                     │        not already claimed?               │
      │                     │ (reverts if either fails)                 │
      │                     │──── transfer 50% ────▶│───▶ Heir A  5,000  │
      │                     │──── transfer 50% ────▶│───▶ Heir B  5,000  │
      │                     │ set claimed = true    │                   │
      │◀──── success ───────│   (all-or-nothing)    │                   │
```

If the bot is offline, any heir, or anyone, can send the same `claim()`. Funds
only ever go to the pre-configured beneficiaries.

---

## 8. Smart contracts

### 8.1 Inheritance Vault
One instance per owner. Holds funds, enforces all rules.

| Function | Who | Purpose |
|---|---|---|
| `init(owner, token, interval, grace, beneficiaries)` | owner | one-time setup |
| `deposit(from, amount)` | anyone | fund the vault |
| `check_in()` | owner | proof-of-life; resets countdown |
| `withdraw(to, amount)` | owner | take funds out (also proves life) |
| `set_beneficiaries(list)` | owner | update heirs (shares total 10 000 bps) |
| `set_schedule(interval, grace)` | owner | change cadence |
| `set_guardians(list, threshold)` | owner | configure the veto layer |
| `guardian_approve()` / `guardian_veto()` | guardian | confirm / cancel a trigger |
| `claim()` | anyone | after deadline, distribute by share |
| `status()` / `deadline()` / `is_claimable()` / `beneficiaries()` | anyone | read views |

**Rules:** shares sum to exactly `10_000` bps, minimum interval **1 day**,
rounding remainder → last beneficiary (vault fully empties), `claimed` flag blocks
double payout, permissionless `claim`, funds only ever go to configured heirs.

### 8.2 Vault Factory
Deploys + initializes a vault in one transaction, and keeps a discovery registry.

| Function | Purpose |
|---|---|
| `init(admin, vault_wasm)` | one-time; store the vault WASM hash |
| `create_vault(owner, token, interval, grace, beneficiaries, salt)` | deploy + init + register, return address |
| `vaults_of_owner(owner)` | powers the owner dashboard |
| `vaults_for_beneficiary(who)` | powers the heir portal |
| `all_vaults()` | full list (bot uses this) |
| `set_vault_wasm(hash)` | admin-only; upgrade the vault version |

---

## 9. Safety mechanisms

Because a wrong payout is catastrophic, safety is layered:

1. **Grace period**, a buffer *after* the interval; one missed check-in never triggers.
2. **Cancel window**, a freeze between trigger and transfer where the owner can abort.
3. **Guardians (M-of-N veto)**, trusted people confirm; any one can veto a false
   trigger. Timeout fallback prevents permanent lock if guardians vanish.
4. **Escalating multi-channel reminders**, email + phone (later SMS/push).
5. **Delegated / multi-channel check-in**, stay "alive" from an email link.
6. **Always reversible while active**, withdraw, top up, change heirs, reset schedule.

**False-trigger scenarios handled:** holiday, hospital/coma, jail, remote area,
guardians veto + long grace + cancel window + multi-channel check-in cover these.

---

## 10. Tech stack

| Layer | Choice | Notes |
|---|---|---|
| Smart contracts | **Rust / Soroban** | vault + factory |
| Chain | **Stellar** | ~5s finality, sub-cent fees |
| Assets | **USDC**, EURC, XLM | any Stellar token |
| Keeper bot | **Node / TS** | `@stellar/stellar-sdk`; holds XLM for gas |
| Heir onboarding | **Passkey smart wallets** | Face ID / fingerprint, no seed phrase |
| Backend / API | **Laravel (PHP)** or all-Node | profiles, reminders, scheduler, mail |
| Frontend | **HTML/JS → React** | owner dashboard + heir portal |
| Wallet | **Freighter** | testnet + mainnet signing |

> Stellar + passkey tooling is TypeScript-native. Recommended shape: **Laravel as
> the app backend + a thin Node/TS service** for all chain interactions. A
> single-language alternative is all-Node/TS.

---

## 11. Business model

Grant-funded public good at launch, designed to become self-sustaining:

| Stream | How |
|---|---|
| **Annual subscription** ⭐ | ~$24-$48/yr to keep the vault active + reminders + bot |
| **Setup fee** | small one-time fee at creation |
| **Protocol fee on payout** | tiny on-chain cut (~0.5%) at distribution |
| **Premium features** | guardians, staggered inheritance, multi-asset |
| **B2B / white-label** ⭐ | license the vault module to wallets, exchanges, banks |

**Simple unit economics:** 1,000 active vaults × $36/yr = **~$36K/yr recurring**,
enough to cover infra + one developer, independent of further grants.

---

## 12. Repository layout

```
keeper/
├── contract/            # Inheritance Vault (Rust / Soroban)
│   ├── src/lib.rs
│   ├── src/test.rs
│   └── Cargo.toml
├── factory/             # Vault Factory + registry (Rust / Soroban)
│   ├── src/lib.rs
│   └── Cargo.toml
├── web/                 # dashboard + landing (HTML/JS → React)
│   ├── index.html
│   └── app.html
├── keeper-bot/          # Node/TS auto-claim bot
├── backend/             # reminders + profiles (Laravel or Node)
├── KEEPER.md            # this document (everything)
└── README.md
```

---
---

# PART B, BUILD IT YOURSELF

You are building the whole thing yourself. This is the exact order, the tools/
commands, and the "done" test for each step. Goal: a **live testnet demo + safety
features + clean repo** that wins SCF.

## 13. Build order overview

```
Phase 0  Setup toolchain            ── 1 hour
Phase 1  Vault contract + tests     ── 1-2 weeks
Phase 2  Guardians + cancel window  ── 1 week      ⭐ don't skip
Phase 3  Factory + registry         ── 3-5 days
Phase 4  Deploy to testnet          ── half day
Phase 5  Live frontend demo         ── 1-2 weeks   ⭐ your strongest asset
Phase 6  Reminders + keeper bot     ── 1-2 weeks
Phase 7  Traction + polish          ── parallel
```

**Critical path to a winning submission:** 1 → 2 → 3 → 4 → 5 + 7.

---

## 14. Phase 0, Setup

Install the toolchain (do once, ~1 hour):
- [ ] **Rust** ≥ 1.85, https://rustup.rs
- [ ] `rustup target add wasm32-unknown-unknown`
- [ ] **Stellar CLI**, `cargo install --locked stellar-cli`
- [ ] **Node.js** ≥ 20 + npm/pnpm
- [ ] **Freighter** wallet (testnet mode), https://freighter.app
- [ ] `git init` → first commit → push to a **public** GitHub repo (public = trust)

**Done test:** `stellar --version` and `cargo --version` both work.

---

## 15. Phase 1, Vault contract

The trust boundary. Build and fully test before touching the frontend.

**Scaffold**
- [ ] `cd contract && stellar contract init.`
- [ ] Add `soroban-sdk` to `Cargo.toml`

**Storage / state**
- [ ] `owner: Address`, `token: Address`
- [ ] `interval: u64`, `grace: u64` (seconds)
- [ ] `last_checkin: u64` (ledger timestamp)
- [ ] `beneficiaries: Vec<(Address, u32)>` (bps, sum = 10_000)
- [ ] `claimed: bool`

**Functions**, build + unit-test each (see §8.1 table).

**Rules to enforce**
- [ ] Shares sum to exactly 10_000 bps
- [ ] Minimum interval 1 day (anti-lockout)
- [ ] `claimed` flag blocks double payout
- [ ] Only owner moves funds while active; `claim` only after `interval + grace`

**Tests (≥ 8 in `src/test.rs`)**
- [ ] init validation, deposit, check-in resets timer, withdraw
- [ ] claim-before-deadline fails, claim-after-deadline distributes correctly
- [ ] double-claim fails, rounding empties the vault

**Done test:** `cargo test` → all pass.

---

## 16. Phase 2, Safety features ⭐

Judges fear "paying a living owner's heirs." Solve it **in the contract now**, not
in a V2 roadmap. This is what separates you from a hackathon toy.

**Cancel window**
- [ ] Add a `PENDING` state: `claim()`'s first call only *arms* the payout (`pending_since`).
- [ ] Actual transfer only after a `cancel_window` (e.g. 3-7 days).
- [ ] `owner` calling `check_in()` during PENDING **aborts** the payout.

**Basic guardians (M-of-N veto)**
- [ ] Storage: `guardians: Vec<Address>`, `threshold: u32`, `approvals`
- [ ] `set_guardians(list, threshold)`, owner only, while active
- [ ] `guardian_approve()`, a guardian confirms the owner is gone
- [ ] `guardian_veto()`, any guardian cancels a false trigger → back to ACTIVE
- [ ] `claim()`: if guardians set, require `approvals ≥ threshold` **and** deadline passed
- [ ] **Timeout fallback**, if guardians never respond in X days, bypass to silence-only

State machine to implement: see [Diagram 2](#6-diagram-2--vault-lifecycle).

- [ ] Tests: veto returns to ACTIVE, threshold enforced, timeout fallback, owner abort during PENDING

**Done test:** a test proves a living owner can stop a payout, and a guardian can veto.

---

## 17. Phase 3, Factory

One-click "deploy my own vault" + a discovery registry.

- [ ] `cd factory && stellar contract init.`
- [ ] Implement functions from §8.2
- [ ] Tests for create + both registry lookups

**Done test:** a single `create_vault` call deploys a working, initialized vault.

---

## 18. Phase 4, Deploy to testnet

```bash
# build both
cd contract && stellar contract build
cd../factory && stellar contract build

# fund a testnet identity
stellar keys generate --global alice --network testnet --fund

# upload vault wasm, get hash
stellar contract upload --wasm <vault.wasm> --source alice --network testnet

# deploy factory, init with the vault wasm hash
stellar contract deploy --wasm <factory.wasm> --source alice --network testnet
stellar contract invoke --id <FACTORY_ID> --source alice --network testnet \
  -- init --admin <G...> --vault_wasm <HASH>
```

- [ ] Record `FACTORY_ID` + vault WASM hash (the frontend needs them).

**Done test:** full `create_vault → deposit → check_in → claim` works from the CLI on testnet.

---

## 19. Phase 5, Live frontend demo ⭐

A judge clicking through beats everything. Move the dashboard from "demo mode" to a
**real testnet wallet connection.**

- [ ] In `web/`, add `@stellar/stellar-sdk` + `@stellar/freighter-api`
- [ ] Connect Freighter, read the user's public key
- [ ] "Create vault" form → factory `create_vault`, sign with Freighter, submit
- [ ] Implement each action: create/fund, check-in (with countdown), manage
      heirs + schedule, set guardians, trigger/veto
- [ ] Heir view: list from `vaults_for_beneficiary`, show claim button
- [ ] **Demo happy-path (< 2 min)** using a short interval (e.g. 60s):
      `create → fund → check-in → simulate silence → claim → heirs receive`

**Done test:** a fresh judge with Freighter can complete the full cycle unaided.

---

## 20. Phase 6, Reminders + keeper bot

Off-chain glue. Can start simple.

**Keeper bot (`keeper-bot/`, Node/TS)**
- [ ] Poll factory `all_vaults()` on a timer
- [ ] Read `is_claimable()` / `deadline()` per vault
- [ ] When claimable, submit `claim()` (bot key holds XLM for gas)
- [ ] Handle "already claimed" gracefully; log everything

**Reminder service (`backend/`)**
- [ ] Store owner contact + vault + deadline
- [ ] Cron: send escalating **email** reminders before each deadline
- [ ] **Check-in-by-email-link** so owners don't need the wallet to stay alive

**Done test:** bot auto-claims an expired testnet vault; a reminder email fires before a deadline.

---

## 21. Phase 7, Traction + polish

Proof, not just code (run in parallel):
- [ ] Clean **README** + architecture diagram
- [ ] Tests visible in **GitHub Actions** (`cargo test`)
- [ ] **2-min demo video** with voiceover
- [ ] **Waitlist page** (even a Google Form) → 10-20 signups
- [ ] 3-5 **user quotes** ("I need this")
- [ ] Post in **Stellar Dev Discord**, gather feedback, be visible

---

## 22. Timeline (solo dev)

| Weeks | Focus |
|---|---|
| 1-2 | Phase 1, vault contract + tests |
| 3 | Phase 2, guardians + cancel window ⭐ |
| 3-4 | Phase 3, factory + Phase 4 testnet deploy |
| 4-6 | Phase 5, live frontend demo ⭐ |
| 6-8 | Phase 6, bot + reminders |
| throughout | Phase 7, README, video, waitlist, Discord |

**Later / post-award:** passkey heir onboarding, SMS/WhatsApp channels,
professional audit → mainnet, multi-asset, B2B white-label.

---
---

# PART C, WIN THE GRANT

**Reality check:** SCF is decided by community voting + an expert panel. No plan
guarantees 100%. This part removes every common reason a project gets rejected and
stacks every factor judges reward.

> **Do first:** verify the **current SCF round** (deadline, award tier, application
> fields, voting mechanism, eligibility) on the official SCF site / Dev Discord.
> Rules change every round.

## 23. What SCF judges score

| Criterion | What they ask | Our state | Target to win |
|---|---|---|---|
| **Necessity** | Does Stellar need this? | Strong, real gap | Keep + add user proof |
| **Novelty** | Is it new on Stellar? | Strong, no vault like this | Keep + defend "not a timelock" |
| **Feasibility** | Can this team ship it? | MVP plan | Live testnet demo judges can click |
| **Team** | Who is building it? | ❌ Missing | Add credible team section |
| **Ecosystem impact** | Who benefits, how much? | Good framing | Add numbers + public-good angle |
| **Business viability** | Survives post-grant? | Decent model | Tighten unit economics |

**Winning insight:** you score high on idea + novelty already. Win by fixing the 3
weak points: **team, traction, and the false-trigger fear.**

---

## 24. Positioning, the winning story

Judges skim; they remember a story. Ours:

> **"Keeper is the first non-custodial way for a normal person to pass their crypto
> to their family if they die, automatically, with no lawyer and no company
> holding the keys. It's a public good that protects families, and a new primitive
> that only Stellar's speed and low fees make practical."**

Three pillars every section must reinforce:
1. **Human**, protects grieving families, not traders.
2. **New**, a primitive that doesn't exist on Stellar.
3. **Trustless**, we can disappear and the funds still reach the heirs.

---

## 25. Kill the 4 objections

Pre-answer these **inside the proposal** so they never become a reason to vote no.

**1. "This is just a timelock."**
→ Dedicated section: *"Why this is a platform, not a timelock."* Show the full
system, factory, bot, reminders, guardians, passkey onboarding, discovery portals.
The contract is 10% of the work.

**2. "You'll pay out while the owner is still alive."**
→ Solve it in the MVP: long grace + **cancel window** + **basic guardians (veto)**
in Phase 2, not V2. Say plainly: *"A living owner can always stop a payout."*

**3. "It's centralized, your bot triggers the payout."**
→ Lead with permissionlessness: *"The bot is convenience, not custody. `claim()`
is permissionless, if we vanish, any heir triggers it."*

**4. "Who are you and can you ship this?"**
→ Team section + a live testnet demo the judge runs themselves. Proof beats promises.

---

## 25b. The Soroban capability question (real SCF feedback)

> A real applicant received this feedback, and we will get the same question:
>
> *"Team capability on Soroban is the other open risk. The team's strength is
> full-stack web and backend work. Native smart-contract engineering is the gap.
> For funds-handling contracts, reviewers want to know exactly who writes and
> reviews the Soroban code."*

This is the single most likely reason a strong web team gets rejected. Reviewers
are nervous about **who writes and reviews the funds-handling Rust.** Answer it
head-on, with evidence, not reassurance:

1. **Show the working contract as proof.** We already have a Soroban contract that
   **compiles to WASM and passes 14/14 unit tests** in CI. A green test badge on a
   funds-handling contract *is* the capability answer. Link it in the proposal.
2. **Name who writes the Soroban code.** State it plainly: the contract author, and
   that the same person owns contract maintenance. Don't be vague.
3. **Name who reviews it, and budget for it.** The honest gap is independent
   review. Close it by putting a **professional Soroban / security audit in the
   T3 budget** (a named line item), and, if possible, an advisor or a second
   Soroban dev for peer review. Reviewers want to hear "an independent auditor
   signs off before mainnet," not "we've got it."
4. **Turn the web/backend strength into an asset.** Full-stack + backend is exactly
   what the *platform* (reminders, bot, heir onboarding, dashboards) needs. Frame
   it as: "chain logic is deliberately small, audited, and permissionless; the
   large surface area is the web/backend platform, our home turf."
5. **Keep the contract deliberately minimal.** A small, readable, well-tested
   contract is easier to audit and *demonstrates judgment*. Our vault is ~300 lines
   and does one thing. That is a feature in this conversation.

**One-line answer for the proposal:**
> *"The funds-handling contract is small, open-source, and already passing a full
> test suite in CI; it is written and maintained by \<name\>, and an independent
> professional Soroban audit (budgeted in T3) reviews it before any mainnet
> deployment."*

---

## 25c. What the grant money is actually for

A trap to avoid: if the submission looks *finished*, reviewers ask *"why do you
need funding?"* Our build is scoped so there is always a clear, fundable gap:

**Already built (unfunded, proves capability, not what we're asking money for):**
- The core InheritVault contract + full test suite + CI.

**What the grant funds (the real, remaining work):**
- **T1**, vault factory + registry, guardians + cancel window, multi-token, and a
  live testnet dashboard on a real wallet.
- **T2**, backend + email/phone reminders, the keeper bot, and the passkey heir
  claim portal (the non-crypto onboarding).
- **T3**, a **professional security audit** (the capability-gap closer), fixes,
  mainnet launch, and pilot users.

Framing line:
> *"We self-funded the core primitive to prove it works. The grant funds the
> platform, the safety layer, and, critically, the independent audit that makes
> it safe to hold real inheritances on mainnet."*

This simultaneously answers "why fund you" **and** the Soroban-capability question:
the biggest single ask is the audit.

---

## 26. The proposal, exact section order

Write your SCF submission in this order (judges read top-down and stop early):

1. **One-paragraph pitch**, the story from §24. Hook first.
2. **The problem**, with the stats, kept tight.
3. **The solution**, vault + platform, 4 bullets max.
4. **Live demo**, link + video **above the fold**.
5. **Why it's novel on Stellar**, no inheritance vault exists; public good.
6. **Why it's not just a timelock**, the platform (Objection 1).
7. **Safety & false-trigger handling**, guardians, grace, cancel (Objection 2).
8. **Trustless by design**, permissionless claim (Objection 3).
9. **Team**, who, past work, why you (Objection 4).
10. **Traction**, waitlist, quotes, demo usage.
11. **Milestones & budget**, verifiable deliverables per tranche.
12. **Business model**, how it survives post-grant.
13. **Ecosystem impact**, users protected, new primitive, open source.
14. **Risks & mitigations**, the honest table (judges trust honesty).

Keep it **scannable**: short paragraphs, tables, bold takeaways. Assume 4 minutes.

---

## 27. Budget & milestones

**Requested: ~42,000 USD in XLM** (not the max, reflects ~970 hrs at ~$43/hr,
in line with comparable awards). Each tranche must deliver a **judge-verifiable**
outcome, with safety moved forward:

| Tranche | Deliverable (verifiable) | Amount |
|---|---|---|
| **T1, Trust core** | Contract + factory + **guardians + cancel window** + live testnet dashboard with real wallet | $13,000 |
| **T2, Platform** | Reminders (email→SMS) + keeper bot + heir passkey claim portal + ≥25 pilot vaults | $17,000 |
| **T3, Mainnet** | Professional audit + fixes + mainnet + ≥10 funded vaults | $12,000 |

Every milestone needs a **binary success test** (e.g. "a non-crypto heir claimed
via passkey on testnet", yes/no). Vague milestones get flagged.

---

## 28. Pre-submission checklist

**Product**
- [ ] Contract + factory deployed to testnet
- [ ] Dashboard connects a real wallet (not demo mode)
- [ ] Guardians + cancel window implemented
- [ ] End-to-end flow works and is recorded

**Proof**
- [ ] Live demo link + 2-min video
- [ ] Waitlist / 10+ interested users
- [ ] 3+ user quotes
- [ ] Public, clean repo + README + visible tests

**Proposal**
- [ ] Follows §26 order, all 4 objections pre-answered
- [ ] Team section present, milestones have binary tests
- [ ] Budget fits current SCF tier, proofread, scannable

**Community**
- [ ] Posted in Discord, gathering feedback
- [ ] Demo shared publicly, ready to respond fast to questions

---

## 29. Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Solo/unknown team | Medium | Strong demo + traction; add advisors if possible |
| "Just a timelock" perception | Medium | §25 Objection 1 section + platform demo |
| False-trigger fear unresolved | High if unaddressed | Ship guardians + cancel in MVP (Phase 2) |
| Competitive round | Unknown | Public-good + novelty framing + loud community presence |
| "Is this a will?" concern | Low | Clear disclaimer; software, not legal service |
| Keeper bot down at claim time | Low | Claim is permissionless, heir/anyone can trigger |

---

## The two things that win

1. **Phase 2 (guardians + cancel window)**, kills the "you'll pay a living owner's
   heirs" objection.
2. **Phase 5 (live testnet demo)**, the single highest-ROI asset for votes.

Everything else supports these two.

---

## Disclaimer

Keeper is open-source, non-custodial software, **not** a legal will, a bank, or an
estate-planning or financial service. On-chain inheritance complements, and does
not replace, proper legal documents. Nothing here is legal or financial advice.
