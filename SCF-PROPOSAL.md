# InheritVault — Stellar Community Fund proposal

> This is the submission draft. Fill the sections marked FILL IN (team, contact,
> traction numbers) with your real details before you submit. Everything else is
> ready. Keep it scannable: reviewers read top to bottom and stop early.

**Project:** InheritVault
**One line:** The first non-custodial way for a normal person to pass their crypto
to their family if they die, automatically, with no lawyer and no company holding
the keys.
**Category:** Public good / new primitive
**Requested:** 42,000 USD in XLM, across three tranches
**Repository:** https://github.com/ishtiaqahmedcoder/InheritVault
**Live contract (testnet):** `CAWHXHUN2UG5C7VQNIO5UAIPIINVBQKGHN5YZN62B3ZF4OKTARLO7FPZ`

---

## 1. The pitch

Self-custody has one fatal edge case: death and disappearance. A wallet is
controlled entirely by whoever holds the key. When that person is gone, the funds
are frozen on-chain forever, visible but unreachable. Over 20% of all Bitcoin is
estimated lost this way, and around 60% of holders have never shared access with
anyone.

InheritVault fixes this. Each user deploys their own vault, locks USDC or XLM, and
names their heirs. The vault tracks one thing: the time since the owner's last
check-in. While the owner keeps checking in they stay in full control. If they ever
go silent past a deadline, the funds pass automatically to the heirs they chose.

The contract never decides that anyone died. It only measures silence. That
simplicity is what makes it trustworthy and auditable. It is a public good that
protects ordinary families, and a new primitive that Stellar's speed and low fees
make practical.

## 2. The problem

Every existing option for passing on self-custodied crypto is bad:

- Writing the seed phrase down: it can be found and stolen.
- Telling no one: the funds die with you.
- Giving keys to a relative today: you hand them everything, now.
- A traditional will: it cannot move an on-chain wallet, and probate takes months.

There is no safe, trustless way for a self-custodial user to pass funds to their
family. That is the gap InheritVault fills.

## 3. The solution

A vault contract per owner, plus a platform that makes it usable.

- On-chain (the trust boundary, no backend can move funds): the vault contract and
  a factory that deploys and indexes vaults.
- Off-chain (makes it usable): profiles, reminders, a keeper bot that triggers
  claims, and heir onboarding.

The shares are locked in the contract, so whoever triggers the payout cannot change
who gets what. That is why the payout can safely be permissionless.

## 4. Proof: already live on testnet

The core is not a promise. It works today.

- Open-source contract in Rust, with 14 unit tests passing in CI on every push.
- Deployed to Stellar testnet and exercised end to end. The full lifecycle
  (create, fund, check in, go silent, claim) ran on-chain, and the heir received
  the funds automatically in one atomic transaction, leaving the vault empty and
  marked claimed.
- Live contract: `CAWHXHUN2UG5C7VQNIO5UAIPIINVBQKGHN5YZN62B3ZF4OKTARLO7FPZ`
  (viewable on Stellar Expert, testnet).
- Interactive dashboard demo showing the full owner flow.

FILL IN: link to a 2 to 3 minute demo video here. It is the single highest-impact
thing in this proposal.

## 5. Why it is novel on Stellar

No inheritance or dead-man's-switch vault exists on Stellar today. The "vaults" that
exist are DeFi yield vaults, an entirely different use case. InheritVault is a
genuinely new primitive for the network, and a public-good one: it protects ordinary
users and their families, not traders.

## 6. Why this is a platform, not just a timelock

The contract is deliberately small, because a small contract is easy to audit and
safe to hold real money. But the product is a platform, and that is where most of
the work is:

- A factory that deploys and indexes one vault per owner, with discovery for both
  owners and heirs.
- A keeper bot that triggers claims automatically.
- Escalating multi-channel reminders (email and phone) so a payout never triggers
  by accident.
- Guardians, an optional human confirmation layer that can veto a false trigger.
- Passkey onboarding so a non-crypto heir can claim with Face ID, no seed phrase.

The contract is roughly 10% of the work. The platform is the other 90%, and it is
exactly the full-stack web and backend work this team is strongest at.

## 7. Safety and the false-trigger problem

A wrong payout is catastrophic, so safety is layered:

- Grace period: a buffer after the interval, so one missed check-in never triggers.
- Cancel window: a freeze between trigger and transfer in which the owner can abort.
- Guardians: trusted people confirm the owner is gone, and any one of them can veto
  a false trigger. A timeout fallback prevents a permanent lock if guardians vanish.
- Escalating reminders and delegated check-in, so travel or illness does not lock
  you out.

A living owner can always stop a payout. That is a design guarantee, not a promise.

## 8. Trustless by design

The keeper bot is a convenience, not custody. The `claim` function is permissionless:
if the bot is offline, or if the team disappears entirely, any heir (or anyone) can
trigger the same payout. Funds only ever go to the pre-configured beneficiaries.

## 9. Team

FILL IN. Reviewers weigh the team heavily, and they will ask specifically who writes
and reviews the funds-handling contract. Answer it directly:

- Who is building this, and your background (full-stack web and backend experience).
- Who writes and maintains the Soroban contract (name the person). The working,
  tested, deployed contract in this repo is the evidence that this capability is
  real.
- Independent review: an external professional Soroban and security audit is
  budgeted in Tranche 3 and runs before any mainnet deployment. Reviewers want to
  hear that an independent auditor signs off before real inheritances are held.
- Why the web and backend strength is an asset: the chain logic is small and
  audited; the large surface (dashboards, reminders, bot, heir onboarding) is
  exactly this team's home turf.

## 10. Traction

FILL IN with real numbers before submitting:

- Waitlist signups (the landing page collects these).
- A few short quotes from people who say they need this.
- Any community feedback from the Stellar Dev Discord.

## 11. Milestones and budget

Requested: 42,000 USD in XLM. This reflects roughly 970 engineering hours for one
developer at about 43 USD per hour, in line with comparable awards. It is not the
maximum, on purpose.

The core primitive is already built and self-funded, which proves it works. The
grant funds the platform, the safety layer, and the independent audit.

| Tranche | Deliverable (verifiable) | Amount |
|---|---|---|
| T1, Trust core | Vault factory and registry, guardians and cancel window, multi-token, and a live testnet dashboard on a real wallet | 13,000 |
| T2, Platform | Backend and email/phone reminders, the keeper bot, and the passkey heir claim portal | 17,000 |
| T3, Mainnet | Professional security audit, fixes, mainnet launch, and pilot users | 12,000 |

Each milestone has a binary success test. For example, T2 is done when a non-crypto
heir claims via passkey on testnet, and T3 is done when the audit is published and
at least ten funded vaults exist on mainnet in the first month.

## 12. Business model

Grant-funded public good at launch, designed to become self-sustaining:

- Annual subscription (about 24 to 48 USD per year) to keep the vault active, with
  reminders and bot monitoring. This is the core recurring revenue.
- A small optional setup fee, and a tiny protocol fee (about 0.5%) at payout.
- Premium features: guardians, staggered inheritance, multi-asset.
- B2B and white-label: licensing the vault module to wallets, exchanges, and banks.
  This is the real scale play.

Simple unit economics: 1,000 active vaults at 36 USD per year is about 36,000 USD
per year recurring, enough to cover infrastructure and one developer independent of
further grants.

## 13. Ecosystem impact

- A new, public-good primitive that only Stellar's speed and fees make practical.
- Fully open-source, MIT licensed, with tests and an audit planned.
- Brings a real, human use case (protecting families) to the network, and a B2B
  path that can bring wallets and institutions onto Stellar.

## 14. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Owner locks themselves out with too-short an interval | App-enforced sensible minimums, grace period, and reminders |
| Accidental trigger from one missed check-in | Configurable grace buffer, cancel window, and guardians |
| Owner alive but silent (travel, hospital, jail) | Guardian veto, multi-channel and delegated check-in, cancel window |
| Contract bug moving funds incorrectly | Full test suite now, professional audit before mainnet |
| Keeper bot down when a vault becomes claimable | Claim is permissionless, an heir or anyone can trigger it |
| Team Soroban capability | Working tested contract as evidence, named contract owner, budgeted independent audit |

## Disclaimer

InheritVault is open-source, non-custodial software. It is not a legal will, a bank,
or an estate-planning or financial service. On-chain inheritance complements, and
does not replace, proper legal documents. Nothing here is legal or financial advice.
