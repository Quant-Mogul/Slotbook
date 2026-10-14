# Devnet test run, 2026-10-14

The run behind the screenshot in the README: `cargo test -p demo --test devnet -- --ignored --test-threads=1`
against program `3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp` on devnet. Every transaction links to the explorer;
the checks are listed at the end. Refusal checks are simulations, so they have no transaction.

Cluster: https://api.devnet.solana.com

Issuer `BMwEqnrGFYNPiKzmbhp1ahcb4XQvBuXXE7QEP6cmj6Lh`, program `3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp`.

## Setup
- fund 8 actors with SOL: [tx](https://explorer.solana.com/tx/5n8tnxRaRPEgKU64w8xXw4LG7qaaZoyqpLbPMQhYYK1APhuQXdQfsNfc7QpwPXYHtA2U6krYhEV4St6JLzrqq4bA?cluster=devnet)
- create payment mint (test USDC): [tx](https://explorer.solana.com/tx/4txweeMFihKxUmZcmMirPUYoCzCtEqQFexccMhbaqgbGboJzfye36VW6oqB6LYytgqDZFFJnY4tzkGCm1g4379n9?cluster=devnet)
- create bond mint (test USDC): [tx](https://explorer.solana.com/tx/sjA3NaKiR15FqB87qFViDtceo7eUZygjzguWRHHxTeztjnR6oygSGThu469SHsmAGKzWbkLpLc6ZtHUGrxUheDE?cluster=devnet)
- create permissioned Token-2022 mint (DefaultAccountState = Frozen): [tx](https://explorer.solana.com/tx/gVDtXjbVZfXUkMJby71HhvaS8FVyyL8saRWM2Ue9quzUQzWHmH7kPGiMdnCmcGwD5CiFjJ2yVx7vhAJ8oM2unuw?cluster=devnet)
- Token ACL create_config (freeze authority -> MintConfig PDA): [tx](https://explorer.solana.com/tx/hhLGiaBM7jnTQcKyzgRBScWy1BuBr3DfKYtYwnLnrxdx4PvRyxnEUg7Fq8S5gXxnmXJedDw8P8qFmWwAeF2pZf3?cluster=devnet)
- holder A (thawed, paid) accounts: [tx](https://explorer.solana.com/tx/66PBUMPcPuWh7FnDrgauGvoxwbLvn3WJTFDL36nnM3AiTMQpevUjoBMAvoaDdx8nggd79ToobAq6pCKyF4wzQfgj?cluster=devnet)
- holder B (frozen, released after thaw) accounts: [tx](https://explorer.solana.com/tx/2XBZmWAL7Qqn67pkGG719XupQnSQtmoTJ6wjUiz4yLmEE321AR7EZLMT4TwpJKvm7hgfnu1anTHK9Dz7VKpzV5H?cluster=devnet)
- holder C (frozen, swept) accounts: [tx](https://explorer.solana.com/tx/4Kmgbrgmnvu3yWDSUdCyakJR6Fpr7PZXG8tVwjNFDVty42z2r3W3bGPFE9E1Dch11sZTqn7cwGEXiMyWzYye3F1t?cluster=devnet)
- history: mint 700 to A, 300 to B: [tx](https://explorer.solana.com/tx/2pG4nbVijBWnaFhtwaHUKYzFMbqhgMTWoVz4J2W5E2U4VGZfSxEnFFZP6nYRpZARbbUVzDoyNqcSb3zLCjAxn7mW?cluster=devnet)
- history: A -> C 150: [tx](https://explorer.solana.com/tx/3BcwWxPgPS9aHSF9ov7Rnv5qWC41nvApCELW5HeVwFhCE7Egcwsv4LH6YTps6tUMF7NS12Ejoke8Gqrn5a6sJt42?cluster=devnet)
- history: A -> C 50: [tx](https://explorer.solana.com/tx/2N9cPw2UJuiuW6AykquMzUXsHP8M6VXswHNWqp9M2zRWe9qQj96GeCbr4U7kkkf9zVqDphK6Pp9foe52t7A8kyFz?cluster=devnet)
- fund test USDC: issuer total, attestor and challenger bonds: [tx](https://explorer.solana.com/tx/5ZNfxzNnoAyBSiaVzT356aGeNKg4beDAEECR5ogEzLY4bSVspvgDntkAYCgjpV2K34rfTSJSWDPgXhc6AsYE5cZL?cluster=devnet)

## Addresses

- permissioned mint `Gq8m3duRLYCCzjcYTpjZP8T1j7cbXU3DqEzEbTzbgU1m`
- issuer config `GTHGoYScq3MqbNQD4Xc2jzSLA64BrMKB5PFYQHGZuzmv`
- distribution `B9oZKau5JjRp9MvMfLHdmDEp49KQrRBnPVELxz7jCKBd`
- distribution vault `5eMy4FwZtTkA9zBmH4jy9uFcATQ7iK29rrZg3GwTPPe8`
- bond vault `EtVaGDbLoJWd9wPNMNis4iTNr5k7Pie2xmf8Pfrg2amb`

(Full list in addresses.json.)

## Step 1: issuer registers the mint and declares a distribution
- init_issuer: [tx](https://explorer.solana.com/tx/2HshCwLkmWCBobRrYhJsvjkL5XZJnfqpzfr75eGTfAHgfwboRnroV7swQmT2pJpqrzZbbR9eJpuycLCGDWZdr5QY?cluster=devnet)
- PASS `issuer_registered`: IssuerConfig for mint Gq8m3duRLYCCzjcYTpjZP8T1j7cbXU3DqEzEbTzbgU1m: 2 attestors, quorum 2
- register_attestor GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG: [tx](https://explorer.solana.com/tx/5X7ViSAajFWdR7duLgE6XSs7Ztus2H8jGh7GMSvC3HDv6LvdsxH9P7KrehRmKqmQmXdoTcdWZfaVDE8m3BKZjhv?cluster=devnet)
- register_attestor r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx: [tx](https://explorer.solana.com/tx/55gCDAjLBJzJoqQ296hn2EUeGT9fGNBcDSrvqSozwhp6zZtppPu2MK3dcGPrZGnC7BWZFToTfsSwmitPA43Vmhrm?cluster=devnet)
- PASS `attestors_bonded`: each attestor bonded 10 USDC; bond vault holds 20 USDC
- PASS `declare_rejects_short_notice`: refused (Error Code: RecordSlotTooSoon.)
- declare_distribution (record slot 510718888): [tx](https://explorer.solana.com/tx/4txkREYmEbVQKG4cdimZK7N7QF9GYxeX4RRTnE2p93UqHFisK4XWR7Md2Xo7FDV4GKTcBwweJUmA7PdzGNBoeSJ6?cluster=devnet)
- PASS `distribution_declared_and_funded`: Declared, record slot 510718888, vault holds 1000 USDC
- history after declare: A -> B 10: [tx](https://explorer.solana.com/tx/2D19sSRYeuGwXYLAJxGk63HXmJVF2SNz81VZAm8UxnNqQyvcgowyLYCydnPzYSLGManHPjfL8Wwpv7Vt5xLNuvRQ?cluster=devnet)
- history after declare: B -> A 10: [tx](https://explorer.solana.com/tx/362ke5A3gSefgMFdnv6NyRJZ169dTFugmZbQWamNH4o1KiXTR8KaW1WVPFFysNeUgRrB3MuVB2sRmGeeCfpoH5fM?cluster=devnet)
- PASS `history_gives_expected_balances`: A, B, C hold [500, 300, 200] of the permissioned mint (expected [500, 300, 200])

## Step 2: two attestors commit matching roots after record slot + margin
(The first register deliberately omits holder C, so step 5 can uphold a challenge.)
- PASS `commit_before_finality_margin`: refused (Error Code: FinalityMarginNotPassed.)
- PASS `commit_rejects_unknown_spec_version`: refused (Error Code: UnsupportedSpecVersion.)
- commit_root by GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG (2 rows): [tx](https://explorer.solana.com/tx/4fUgZLjEnMndUVgBpJbeLez4YzVdrvyCmbPZo19usXiLWe7DXF6RQkwbypEDhHtZaDqseimHbwZFN8MBMiNWPeUR?cluster=devnet)
- commit_root by r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx (2 rows): [tx](https://explorer.solana.com/tx/4xBzuQsPQyLXhcTqE8NKAHEPpdHNPwFTmig1fTYqv16ibhFRCGqtEgbFw6aB5bPMw86WmKLJN3gQ6Rr4FgAhLJbV?cluster=devnet)
- PASS `quorum_commits_root`: Committed, register_total 800, window ends at slot 510719031
- PASS `finalize_before_window`: refused (Error Code: WindowNotElapsed.)
- PASS `claim_before_finalize`: refused (Error Code: InvalidState.)

## Step 3: the root is reproduced from the spec by an independent script
`python3 scripts/reproduce_root.py target/devnet-test/register_committed.json`

```
rows 2  register_total 800  root a5798069dba4685c5b6fd342e783015dfb898cc4a04581eba0bbf4831fe0edda
MATCH: the on-chain root is reproduced byte for byte from the spec.
```
- PASS `root_reproduced_independently`: the Python spec implementation recomputes the on-chain root

## Step 4: a wrong-balance challenge is rejected
- challenge #0: owner CGFsHkCJA5oDiiHh66iQLddgv4exBpZ2Tsf3eQKEZcmh balance 999: [tx](https://explorer.solana.com/tx/2wCee6ysQ5eJJzH7RFevcTgna3viARnoLguSdZTWDLow4dFhKNaf6z667Bxi4QGUBZZUzmo6vkmgZY6hWk9x4y5Y?cluster=devnet)
- PASS `challenge_opens_dispute`: Disputed; challenger bond 10 USDC locked
- PASS `second_challenge_while_disputed`: refused (Error Code: InvalidState.)
- PASS `resolve_by_non_resolver`: refused (Error Code: NotResolver.)
- resolve_challenge #0: Rejected: [tx](https://explorer.solana.com/tx/2Pjyrud3YiCoMD4LexHQFuiyNqquZG3AdRcDYpx3isoqDmRNEH89nRbNjqQ9Ww55bzz5uSt6FuoiDqkJMPzzpYH5?cluster=devnet)
- PASS `rejected_challenge_pays_resolver_and_attestors`: Committed; resolver fee 1 USDC, each attestor's bond 14.5 USDC (+4.5 USDC), challenge account closed

## Step 5: a challenge naming the omitted owner is upheld
- challenge #1: owner 62YNaiwjhYCKUqeYRrySUgtCxT8XSX4KhRkF8rHWtEs6 balance 200: [tx](https://explorer.solana.com/tx/4B6iUXBBsGwYXuraJ3YcPFhEQCQDvuK6mS1taaXQPJ2N65gLBU5m4dLJGPiCKm2RkGoDx69ehWRjtAgcgoUd5nuf?cluster=devnet)
- resolve_challenge #1: Upheld: [tx](https://explorer.solana.com/tx/T8bMNYz6FjCbESVofTr4QgLUTMPbscEsGVLKwDCD9NpzLro19HdJUXJPLvXQtCma6iXrZqZ5VmiheC4hhigLLdk?cluster=devnet)
- PASS `upheld_challenge_slashes_attestors`: Declared, commitments closed; challenger holds 28 USDC (bond back + two slashes less fees), resolver 3 USDC, attestor bonds 4.5 USDC
- PASS `no_cancel_after_record_slot`: refused (Error Code: CannotClose.)
- withdraw_bond GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG: [tx](https://explorer.solana.com/tx/3pP6Uv1TdvrneHVvAK6YdntpiHRReCroeE5SpuCKAkYimzQ7gHDkMcD1knqddqo6iNiDy3ME3xN5B5rCGWJQDtKH?cluster=devnet)
- register_attestor GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG: [tx](https://explorer.solana.com/tx/2rJ6CJpbasKe31a2NX4ts38oknCXfs1r4DVLKeBHawuHYufeweTpWkB5TFqxwF76QVQ4BTt9QNkC4YutNcvCnNut?cluster=devnet)
- withdraw_bond r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx: [tx](https://explorer.solana.com/tx/2Mg7HETaoafcg5d9jLpQSu4fxac7ukLZpCqdH56FEXzRDDJPwGvxCdKPa6cqzitCH5ZNsTg9s1nHZZk7uwrjabQW?cluster=devnet)
- register_attestor r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx: [tx](https://explorer.solana.com/tx/3junf82q7tESL1YJLCLCvYk4qE9zdtUb6nMvTHZ5KHLgDmUd1v24TgYcmj5Rh5htAkTLM7kvLdVYDYNUKgFbf4cR?cluster=devnet)
- commit_root by GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG (3 rows): [tx](https://explorer.solana.com/tx/4GF7JcW4Zsonku32nv8FGcbrDftv837HWBeq2UYrpg9W2qyuLN6NecuedDJhRnffB4XNEGmNJbSP2k2PHG4aaJSh?cluster=devnet)
- commit_root by r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx (3 rows): [tx](https://explorer.solana.com/tx/4vYZNZg2bssfyQyS6vJj6Ejitw7iRP6sBNpSJyvpfzRjmcxmSxfCkdSGBetLRfEYm1Zryr3LNVHnspDGGEaF9Y1q?cluster=devnet)
- PASS `corrected_root_committed_and_reproduced`: Committed, register_total 1000, root reproduced by the Python spec code

## Step 6: window elapses, finalize, a thawed holder claims and is paid
- PASS `challenge_after_window`: refused (Error Code: WindowClosed.)
- finalize: [tx](https://explorer.solana.com/tx/ysTqornyAfS36Y8mRXwkhrDdP88oGPVAirj2fsZKx6mBDcH93bYGi5GLYHvzntMUYqRXvjWxWyN5s7aigX4jeYA?cluster=devnet)
- PASS `finalize_opens_claims`: Open
- accounts for an outsider (payment and permissioned mint): [tx](https://explorer.solana.com/tx/4X17rYNkgBG7iziGgrPTqycfUdFRkaNqc6Hpnomou5bEySWHwSysgVVgy5RHP51vPeGuwwwxSUkPK2tLK9E2YwZc?cluster=devnet)
- PASS `claim_with_inflated_balance`: refused (Error Code: InvalidProof.)
- PASS `claim_with_someone_elses_proof`: refused (Error Code: InvalidProof.)
- claim by A (thawed, paid): [tx](https://explorer.solana.com/tx/35K1GCNrZgf4rAFuAKGYfP9tMARvP7JDXKNih6y1MDx51XG26gknUFK2ZVu7AT3F5XETMqwUBFt15yYq6WRrdtsN?cluster=devnet)
- PASS `holder_paid_pro_rata`: A held 500 of 1000 and received 500 USDC of 1000 USDC
- PASS `double_claim`: refused (already in use)
- PASS `withdraw_bond_with_live_commitment`: refused (Error Code: LiveCommitments.)

## Step 7: a frozen holder lands in Pending, is thawed, and is released
- freeze B and C (Token ACL authority freeze): [tx](https://explorer.solana.com/tx/5JxicWsUi7Y6PYYZTvJmdwricrtq8jrDZuQ3c5tE5a4jhNwtcBegaZTXUjBM72BcuZKPc8vk87hJTXhG4Xzsm3Tw?cluster=devnet)
- B's side account (Token-2022, ImmutableOwner, not the ATA), thawed: [tx](https://explorer.solana.com/tx/2yMFBirQM5L9gtNAU5tNzopgrqSjPQD3KVf5aKBWibEiQrjnqEceErXZuVUPKtXRVPKYqDhbK3nCteCEDWQNyJ9g?cluster=devnet)
- PASS `frozen_holder_cannot_claim_through_side_account`: refused (Error Code: NotHolderAta.)
- claim by B (frozen, released after thaw): [tx](https://explorer.solana.com/tx/4XAExydjcisbsxpjiVZav1WDJkQY4pfhCnjWNq2W8FV7biiTimQQEDopinFqHVRxXZa2YsMVVGqLjRcLRC7mhWVS?cluster=devnet)
- PASS `frozen_holder_goes_to_pending`: B is frozen: 300 USDC held in Pending, nothing paid yet
- thaw B (stand-in for the gate's permissionless thaw): [tx](https://explorer.solana.com/tx/3ZTYdAai7dRomfMDH7fsTRbwrerabM6KLhDhrzBv1nLjQorcA48k9vAkhFPCfBAGweBZ9vmzvpL8mRnjZ17E1xD9?cluster=devnet)
- release_pending for B (crank): [tx](https://explorer.solana.com/tx/Pfg95JFYYMvtsHYC2m47CxxQRkUShgk6KYo3jZXLbrmZznXCW6NTadNrWM51KgZz8oZv8RBTB4p79tRQm9b1Tfd?cluster=devnet)
- PASS `pending_released_after_thaw`: B received 300 USDC; Pending closed

## Step 8: hold window elapses, issuer sweeps, distribution closes, attestors leave
- claim by C (frozen, swept): [tx](https://explorer.solana.com/tx/4gohdomXkFKfKLxa8VuQGBdyfYrpPDbmkcq1nnmxLcYgxHxfVkBZ6KXyp9D2Nn3uspcqFYgitnKFuY8FpPCkeRnS?cluster=devnet)
- PASS `release_while_frozen`: refused (Error Code: NotTransferable.)
- PASS `sweep_before_hold_window`: refused (Error Code: HoldWindowNotElapsed.)
- sweep_pending for C: [tx](https://explorer.solana.com/tx/jEtxYBGf3kuFXcwKKhN6Fqu5m28fFpcPCuvhn4Gdx3Zprv59iRm1vh9D5pZQrUpe6NcYJy1NZoPCstyiAUuLYSh?cluster=devnet)
- PASS `unclaimed_pending_swept_to_issuer`: C stayed frozen; 200 USDC swept back to the issuer
- PASS `close_before_claim_expiry`: refused (Error Code: CannotClose.)
- close_distribution: [tx](https://explorer.solana.com/tx/GAEPpsXap6dzZrazHjihP4paUsDBzJVtkKzJAFcg1sMWkS3XkkU9HMGZegVQYgdm8V1MzDUxPDWRgQKKTRMhidc?cluster=devnet)
- PASS `distribution_closed`: Closed; claimed 800 USDC + swept 200 USDC = total; vault closed
- release_attestor GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG: [tx](https://explorer.solana.com/tx/5LgR7aCkPwAjiAczTgqrXS1f9aKFAdKErEgqK8zdkoCQdYWu6qC2Ya556GSXKXMYpA22wZyh32JrxTLj2Jhv5gDb?cluster=devnet)
- withdraw_bond GaKZ1TxJy8qotPYuk82ww9fNVFKoXbjFSfLqGwgNL9xG: [tx](https://explorer.solana.com/tx/42QfDbaya4pETuZr2noV4cCQCMSwASZLumUyDWt6AiDaRoGZ8fe78JKhhBNG8NLoGfGzEVdi61JxFEz3rhCULUAd?cluster=devnet)
- release_attestor r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx: [tx](https://explorer.solana.com/tx/HToEAywdNsit25UvWu9hDxBQHvVy1qDoo3vaPRCitqXWxyAcXgHwfCK1dtrCxMVQYTmob2ERMNct1HH48D8Wy7K?cluster=devnet)
- withdraw_bond r1oRBP2HA9V41NG53K9RAgLbqagnmvtrXvLqFh6TRtx: [tx](https://explorer.solana.com/tx/66tJR2tu7wJwas9x4GC9jx9kCYtW93ufjiCc6Z9yUTh5BXXsnTziWfDJ1thNdvgsrrB2HdhqgnXPWGntyFPm6DGc?cluster=devnet)
- PASS `attestors_exit_with_bonds`: attestor accounts closed, bond vault empty; each attestor ends with 14.5 USDC (lost one bond to the upheld challenge, earned 4.5 USDC from the rejected one)

34 of 34 checks passed.

## Step 9 (not on-chain)

Superbank design proposal: https://github.com/solana-rpc/superbank/issues/110 (docs/SUPERBANK_PROPOSAL.md).

## Checks: 34 of 34 passed

- PASS `issuer_registered`: IssuerConfig for mint Gq8m3duRLYCCzjcYTpjZP8T1j7cbXU3DqEzEbTzbgU1m: 2 attestors, quorum 2
- PASS `attestors_bonded`: each attestor bonded 10 USDC; bond vault holds 20 USDC
- PASS `declare_rejects_short_notice`: refused (Error Code: RecordSlotTooSoon.)
- PASS `distribution_declared_and_funded`: Declared, record slot 510718888, vault holds 1000 USDC
- PASS `history_gives_expected_balances`: A, B, C hold [500, 300, 200] of the permissioned mint (expected [500, 300, 200])
- PASS `commit_before_finality_margin`: refused (Error Code: FinalityMarginNotPassed.)
- PASS `commit_rejects_unknown_spec_version`: refused (Error Code: UnsupportedSpecVersion.)
- PASS `quorum_commits_root`: Committed, register_total 800, window ends at slot 510719031
- PASS `finalize_before_window`: refused (Error Code: WindowNotElapsed.)
- PASS `claim_before_finalize`: refused (Error Code: InvalidState.)
- PASS `root_reproduced_independently`: the Python spec implementation recomputes the on-chain root
- PASS `challenge_opens_dispute`: Disputed; challenger bond 10 USDC locked
- PASS `second_challenge_while_disputed`: refused (Error Code: InvalidState.)
- PASS `resolve_by_non_resolver`: refused (Error Code: NotResolver.)
- PASS `rejected_challenge_pays_resolver_and_attestors`: Committed; resolver fee 1 USDC, each attestor's bond 14.5 USDC (+4.5 USDC), challenge account closed
- PASS `upheld_challenge_slashes_attestors`: Declared, commitments closed; challenger holds 28 USDC (bond back + two slashes less fees), resolver 3 USDC, attestor bonds 4.5 USDC
- PASS `no_cancel_after_record_slot`: refused (Error Code: CannotClose.)
- PASS `corrected_root_committed_and_reproduced`: Committed, register_total 1000, root reproduced by the Python spec code
- PASS `challenge_after_window`: refused (Error Code: WindowClosed.)
- PASS `finalize_opens_claims`: Open
- PASS `claim_with_inflated_balance`: refused (Error Code: InvalidProof.)
- PASS `claim_with_someone_elses_proof`: refused (Error Code: InvalidProof.)
- PASS `holder_paid_pro_rata`: A held 500 of 1000 and received 500 USDC of 1000 USDC
- PASS `double_claim`: refused (already in use)
- PASS `withdraw_bond_with_live_commitment`: refused (Error Code: LiveCommitments.)
- PASS `frozen_holder_cannot_claim_through_side_account`: refused (Error Code: NotHolderAta.)
- PASS `frozen_holder_goes_to_pending`: B is frozen: 300 USDC held in Pending, nothing paid yet
- PASS `pending_released_after_thaw`: B received 300 USDC; Pending closed
- PASS `release_while_frozen`: refused (Error Code: NotTransferable.)
- PASS `sweep_before_hold_window`: refused (Error Code: HoldWindowNotElapsed.)
- PASS `unclaimed_pending_swept_to_issuer`: C stayed frozen; 200 USDC swept back to the issuer
- PASS `close_before_claim_expiry`: refused (Error Code: CannotClose.)
- PASS `distribution_closed`: Closed; claimed 800 USDC + swept 200 USDC = total; vault closed
- PASS `attestors_exit_with_bonds`: attestor accounts closed, bond vault empty; each attestor ends with 14.5 USDC (lost one bond to the upheld challenge, earned 4.5 USDC from the rejected one)
