# Threat Model

What Slotbook defends against, how, and what it assumes. Each defence names the
check in the program and the test that exercises it: LiteSVM tests in
`programs/slotbook/tests/`, devnet checks in `crates/demo` (see the README).

## Actors and trust

| Actor | Trusted for | Not trusted for |
|---|---|---|
| Issuer | Declaring and funding distributions; keeping `salt_seed` | The register (attestors build it); taking vault funds early |
| Attestor (bonded) | Nothing alone; a quorum of matching roots is needed | Any single root; a wrong root costs its bond |
| Resolver (one per issuer) | Recomputing one disputed owner under the spec and ruling honestly | Moving funds beyond the ruling; holding a dispute open forever |
| Challenger (bonded) | Nothing; a rejected challenge costs the bond | |
| Holder | Nothing; must prove their leaf | Claiming twice, someone else's leaf, a bigger balance |

The resolver is the main trust assumption in v1: one key decides each dispute. It must
be neither the issuer nor an attestor (`ResolverConflict`). If it does not rule within
`resolve_timeout_slots`, anyone can resolve `Expired`: commitments are dropped, the
challenger's bond is returned and attestors re-commit.

## Threats and defences

### Wrong register

| Threat | Defence | Tested by |
|---|---|---|
| One attestor commits a wrong root | A quorum of matching roots and totals before `Committed`; differing roots open a `Conflict` dispute | `full_lifecycle::conflict_path`, devnet `quorum_commits_root` |
| A late commitment disagrees after quorum | Must match the final root and total (`RootMismatch`) | `guards::late_commit_must_match` |
| All attestors omit or misstate a holder | Anyone challenges during the window; `Upheld` slashes every backer and they re-commit | `full_lifecycle::full_demo`, devnet `upheld_challenge_slashes_attestors` |
| The total is wrong, so every payout is wrong | A challenge naming the all-zero owner disputes `register_total` | `guards::register_total_challenge` |
| Attestors commit before the record slot is final | `finality_margin_slots` after the record slot (`FinalityMarginNotPassed`) | devnet `commit_before_finality_margin` |
| Attestors build to a different spec | `spec_version` must be 100 (`UnsupportedSpecVersion`) | devnet `commit_rejects_unknown_spec_version` |
| Issuer picks a record slot after seeing balances | `notice_slots` between declare and the record slot (`RecordSlotTooSoon`) | devnet `declare_rejects_short_notice` |
| Holders hidden from replay | The mint must default new accounts to Frozen and must not use confidential transfers (`init_issuer`, D15) | `init_issuer` checks |

Anyone can recompute a root from a register with the independent Python code,
`scripts/reproduce_root.py` (devnet `root_reproduced_independently`).

### Griefing the dispute process

| Threat | Defence | Tested by |
|---|---|---|
| Spam challenges | Each costs `challenger_bond`; a rejected one pays the resolver and the backing attestors | devnet `rejected_challenge_pays_resolver_and_attestors` |
| A second challenge while one is open | Only from `Committed` (`InvalidState`) | devnet `second_challenge_while_disputed` |
| Challenge after the window to delay payout | `WindowClosed` | `guards::challenge_after_window_is_refused`, devnet `challenge_after_window` |
| Someone other than the resolver rules | `NotResolver` | devnet `resolve_by_non_resolver` |
| Resolution with the wrong challenger, or a commitment passed twice or left out | The challenge, challenger and every live commitment are bound once (`WrongChallenge`, `CommitmentSetMismatch`) | `guards::resolve_binds_the_challenge_and_each_commitment_once` |
| Resolver never rules | `Expired` after `resolve_timeout_slots`, callable by anyone | `full_lifecycle::expired_path` |
| Attestor withdraws its bond before its root is settled | `LiveCommitments` until `release_attestor` | devnet `withdraw_bond_with_live_commitment` |
| Resolver fee eats the whole bond | `resolver_fee` must be below both bonds (`ResolverFeeTooHigh`) | `validate_params` |

### Stealing or blocking payouts

| Threat | Defence | Tested by |
|---|---|---|
| Claim a larger balance | The leaf is rebuilt from the claimed balance, so the proof fails (`InvalidProof`) | devnet `claim_with_inflated_balance` |
| Claim with someone else's leaf and proof | The leaf is rebuilt with the signer as owner, so the proof fails | devnet `claim_with_someone_elses_proof` |
| Claim twice | The owner's `ClaimReceipt` PDA already exists | devnet `double_claim` |
| Claim before the window closes | Only from `Open` (`InvalidState`); `finalize` waits for the window (`WindowNotElapsed`) | devnet `claim_before_finalize`, `finalize_before_window` |
| Pay a frozen (for example sanctioned) holder anyway | A frozen holder's payout goes to `Pending`, not to them | devnet `frozen_holder_goes_to_pending` |
| Release a held payout while still frozen | `NotTransferable` | devnet `release_while_frozen` |
| Release a held payout to someone else | Must go to an account the leaf owner owns (`NotLeafOwner`) | `guards::release_only_to_the_owner` |
| Issuer sweeps a held payout early | Only after `hold_window_slots` (`HoldWindowNotElapsed`) | devnet `sweep_before_hold_window` |
| Issuer closes and takes the vault early | Only after claim expiry with nothing pending (`CannotClose`) | devnet `close_before_claim_expiry` |
| Issuer drains a vault through the token itself | Payment and bond mints with PermanentDelegate, TransferFee, TransferHook or ConfidentialTransfer are refused | `guards::bond_mint_with_permanent_delegate_is_refused`, `guards::payment_mint_with_permanent_delegate_is_refused` |
| Block `claim` by pre-funding the Pending address | `create_pda_account` tops up, allocates and assigns | `guards::pre_funded_pending_address` |
| Block `init_issuer` by pre-creating the bond vault | `init_if_needed` on the vault | `guards::bond_vault_pre_created_by_a_stranger` |
| Change the rules mid-distribution | `update_issuer` only with no active distributions (`ActiveDistributions`) | `guards::close_declared_then_update_issuer` |
| Register someone else's mint | The signer must be the mint authority (`NotMintAuthority`) | `init_issuer` checks (D22) |

### Privacy

Leaves are salted with a seed only the issuer, attestors and resolver know, so nobody
else can test balance guesses against the root or a proof. This is not strong privacy:
balances at a slot are public on-chain anyway. See spec section 5.

## Out of scope in v1

- A dishonest resolver. One key decides each dispute; a resolver set or on-chain
  recomputation is future work.
- One operator running every attestor (as in the demo) removes the independence that
  the quorum is meant to give.
- Ledger history that RPC no longer serves. The reference replay needs history from the
  mint's creation; an indexed history store would make it a single query.
- Holders the issuer wants excluded (treasury, vaults): v1 has no exclusion list (spec 3.8).
