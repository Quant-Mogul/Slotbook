# Devnet test run, 2026-10-12

The run behind the screenshot in the README: `cargo test -p demo --test devnet -- --ignored --test-threads=1`
against program `3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp` on devnet. Every transaction links to the explorer;
the checks are listed at the end. Refusal checks are simulations, so they have no transaction.

Cluster: https://api.devnet.solana.com

Issuer `BMwEqnrGFYNPiKzmbhp1ahcb4XQvBuXXE7QEP6cmj6Lh`, program `3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp`.

## Setup
- fund 8 actors with SOL: [tx](https://explorer.solana.com/tx/5qjsw9fJzNwphUBMS8bbrr3YHPE4uyEnx2SSPGRxogeE4p1b5V91tGXXnNacbd8TWGGisLNi7BR8dax4kL4WSUiY?cluster=devnet)
- create payment mint (test USDC): [tx](https://explorer.solana.com/tx/3edqHkrzsh9uCJ1L4YcHyDkS7NngGs9E8aT86P2e6J2FHFCUNYVZyzdq7oZ4eRtcYs1H6f81FwebNfecMHuEJpo4?cluster=devnet)
- create bond mint (test USDC): [tx](https://explorer.solana.com/tx/vqCZ3crZ7J11N4xXe58NLiZUx7NVG8JPNTWwqEKrgUEWRYZS2vdKfFBmyzT3DPpgwWXzA9TTNCBwfsfeFQxQuwR?cluster=devnet)
- create permissioned Token-2022 mint (DefaultAccountState = Frozen): [tx](https://explorer.solana.com/tx/4oyMLbA1yU4vo8ofJaZfyaXdiUqLmb9ivr2zuEN8nTKZ3tvCZbzLFQfbgBr7S5dUTPr3gavjerb7DoyCQ8bQXGi7?cluster=devnet)
- Token ACL create_config (freeze authority -> MintConfig PDA): [tx](https://explorer.solana.com/tx/5H7UfGpH1LeR39V8iRt8NnhC6dopedkwVEoQ8VAPZQ2K9nwAoDizNHbXKnhydKrcxuMdMdnY3PmVxjfKNDf7yrN6?cluster=devnet)
- holder A (thawed, paid) accounts: [tx](https://explorer.solana.com/tx/5b3fSFN3irLV9egYJCqkh3SfPfRT8CDrMv5jwWJNULTXCSApQfVWqco2HRdHzNoq41ynWroZXrJbZS9UTMJLV7o?cluster=devnet)
- holder B (frozen, released after thaw) accounts: [tx](https://explorer.solana.com/tx/5h8FGZUpSXes6ufzUzVPgV2AVoTxKYUCVwPgvGzhsmJDampHcUp4czA1DDXejxivC1hPuKjp4srWpq4WK4FhKokA?cluster=devnet)
- holder C (frozen, swept) accounts: [tx](https://explorer.solana.com/tx/3zFGsEv3wMtMowX3q682GyNrk1d4Z3bT3d52AfstS1bt2nitQtZWD2RoyECfmgB7Jd8nJGh49cqFVr656WdV16BH?cluster=devnet)
- history: mint 700 to A, 300 to B: [tx](https://explorer.solana.com/tx/vxbajRGtLbxbfTdt5s5Ub8tq2Bib6SCWJ4U5WB6MSDj67fcW6dxekpU9zYfiCZLY7QUFVPr67P7FmZ5AN8JQSbB?cluster=devnet)
- history: A -> C 150: [tx](https://explorer.solana.com/tx/4wsjQaZz7SND3txhUeMJ8MEfXnbzHu2h4wNzPPHqH38pQDfSLDTHrcsD8VVAkxV3SFTJdzk1g63La5iMp48UTd3g?cluster=devnet)
- history: A -> C 50: [tx](https://explorer.solana.com/tx/Fs8cRK6YjU9xe8TzBDvS7qsT3JAYvLcYJa7FN8zPuHF9vq2PipdqLpfuFDRc1WUCjjr7oJELgufqXE3uVZ7pbea?cluster=devnet)
- fund test USDC: issuer total, attestor and challenger bonds: [tx](https://explorer.solana.com/tx/2xkkE97vLU1W7TAYxAdj9J6H8iyQn8P19XermPDp74v6SFeD7Q3gU2zmZh2ppZ4Cy2mEse541xBt5BAmmdrgQdBH?cluster=devnet)

## Addresses

- permissioned mint `Cjj8zScNftRF1xLqmg3N9Qh6qWHB31JXyXNVobCcSYXt`
- issuer config `HZw8tL7LezN4rWaZKxjx8prTuaGmN3G66KvYJKrpXrBU`
- distribution `9aKad9kxnpN7Cr7NaUWntHSxgJnBZmcatgpJKZSD6YA3`
- distribution vault `7PnsiYQ1CEXg7EE5LsTBRuE2ZLxP6vyHJ7LkD1QxC8T9`
- bond vault `BhLH3Wnw5YAsYiTEbaEeFBym1JeSgc5642BMDLaoBoEX`

(Full list in addresses.json.)

## Step 1: issuer registers the mint and declares a distribution
- init_issuer: [tx](https://explorer.solana.com/tx/ULLKf6kgNuxiQqX6cU6fDyi1rByRAfLs84dBFeb4Rv3ibFKBGR7BZixfDrXdtUF5uniG3TAsAeYkyJgg2PBzNJj?cluster=devnet)
- PASS `issuer_registered`: IssuerConfig for mint Cjj8zScNftRF1xLqmg3N9Qh6qWHB31JXyXNVobCcSYXt: 2 attestors, quorum 2
- register_attestor 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM: [tx](https://explorer.solana.com/tx/3WUggxcSHsjf2tbDFJmhkG3d1nW7FyYTBGqdPDsBJ4LG5TwXaH1HX67NmdaQcP2rN8eGEb2PF57hor9XrgV1GAMc?cluster=devnet)
- register_attestor BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq: [tx](https://explorer.solana.com/tx/4LGPaBkntnQFBt7an84EvBvKYWBb8zBEa2hBx6PMTVdQhfLdnovBHnrWD8q1zJgmuyiJNsURrtbrTE4yrXZCjWWb?cluster=devnet)
- PASS `attestors_bonded`: each attestor bonded 10 USDC; bond vault holds 20 USDC
- PASS `declare_rejects_short_notice`: refused (Error Code: RecordSlotTooSoon.)
- declare_distribution (record slot 510328664): [tx](https://explorer.solana.com/tx/2Pnr1rZqmvZtCxSiLyFXuVrPwstEd93jEWUAdsa29cuTgmt3J6VYAnPnvinZw9eirGgpsYbj1yKN4Jqi6fpjRnkx?cluster=devnet)
- PASS `distribution_declared_and_funded`: Declared, record slot 510328664, vault holds 1000 USDC
- history after declare: A -> B 10: [tx](https://explorer.solana.com/tx/3i94FfeMpo8NC4LkWnz5gGTh5Xr1FV4MU1d4YgwvFaoELVc7aZLmMH7goHoFMozDHBzmMVMzgGmC4933Y7HWmya4?cluster=devnet)
- history after declare: B -> A 10: [tx](https://explorer.solana.com/tx/5NqoSScS9bYkJaZCdRfx5wdLRsY3Lj7sHtzBydwxx4DpzG9gitfVHSQ4YJnwZt2SgZTWYjimKv8yFJT1mAjJ8DfH?cluster=devnet)
- PASS `history_gives_expected_balances`: A, B, C hold [500, 300, 200] of the permissioned mint (expected [500, 300, 200])

## Step 2: two attestors commit matching roots after record slot + margin
(The first register deliberately omits holder C, so step 5 can uphold a challenge.)
- PASS `commit_before_finality_margin`: refused (Error Code: FinalityMarginNotPassed.)
- PASS `commit_rejects_unknown_spec_version`: refused (Error Code: UnsupportedSpecVersion.)
- commit_root by 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM (2 rows): [tx](https://explorer.solana.com/tx/2zvp4a4YLYe7DiTe7HjeD5nuyJSXiruA715EvMFAacm9wm9EhuVgmH2rhGa2hRf81u5GCoAsvnKAC5f16Xtkii2L?cluster=devnet)
- commit_root by BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq (2 rows): [tx](https://explorer.solana.com/tx/4RXVdUHFthAcHu2aaKQTmzBr2Ck2dy8Nd7WWtVWdKrURvucdfgo9yRb4kHB3X9eKyLC3UjtpEyDbVgu1Vfm2EdXD?cluster=devnet)
- PASS `quorum_commits_root`: Committed, register_total 800, window ends at slot 510328808
- PASS `finalize_before_window`: refused (Error Code: WindowNotElapsed.)
- PASS `claim_before_finalize`: refused (Error Code: InvalidState.)

## Step 3: the root is reproduced from the spec by an independent script
`python3 scripts/reproduce_root.py target/devnet-test/register_committed.json`

```
rows 2  register_total 800  root 95c076e690e5f475488eb045dffb73ad50b95fd202a46f356e775d6d12f4fa58
MATCH: the on-chain root is reproduced byte for byte from the spec.
```
- PASS `root_reproduced_independently`: the Python spec implementation recomputes the on-chain root

## Step 4: a wrong-balance challenge is rejected
- challenge #0: owner 9V3LNVvdTXKZZper9s39JeesDRPQ1W1yfy66rUYtiyvX balance 999: [tx](https://explorer.solana.com/tx/Fvkj6ReVBd98m73JTGoLvuRh2pUvmZYna2x4MExhuDoodBkRc82Fp4EVuTA54sBipdYngBrG4DwtxdfpAeYiF9D?cluster=devnet)
- PASS `challenge_opens_dispute`: Disputed; challenger bond 10 USDC locked
- PASS `second_challenge_while_disputed`: refused (Error Code: InvalidState.)
- PASS `resolve_by_non_resolver`: refused (Error Code: NotResolver.)
- resolve_challenge #0: Rejected: [tx](https://explorer.solana.com/tx/64VD3crHpqVb1g1SAX45DH9AuLmQ9dLK1EtxbyLaUgykQ5bjJodLyiwJJCBs9TGypqGSNWFmhH6hHv5niua27nRk?cluster=devnet)
- PASS `rejected_challenge_pays_resolver_and_attestors`: Committed; resolver fee 1 USDC, each attestor's bond 14.5 USDC (+4.5 USDC), challenge account closed

## Step 5: a challenge naming the omitted owner is upheld
- challenge #1: owner 8wz9wD85a3786Yqe4dtPY1edctnC632QErPeJiYbpvQ8 balance 200: [tx](https://explorer.solana.com/tx/4EGy7NA1hZ6BNU297A6S43gMNqeF2dYLgux5ehx6twyurwzQfmvC2W7HuuA8aAdDgF3c79geEEgRxCWwBj5Qr855?cluster=devnet)
- resolve_challenge #1: Upheld: [tx](https://explorer.solana.com/tx/3ahT4VHip6oRJtPA51GpMpLGAGwJ5EjQHQXqhzwLzEA8jpXKhN5ZqCbjiqqktE4pEVr2NBTQ9SKeVJkKd38u2zfa?cluster=devnet)
- PASS `upheld_challenge_slashes_attestors`: Declared, commitments closed; challenger holds 28 USDC (bond back + two slashes less fees), resolver 3 USDC, attestor bonds 4.5 USDC
- withdraw_bond 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM: [tx](https://explorer.solana.com/tx/5vvdj52HF32nZVXpKnVQ8kguabVb4FkAjtBFXNSyQ8yWVDGqtvTaQEsqE8FrhbftMeA8GtrmdhbRkoirwmKf2j6w?cluster=devnet)
- register_attestor 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM: [tx](https://explorer.solana.com/tx/5G4CBASXuLbxkjqpwZXcwbornb9nT6bymF5MmMiXeiPCRdtdW88jKj1rczFGjn3aQzzHJsJRUuVtqybNWQfnJpJW?cluster=devnet)
- withdraw_bond BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq: [tx](https://explorer.solana.com/tx/4WJyFEkzqZMDw1jqJKw8AFt3gkC4ygPJWiU2AbjYqLPJHSHCBhNtaBfvzFxRSFGX2jjbEUBjnJXkMmGfJPVqrA87?cluster=devnet)
- register_attestor BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq: [tx](https://explorer.solana.com/tx/4NQC24wdnmzivnHm6ty4oHMRkqvRXkZroC12byg8pUg2n1aXGTA1dsgFP4Dzgz12Fs6oB6gUVLJRprzyCU45PDa8?cluster=devnet)
- commit_root by 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM (3 rows): [tx](https://explorer.solana.com/tx/3nxfDCo7JmA6GW8V8W6vjjtxFQvYPUXEF357QEhp6BvHottNTceFrdhGG8Yda3wJCNFJv7Sg6hEiLrxGCXfPix8n?cluster=devnet)
- commit_root by BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq (3 rows): [tx](https://explorer.solana.com/tx/4BjzSmZuvMNzYU6yM9ngdDYEDEei6ptPbt1GUYh5jNXEmSAH8MxQytdjYde7tpdvw2iuWPGDEUNUtu8JrB9fon5v?cluster=devnet)
- PASS `corrected_root_committed_and_reproduced`: Committed, register_total 1000, root reproduced by the Python spec code

## Step 6: window elapses, finalize, a thawed holder claims and is paid
- PASS `challenge_after_window`: refused (Error Code: WindowClosed.)
- finalize: [tx](https://explorer.solana.com/tx/2QgA3xJZVbkrWuA3HUHJZB5jZ3sUg9Sj2iwtyy86Z7Q3eE11ngwRT2TtzakqfvDdGz9m4mKg9STSTBUpnJuj6mrN?cluster=devnet)
- PASS `finalize_opens_claims`: Open
- accounts for an outsider (payment and permissioned mint): [tx](https://explorer.solana.com/tx/5QsqC7MRgLn4Somj3WctHbV45BWQaUqDgJsLTB3qS4ptgogi9Ppt7VCP526xzzr9EbaM2rx2kZGMCEq4w1JZA15P?cluster=devnet)
- PASS `claim_with_inflated_balance`: refused (Error Code: InvalidProof.)
- PASS `claim_with_someone_elses_proof`: refused (Error Code: InvalidProof.)
- claim by A (thawed, paid): [tx](https://explorer.solana.com/tx/2H7UJN47ZEnLCBmY6Q8YH8gqomjaDNQ45xHm242eSNYt1r9d3JZVbsXwMiVR9abCsxZvmtUnWeL62Adz7fXBTBS7?cluster=devnet)
- PASS `holder_paid_pro_rata`: A held 500 of 1000 and received 500 USDC of 1000 USDC
- PASS `double_claim`: refused (already in use)
- PASS `withdraw_bond_with_live_commitment`: refused (Error Code: LiveCommitments.)

## Step 7: a frozen holder lands in Pending, is thawed, and is released
- freeze B and C (Token ACL authority freeze): [tx](https://explorer.solana.com/tx/4r6WYDd1VfrE3qxsLCASb5ghrFK786Qu9tCzVRSSQnJUHpTswDj9gU47SsHAoahAjApyGeu1jhN3GCUdpRmr2rXW?cluster=devnet)
- claim by B (frozen, released after thaw): [tx](https://explorer.solana.com/tx/2a5qWHT54bCpXGmRfx5DPDyqAN6oXyXWKNLL4BCdvfz7xS5pg3ShpPxuTrhGcrGvSeFcP3pftEFJjSNrjk8fAXuY?cluster=devnet)
- PASS `frozen_holder_goes_to_pending`: B is frozen: 300 USDC held in Pending, nothing paid yet
- thaw B (stand-in for the gate's permissionless thaw): [tx](https://explorer.solana.com/tx/3siUfmKqU4zKjbdPGpUNuFR4TEqufZZz6YVGEGSjosEixxMJHCs4GQRVeX27Q9zbpNHdSPZviydFrgbaMGiQGtMd?cluster=devnet)
- release_pending for B (crank): [tx](https://explorer.solana.com/tx/2gXNNQcgKsRxEvL1sYjPcGc5ZAc4cq1nsmRSy3VpZbYNnsWAzbax9uUiKW9BHzx6eKJoASRdXt8oK6Gmmj8dK5oS?cluster=devnet)
- PASS `pending_released_after_thaw`: B received 300 USDC; Pending closed

## Step 8: hold window elapses, issuer sweeps, distribution closes, attestors leave
- claim by C (frozen, swept): [tx](https://explorer.solana.com/tx/4ccvRp9GGWpxEBvtxVS4CwLRQyPt5QoS48CCtK1KdEhL9mm2rQAPefLxR3rdMdYGkZn65geQpBhBFYTpp8orcz7e?cluster=devnet)
- PASS `release_while_frozen`: refused (Error Code: NotTransferable.)
- PASS `sweep_before_hold_window`: refused (Error Code: HoldWindowNotElapsed.)
- sweep_pending for C: [tx](https://explorer.solana.com/tx/21ciXtNuckSqWxbRuPXGDuo7LVWJMRUovfr9FouHwWuVbAuctYgDJ8nb74SVKHan6nX7cM1fyFbP4pwP7JBfuihq?cluster=devnet)
- PASS `unclaimed_pending_swept_to_issuer`: C stayed frozen; 200 USDC swept back to the issuer
- PASS `close_before_claim_expiry`: refused (Error Code: CannotClose.)
- close_distribution: [tx](https://explorer.solana.com/tx/5gZoqrDwTAWZgHA528xQ5FxQQ3hU67kCWZVBE1E7rYhTZBXDizvbUexT9AWv9ssAv7mceKkYLeeqLzfSbqwJMgRq?cluster=devnet)
- PASS `distribution_closed`: Closed; claimed 800 USDC + swept 200 USDC = total; vault closed
- release_attestor 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM: [tx](https://explorer.solana.com/tx/2MPTdHmAGQmAbKTkwjUviQrvSZSojABLY9xFtvvmLMAWbvTMB6cRzjmyhDXhDWGGJnSKR6FtDMQS2rtDjGe9ZJZ4?cluster=devnet)
- withdraw_bond 2YJMzBzd7XaDXiedSb8f1S7DXJ3ZVSKevXpHkMfR1xiM: [tx](https://explorer.solana.com/tx/dhj51KvhKsSV9dbbBFtBbGTPy9jioPQbixw7ybCCdfSn5WAUkrnAUCjjm85raNiuUKjgF9AvUhHQWKiZeG8ASzV?cluster=devnet)
- release_attestor BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq: [tx](https://explorer.solana.com/tx/44fdz3nhdw45DSmhVHrZMr8PerFv36EUT7j3ByF8XsTCnKfCh48p7QJf4V6PtwC4XHdsmBKJP6vUauk6mS6VtpwL?cluster=devnet)
- withdraw_bond BYgAKoYiVbLaLGgDm2sDNpBcpNcomfxQ2ZTE9z8vpmiq: [tx](https://explorer.solana.com/tx/ivnTzBVz6b413eAo6CXMWcQ5h4AazwKLFyfdAamJG1sLwxY3ftQyH3tmgqszYDgZeR8sDcH4MbthMZjCknsYDcC?cluster=devnet)
- PASS `attestors_exit_with_bonds`: attestor accounts closed, bond vault empty; each attestor ends with 14.5 USDC (lost one bond to the upheld challenge, earned 4.5 USDC from the rejected one)

32 of 32 checks passed.

## Step 9 (not on-chain)

Superbank design issue with SQL, and two issuer write-ups.

## Checks: 32 of 32 passed

- PASS `issuer_registered`: IssuerConfig for mint Cjj8zScNftRF1xLqmg3N9Qh6qWHB31JXyXNVobCcSYXt: 2 attestors, quorum 2
- PASS `attestors_bonded`: each attestor bonded 10 USDC; bond vault holds 20 USDC
- PASS `declare_rejects_short_notice`: refused (Error Code: RecordSlotTooSoon.)
- PASS `distribution_declared_and_funded`: Declared, record slot 510328664, vault holds 1000 USDC
- PASS `history_gives_expected_balances`: A, B, C hold [500, 300, 200] of the permissioned mint (expected [500, 300, 200])
- PASS `commit_before_finality_margin`: refused (Error Code: FinalityMarginNotPassed.)
- PASS `commit_rejects_unknown_spec_version`: refused (Error Code: UnsupportedSpecVersion.)
- PASS `quorum_commits_root`: Committed, register_total 800, window ends at slot 510328808
- PASS `finalize_before_window`: refused (Error Code: WindowNotElapsed.)
- PASS `claim_before_finalize`: refused (Error Code: InvalidState.)
- PASS `root_reproduced_independently`: the Python spec implementation recomputes the on-chain root
- PASS `challenge_opens_dispute`: Disputed; challenger bond 10 USDC locked
- PASS `second_challenge_while_disputed`: refused (Error Code: InvalidState.)
- PASS `resolve_by_non_resolver`: refused (Error Code: NotResolver.)
- PASS `rejected_challenge_pays_resolver_and_attestors`: Committed; resolver fee 1 USDC, each attestor's bond 14.5 USDC (+4.5 USDC), challenge account closed
- PASS `upheld_challenge_slashes_attestors`: Declared, commitments closed; challenger holds 28 USDC (bond back + two slashes less fees), resolver 3 USDC, attestor bonds 4.5 USDC
- PASS `corrected_root_committed_and_reproduced`: Committed, register_total 1000, root reproduced by the Python spec code
- PASS `challenge_after_window`: refused (Error Code: WindowClosed.)
- PASS `finalize_opens_claims`: Open
- PASS `claim_with_inflated_balance`: refused (Error Code: InvalidProof.)
- PASS `claim_with_someone_elses_proof`: refused (Error Code: InvalidProof.)
- PASS `holder_paid_pro_rata`: A held 500 of 1000 and received 500 USDC of 1000 USDC
- PASS `double_claim`: refused (already in use)
- PASS `withdraw_bond_with_live_commitment`: refused (Error Code: LiveCommitments.)
- PASS `frozen_holder_goes_to_pending`: B is frozen: 300 USDC held in Pending, nothing paid yet
- PASS `pending_released_after_thaw`: B received 300 USDC; Pending closed
- PASS `release_while_frozen`: refused (Error Code: NotTransferable.)
- PASS `sweep_before_hold_window`: refused (Error Code: HoldWindowNotElapsed.)
- PASS `unclaimed_pending_swept_to_issuer`: C stayed frozen; 200 USDC swept back to the issuer
- PASS `close_before_claim_expiry`: refused (Error Code: CannotClose.)
- PASS `distribution_closed`: Closed; claimed 800 USDC + swept 200 USDC = total; vault closed
- PASS `attestors_exit_with_bonds`: attestor accounts closed, bond vault empty; each attestor ends with 14.5 USDC (lost one bond to the upheld challenge, earned 4.5 USDC from the rejected one)
