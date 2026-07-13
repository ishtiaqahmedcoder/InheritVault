# Testnet demo transcript

The InheritVault contract was deployed to Stellar testnet and exercised end to end.
This is the record of that run. Anyone can reproduce it with the Stellar CLI using
the commands in the project README.

## Deployment

- Network: Stellar testnet
- Vault contract: `CAWHXHUN2UG5C7VQNIO5UAIPIINVBQKGHN5YZN62B3ZF4OKTARLO7FPZ`
- Token used: native XLM (Stellar Asset Contract `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`)
- Owner: `GCLDOEIKGVZ2LQHJ5F7LUZYZFQYLPFDMDQELX6IISQGAWRFCY6FVRFFS`
- Heir: `GBZWWHLSM5LADPC4GJEZ4SPPVCLTO3VC5DRNYN6ZWC4AZQ6W7CIO5QC3`

## The lifecycle, verified on-chain

1. `init` with a single beneficiary at 100%, and a check-in interval. Transaction
   submitted successfully.
2. `deposit` of 5 XLM. The token emitted a transfer event moving 50,000,000 stroops
   from the owner into the vault.
3. Views returned live state: `status` was `Active`, `is_claimable` was `false`,
   `beneficiaries` listed the heir at 10,000 bps, and `time_left` counted down.
4. `check_in` succeeded and reset the countdown.
5. `claim` was called before the deadline and correctly reverted with
   `Error(Contract, #6)`, which is `NotYetClaimable`. This is the safety guard,
   demonstrated live.
6. The owner went silent. After the deadline passed, `is_claimable` returned `true`.
7. `claim` was called and succeeded. The token emitted a transfer event moving
   50,000,000 stroops from the vault to the heir.

## Result

- Heir XLM balance before claim: 100,000,000,000 stroops.
- Heir XLM balance after claim: 100,050,000,000 stroops (up by exactly 5 XLM).
- Vault balance after claim: 0.
- Vault `status` after claim: `Claimed`.

The full inheritance flow works on-chain: the funds reached the heir automatically,
the vault emptied, and it can never be claimed twice.
