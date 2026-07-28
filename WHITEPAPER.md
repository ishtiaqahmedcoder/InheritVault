# InheritVault Whitepaper

### On-chain, non-custodial inheritance for the Stellar network

**Version 1.0**, Draft for the Stellar Community Fund and public review

**Repository:** https://github.com/ishtiaqahmedcoder/InheritVault
**Live contract (Stellar testnet):** `CAWHXHUN2UG5C7VQNIO5UAIPIINVBQKGHN5YZN62B3ZF4OKTARLO7FPZ`
**License:** MIT

> InheritVault is a non-custodial "dead-man's switch" for crypto. A user locks
> funds in their own vault, checks in from time to time, and if they ever go
> silent past a chosen deadline, the funds pass automatically to the beneficiaries
> they named. No lawyer, no custodian, no company holding the keys.

---

## Table of contents

1. [Abstract](#1-abstract)
2. [The problem](#2-the-problem)
3. [Market opportunity](#3-market-opportunity)
4. [The InheritVault solution](#4-the-inheritvault-solution)
5. [Protocol design and lifecycle](#5-protocol-design-and-lifecycle)
6. [System architecture](#6-system-architecture)
7. [Smart contracts](#7-smart-contracts)
8. [Safety mechanisms](#8-safety-mechanisms)
9. [Security model and trust assumptions](#9-security-model-and-trust-assumptions)
10. [Why Stellar and Soroban](#10-why-stellar-and-soroban)
11. [Heir onboarding](#11-heir-onboarding)
12. [Business model and sustainability](#12-business-model-and-sustainability)
13. [Roadmap](#13-roadmap)
14. [Path to decentralization](#14-path-to-decentralization)
15. [Risks and mitigations](#15-risks-and-mitigations)
16. [Team](#16-team)
17. [Conclusion](#17-conclusion)
18. [Disclaimer](#18-disclaimer)

---

## 1. Abstract

Self-custody gives people full control of their money, but it has one fatal edge
case: death and disappearance. A cryptocurrency wallet is controlled entirely by
whoever holds the private key. There is no bank, no help desk, no password reset,
and no court order that can recover a lost key. When the only person who knows the
key dies or vanishes without a plan, the funds are frozen on-chain forever, visible
but unreachable.

InheritVault solves this with a simple and auditable primitive. Each user deploys
their own vault contract and deposits USDC or XLM. The contract tracks a single
value: the time since the owner's last check-in. While the owner is active they
keep full control. If they ever go silent past a configured interval and grace
period, the pre-named beneficiaries receive their exact shares directly, in one
atomic transaction.

The contract never decides that anyone has died. It only measures silence. That
simplicity is what makes it trustworthy, cheap to audit, and safe to hold real
value. The payout is permissionless, so the funds reach the heirs even if the
InheritVault team disappears entirely. The core vault contract is implemented in
Rust for Soroban, is covered by a passing test suite, and is already deployed and
exercised end to end on the Stellar testnet.

---

## 2. The problem

A private key is the sole point of control for a self-custodial wallet. This is a
feature while the owner is alive and well, and a catastrophe when they are not.
Every existing way to plan for that catastrophe is inadequate:

| Option | Why it fails |
|---|---|
| Write the seed phrase down | It can be found and stolen at any time |
| Tell no one | The funds die with the owner |
| Give the keys to a relative today | You hand them full control now, not later |
| A traditional will | It cannot move an on-chain wallet, and probate can take many months |
| Custodial "beneficiary" features | They reintroduce the custodian that self-custody exists to avoid |

The result is a large and growing pool of permanently lost value, and families who
can see a balance on a block explorer but have no way to ever reach it. There is no
safe, trustless way for a self-custodial user to pass funds to the people they
love. That gap is what InheritVault fills.

---

## 3. Market opportunity

The problem is real, large, and growing with adoption.

- More than 20 percent of all Bitcoin is estimated to be lost forever, a large part
  of it stranded because the key holder is gone (widely cited Chainalysis-era
  estimates).
- Roughly 60 percent of crypto owners have never shared their keys with family, and
  a large share store them in unsecured locations.
- Between 14 and 17 percent of U.S. adults have owned crypto, yet only around a
  quarter of Americans have a will of any kind.
- Estate attorneys report tens of millions of dollars lost to heirs because no one
  knew the private keys.

In recent years the first mainstream tools appeared (digital estate services,
inactive-account managers, legacy-contact features from large tech platforms),
which validates demand. None of them is a non-custodial, on-chain, automatic
solution for self-custodied crypto. That is the wedge.

The strongest initial audience is small to mid-size holders who cannot justify a
lawyer or an institutional custodian but still want their crypto to reach their
family. The long-term opportunity is business to business: licensing the vault
module to wallets, exchanges, and banks that want to offer inheritance to their own
users.

---

## 4. The InheritVault solution

InheritVault has two layers.

**On-chain (the trust boundary).** A vault contract per owner holds the funds and
enforces every rule. A factory contract deploys and indexes vaults so owners and
heirs can discover them. No backend can move funds; the contract is the only
authority over the money.

**Off-chain (the usability layer).** A platform provides profiles, multi-channel
reminders, a keeper bot that triggers claims, and guided onboarding for heirs who
are not crypto native. None of these can move funds; they only make the primitive
usable.

Core properties:

- **Non-custodial.** Keys never leave the owner's wallet.
- **Owner-only while active.** Only the owner can withdraw or change terms until the
  deadline passes.
- **Permissionless claim.** After the deadline, anyone can trigger the payout, but
  funds only ever go to the pre-configured beneficiaries.
- **Exact shares.** Splits are fixed in the contract in basis points that sum to
  100 percent, so whoever triggers the payout cannot change who gets what.
- **Fully isolated.** One contract instance per owner; funds are never pooled.
- **The dead owner's key is never needed.** The funds live in the contract, not in
  the owner's personal wallet.

---

## 5. Protocol design and lifecycle

The protocol models a single question: has the owner been silent for too long? Its
lifecycle has four phases.

**Setup.** The owner connects a wallet, deploys a vault through the factory, and
locks USDC or XLM. They name beneficiaries with exact shares, choose a check-in
interval (for example every 6 months), and set a grace period during which daily
reminders are sent.

**Active.** Any activity from the owner resets the countdown. This includes an
explicit check-in, a withdrawal, a deposit adjustment, or a change to the
beneficiaries or schedule. The owner keeps full control and can exit at any time by
withdrawing everything.

**Grace.** As the deadline approaches, the platform sends escalating reminders by
email and phone. A single check-in during this window returns the vault to the
Active phase. This window exists so that one missed reminder never causes a payout.

**Claim.** If the owner stays silent past interval plus grace, the vault becomes
claimable. The keeper bot triggers the payout, or an heir, or anyone, since the
call is permissionless. The contract verifies on-chain that the deadline has passed
and that the vault has not already been claimed, then transfers each beneficiary
their share directly, marks the vault as claimed, and can never run again.

A key nuance: blockchains have no built-in timer, so a transaction must trigger the
payout. The keeper bot automates that trigger so the process feels automatic. Since
the shares are already locked in the contract, whoever sends the trigger cannot
change the outcome.

---

## 6. System architecture

```
                          Owner and heirs (browser, wallet)
                                       |
                          Platform (web app and API)
                          profiles, reminders, onboarding
                                       |
             +-------------------------+-------------------------+
             |                         |                         |
       Vault Factory              Keeper Bot                Reminder service
       (Soroban)                  (Node, TypeScript)        email and phone
             |                         |
        deploys                   triggers claim()
             |                         |
       Inheritance Vault (one per owner)
       funds, shares, timer, claim
             |                         |
        Heir wallet A             Heir wallet B
```

On-chain components (the factory and the vault) form the trust boundary. Off-chain
components (the platform, the keeper bot, the reminder service) make the system
usable but hold no authority over funds. If every off-chain component disappeared,
an heir could still claim directly from the contract.

---

## 7. Smart contracts

### 7.1 Inheritance Vault

One instance per owner. Written in Rust for Soroban. Holds the funds and enforces
all rules. Public interface:

| Function | Caller | Purpose |
|---|---|---|
| `init(owner, token, interval, grace, beneficiaries)` | owner | one-time setup |
| `deposit(from, amount)` | anyone | fund the vault |
| `check_in()` | owner | proof of life; resets the countdown |
| `withdraw(to, amount)` | owner | take funds out; also proves life |
| `set_beneficiaries(list)` | owner | update heirs; shares must total 10,000 bps |
| `set_schedule(interval, grace)` | owner | change the cadence |
| `claim()` | anyone | after the deadline, distribute by share |
| `status`, `deadline`, `is_claimable`, `time_left`, `beneficiaries`, `owner`, `token`, `last_check_in` | anyone | read-only views |

Invariants enforced by the contract:

- Beneficiary shares are expressed in basis points and must sum to exactly 10,000.
- A minimum interval floor prevents a zero interval that would lock the owner out
  instantly; sensible real-world minimums are a policy of the application layer.
- The rounding remainder of a distribution goes to the last beneficiary, so the
  vault always fully empties.
- A `claimed` flag prevents any second payout.
- Only the owner can move funds while the vault is active; `claim` only succeeds
  after interval plus grace has elapsed.
- Instance storage time-to-live is extended on writes so the vault survives long
  periods of silence.

The contract is covered by a suite of unit tests that exercise initialization and
validation, deposit, check-in, withdrawal, claim before the deadline (which must
fail), claim after the deadline (which distributes correctly), double claim (which
must fail), and rounding behavior. The suite runs in continuous integration on
every push, and the contract compiles to a small WebAssembly binary suitable for
Soroban.

### 7.2 Vault Factory

Deploys and initializes a vault in a single transaction and keeps a discovery
registry so all parties can find their vaults.

| Function | Purpose |
|---|---|
| `init(admin, vault_wasm)` | one-time; stores the vault WASM hash |
| `create_vault(owner, token, interval, grace, beneficiaries, salt)` | deploy, init, register, and return the new vault address |
| `vaults_of_owner(owner)` | all vaults an owner created |
| `vaults_for_beneficiary(who)` | all vaults someone is named in; powers the heir portal |
| `all_vaults()` | every vault created; used by the keeper bot |
| `set_vault_wasm(hash)` | admin only; point at a new vault version |

---

## 8. Safety mechanisms

A wrong payout is catastrophic, so safety is layered rather than trusted to a
single control.

1. **Grace period.** A configurable buffer after the interval, so a single missed
   check-in never triggers a transfer.
2. **Escalating multi-channel reminders.** Email and phone today, with SMS, push,
   and messaging apps planned, growing more frequent as the deadline approaches.
3. **Delegated and multi-channel check-in.** The owner can prove life from an email
   link or message reply, not only from the wallet, so travel or illness does not
   cause a lockout.
4. **Always reversible while active.** The owner can withdraw, top up, change heirs,
   or reset the schedule at any time.
5. **Cancel window.** A short freeze between the trigger and the actual transfer, in
   which a living owner can still cancel.
6. **Guardians (optional).** A threshold of trusted people must confirm before a
   claim executes, and any one of them can veto a false trigger. A timeout fallback
   ensures that if guardians never respond, the vault falls back to silence-only so
   funds can never be permanently locked.

The design guarantee is simple to state: a living owner can always stop a payout.

---

## 9. Security model and trust assumptions

**What the contract guarantees.** Only the owner can move funds while active. Funds
can only ever be distributed to the pre-configured beneficiaries in their exact
shares. A claim can only succeed after the deadline, and only once.

**Permissionlessness as a trust feature.** The keeper bot is a convenience, not a
custodian. Because `claim` is permissionless, the system does not depend on the
InheritVault team, the bot, or any single server. If all of that disappeared, an
heir could still trigger the same payout directly.

**Threats considered.**

- *False trigger while alive.* Mitigated by grace, reminders, delegated check-in,
  the cancel window, and optional guardians.
- *Griefing by a third party triggering claim early.* Impossible; `claim` reverts
  before the deadline, and even after, funds only reach the configured heirs.
- *Contract bug moving funds incorrectly.* Mitigated by a small, readable contract,
  a full test suite, and an independent professional security audit before mainnet.
- *Storage expiry during long silence.* Mitigated by extending storage time-to-live
  on writes and by Soroban state restoration.
- *Guardian collusion or loss.* Mitigated by a threshold below the total, the
  owner's ability to replace guardians while active, and a timeout fallback.

**Trust assumptions.** Users must run a wallet and set a realistic schedule.
Beneficiaries need a Stellar wallet with a trustline to receive the asset; the
passkey onboarding module removes this barrier for non-crypto heirs.

---

## 10. Why Stellar and Soroban

InheritVault is a natural fit for Stellar for four reasons.

- **Cost.** A long-lived vault involves many small operations over years: check-ins,
  reminders, and eventually a distribution. Stellar's sub-cent fees make this
  practical where higher-fee chains would not.
- **Speed.** Roughly five-second finality means a payout settles almost instantly
  once triggered.
- **Assets.** Native support for USDC, EURC, and XLM lets people protect stable
  value, not only volatile assets.
- **A genuinely new primitive.** A review of the Stellar ecosystem and of prior
  Stellar Community Fund awards found no inheritance or dead-man's-switch vault. The
  "vaults" that exist are DeFi yield vaults, an entirely different use case.
  InheritVault is a new public-good primitive for the network.

Soroban's Rust smart contracts, deterministic execution, and state model let the
core logic stay small and auditable while the platform grows around it.

---

## 11. Heir onboarding

A beneficiary must be able to receive funds. In the simplest case they already have
a Stellar wallet with a trustline for the asset. For heirs who are not crypto
native, InheritVault plans a passkey smart-wallet onboarding flow that uses device
biometrics (Face ID or a fingerprint) and requires no seed phrase, building on
Stellar's support for secp256r1 signatures. The heir portal, powered by the
factory's `vaults_for_beneficiary` registry, lets an heir discover the vaults they
are named in and claim with a single guided action.

---

## 12. Business model and sustainability

InheritVault launches as a grant-funded public good and is designed to become
self-sustaining.

| Stream | How it works |
|---|---|
| Annual subscription (primary) | Owners pay roughly 24 to 48 USD per year to keep the vault active, with reminders and bot monitoring |
| Setup fee | A small optional one-time fee at creation |
| Protocol fee on payout | A tiny on-chain cut of roughly half a percent at distribution |
| Premium features | Guardians, staggered inheritance, multi-asset, higher limits |
| Business to business and white-label (primary) | Licensing the vault module to wallets, exchanges, and banks |

Simple unit economics: 1,000 active vaults at 36 USD per year is about 36,000 USD
of recurring revenue, enough to cover the keeper bot, reminder infrastructure, and
one developer, independent of further grants. During the grant period the service
is free or low cost to drive adoption; monetization begins after mainnet.

---

## 13. Roadmap

The core primitive is already built and self-funded, which proves it works. The
grant funds the platform, the safety layer, and the independent audit.

**Tranche 1, Trust core.** Vault and factory hardening, guardians and a cancel
window, multi-token support, and a live testnet dashboard connected to a real
wallet.

**Tranche 2, Platform.** Backend with email and phone reminders, the keeper bot,
and the passkey heir claim portal. Target of at least 25 pilot vaults.

**Tranche 3, Mainnet.** A professional security audit, fixes, mainnet launch, and
pilot users. Target of at least 10 funded mainnet vaults in the first month.

Current status: the inheritance vault contract is implemented, unit-tested,
deployed to testnet, and verified end to end on-chain, and both a landing page and
an interactive dashboard demo are live.

---

## 14. Path to decentralization

InheritVault is designed so that trust in any operator decreases over time.

- The trust boundary is already fully on-chain and permissionless at the point of
  payout.
- The factory's admin key controls only which vault code version is offered to new
  users; it cannot touch existing vaults or funds. Over time this can move to a
  multisig or community governance.
- The keeper bot is replaceable by anyone, and the reference implementation will be
  open source so third parties can run their own.
- All contracts are open source under the MIT license, so the primitive survives the
  company.

---

## 15. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Owner locks themselves out with too short an interval | Enforced interval floor, grace period, reminders |
| Accidental trigger from one missed check-in | Configurable grace buffer, cancel window, guardians |
| Owner alive but silent (travel, hospital, jail) | Guardian veto, multi-channel and delegated check-in, cancel window |
| Contract bug moving funds incorrectly | Full test suite now, professional audit before mainnet |
| Keeper bot down when a vault becomes claimable | Claim is permissionless; an heir or anyone can trigger it |
| Heir is not crypto native | Passkey smart-wallet onboarding with no seed phrase |
| Perception that it is only a timelock | The platform scope: factory, reminders, bot, guardians, onboarding |
| Trust barrier of locking large funds in new code | Non-custodial design, open source, audit, and starting with smaller vaults |

---

## 16. Team

*To be completed.* This section should name who builds and maintains the
funds-handling Soroban contract and the platform, and their relevant background.
The strongest answer to reviewer questions about smart-contract capability is the
working, tested, deployed contract in the repository, combined with a commitment to
an independent professional audit before any mainnet deployment.

---

## 17. Conclusion

Self-custody should not mean that a person's crypto dies with them. InheritVault
turns a hard human problem into a small, auditable on-chain primitive: a vault that
measures silence and, only after a clear deadline and a grace period, passes exactly
what the owner chose to exactly the people they named. It is non-custodial, it is
permissionless at the point that matters, and it is already running on Stellar
testnet. With the platform, safety layer, and audit that this project sets out to
build, InheritVault can become a durable public good that lets ordinary families
inherit crypto safely, and a new primitive that brings a genuinely human use case to
the Stellar network.

---

## 18. Disclaimer

InheritVault is open-source, non-custodial software. It is not a legal will, a bank,
or an estate-planning or financial service. On-chain inheritance complements, and
does not replace, proper legal documents. Nothing in this document is legal or
financial advice. Statistics cited are drawn from widely reported public estimates
and are used to illustrate the scale of the problem, not as precise figures. Any
forward-looking statements about features, revenue, or timelines are goals, not
guarantees.
