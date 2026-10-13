# Superbank proposal: holders of a mint at a slot

Status: proposed; design issue open upstream: [solana-rpc/superbank#110](https://github.com/solana-rpc/superbank/issues/110). Target: [solana-rpc/superbank](https://github.com/solana-rpc/superbank) (Triton One,
AGPL-3.0). Researched against v0.7.0 (2026-09-24) and main at `e49d4d7` (2026-10-07); SQL tested
against v0.8.0 (2026-10-08).
Spec: Slotbook Snapshot Spec v1.0 (`docs/SNAPSHOT_SPEC.md`), sections 3 and 4.

## Summary

Add one read-only method to `superbank-rpc` that returns the holder register of an SPL token
(Token or Token-2022) at a past slot: every owner and their raw balance, summed across their token
accounts, as of the end of that slot at finalized commitment. The rows match Slotbook spec section 3
exactly, so an attestor can build a Merkle root from them with no further processing.

Superbank already stores everything needed: per-transaction pre and post token balances with mint,
owner and raw amount, the position of each transaction in its block, and success status. The method
is one query over `default.transactions` plus a short handler. No new ingestion is needed.

## Problem

Permissioned SPL tokens (tokenized funds, bonds, equities) pay out to whoever held them at a
**record date**. On Solana the record date is a slot. Today nobody can answer "who held mint M at
slot S" from one call. Current-state APIs answer "now", not "then".

Slotbook's reference attestor rebuilds the register by replaying the ledger over standard RPC
(spec section 4). That works but it is slow and fragile:

- **Many calls.** `getSignaturesForAddress` on the mint, then on every discovered token account,
  then one `getTransaction` per signature, then `getBlock` per slot. Call count grows with every
  transfer of the mint.
- **Ordering.** Standard `getTransaction` does not return the transaction's position in its block.
  The attestor must call `getBlock(slot)` for every slot touched, just to order transactions inside
  a slot. Fetch order is not ledger order.
- **Plain transfers and closes do not name the mint.** A plain `transfer` lists source,
  destination and authority. `close_account` lists the account, destination and owner. Neither
  shows up in `getSignaturesForAddress(mint)`. The attestor needs a second round per token account,
  and the method is complete only for mints that were default-frozen from creation (spec rule D15).
- **Closed and reopened accounts.** A closed account disappears from post balances; a reopened one
  restarts at zero. Getting this right needs exact `(slot, position)` order across all of the above.

Superbank holds the same data in ClickHouse, already ordered by `(slot, slot_idx)`, with failed
transactions flagged and the mint present in every token balance entry. The token balance metadata
carries the mint even for plain transfers and closes, so a query by mint does not need the D15
discovery trick at all.

## Proposed method

Name: `getTokenHoldersAtSlot` (open to the maintainers' naming).

```json
{
  "jsonrpc": "2.0", "id": 1,
  "method": "getTokenHoldersAtSlot",
  "params": [
    "<mint base58>",
    {
      "slot": 412345678,
      "commitment": "finalized",
      "minSlot": 401000000,
      "limit": 10000,
      "paginationToken": null
    }
  ]
}
```

| Param | Required | Meaning |
|---|---|---|
| `mint` | yes | SPL Token or Token-2022 mint address |
| `slot` | yes | Record slot S. State after every successful transaction in slots `<= S`. |
| `commitment` | no | Only `finalized` is accepted. Anything else returns `-32602`. |
| `minSlot` | no | Start of the scan. Default and maximum: the first slot where the mint appears in `gsfa`. A later value is refused, because holders whose last activity is before it would be missing. |
| `limit` | no | Max rows per page (default and cap set by the operator). |
| `paginationToken` | no | Opaque cursor (the last owner returned). |

Response:

```json
{
  "context": { "slot": 412399999 },
  "value": {
    "mint": "<base58>",
    "slot": 412345678,
    "resolvedSlot": 412345678,
    "minSlot": 401000000,
    "holders": [
      { "owner": "<base58>", "amount": "1000" }
    ],
    "holderCount": 1,
    "total": "1000",
    "paginationToken": null
  }
}
```

- `holders` is sorted by owner bytes ascending (spec 3.5). `amount` is a decimal string of the raw
  u64 (spec 3.4).
- `resolvedSlot` is S if a block was produced in S, else the first produced slot after S (spec 3.3).
- `holderCount` and `total` cover the whole register, not just the page. `total` is the spec's
  `register_total`.
- Errors: slot not yet covered by finalized data; a `minSlot` after the mint's first slot; a token
  balance with no owner; an all-zero owner;
  a transaction in range with missing metadata.

## SQL against Superbank's schema

Written against `ddl/local/transactions.sql`, `ddl/local/blocks_metadata.sql` and `ddl/local/gsfa.sql`
(v0.8.0 only added nullable columns to `blocks_metadata`; the columns used here are unchanged). It follows the patterns of Superbank's own `token_owner_activity`
MV and `token_transfers` view.

**Tested** in chdb (embedded ClickHouse 26.9) against Superbank v0.8.0's `transactions`,
`blocks_metadata` and `gsfa` DDL: `scripts/superbank/test_holders_at_slot.py` runs Query 1 and
Query 2 exactly as written below. The expected register comes from a ledger simulator, not from
the token-balance rows. Cases: two transfers in one slot, two accounts with one owner, a failed
transaction, an account closed in the slot it was emptied, an address closed and reused for another
mint (and the reverse), a zero balance, a `SetAuthority` owner change, an account only in v0 loaded
addresses, a sum above `u64::MAX`, a transaction after S; and the guards for a late `minSlot`, a
transaction without token metadata, and a NULL owner. Not tested: real mainnet data, performance.

Assumptions about real data, still to confirm:

- A1. Every transaction that changes a token account lists that account in its pre and post token
  balances, with the mint. (This is how Agave collects token balances. To confirm.)
- A2. `default.transactions` holds finalized data only, and every block in `[minSlot, resolvedSlot]`
  is ingested.
- A3. `meta_*_token_amount` is the raw `amount` of the token account (base units, withheld
  transfer fees excluded, never scaled by interest or UI multipliers).

### Query 1: the register

```sql
-- Holders of {mint} at {record_slot}. Slotbook snapshot spec v1.0, section 3.
SELECT
    owner,
    sum(toUInt128(amount)) AS balance
FROM
(
    -- Last state of every token account that held the mint, as of record_slot.
    SELECT
        token_account,
        argMax(tuple(post_is_m, post_owner, post_amount), tuple(slot, slot_idx)) AS last,
        last.1 AS exists_after,
        last.2 AS owner,
        last.3 AS amount
    FROM
    (
        WITH
            CAST(base58Decode({mint:String}) AS FixedString(32)) AS m,
            arrayConcat(tx_account_keys,
                        meta_loaded_addresses_writable,
                        meta_loaded_addresses_readonly) AS all_keys
        SELECT
            slot,
            slot_idx,
            all_keys[toUInt16(idx) + 1]                   AS token_account,
            indexOf(meta_pre_token_account_index, idx)    AS pre_pos,
            indexOf(meta_post_token_account_index, idx)   AS post_pos,
            pre_pos  > 0 AND meta_pre_token_mint[pre_pos]   = m AS pre_is_m,
            post_pos > 0 AND meta_post_token_mint[post_pos] = m AS post_is_m,
            if(post_is_m, meta_post_token_owner[post_pos], NULL) AS post_owner,
            toUInt64(if(post_is_m, meta_post_token_amount[post_pos], '0')) AS post_amount
        FROM default.transactions
        ARRAY JOIN arrayDistinct(arrayConcat(meta_pre_token_account_index,
                                             meta_post_token_account_index)) AS idx
        WHERE slot >= {min_slot:UInt64}
          AND slot <= {record_slot:UInt64}
          AND meta_status_ok = 1
          AND (has(meta_pre_token_mint, m) OR has(meta_post_token_mint, m))
          AND (pre_is_m OR post_is_m)
    )
    GROUP BY token_account
    HAVING exists_after
)
GROUP BY owner
HAVING balance > 0
ORDER BY owner ASC;
```

Notes:

- `argMax(..., tuple(slot, slot_idx))` picks each account's last change in ledger order. Duplicate
  rows from `ReplacingMergeTree` carry identical values, so they do not change the result; no
  `FINAL` is needed.
- An account closed, or reopened under another mint, has a last entry with `post_is_m = 0` and
  drops out.
- `sum(toUInt128(...))` avoids a silent u64 wrap. The handler checks `total <= u64::MAX`.
- `ORDER BY owner` on `FixedString(32)` is byte order, which is spec 3.5.
- Pagination: add `AND owner > {after:FixedString(32)}` and `LIMIT {limit}` in the outer query.

### Query 2: guards (run before or with Query 1)

```sql
WITH CAST(base58Decode({mint:String}) AS FixedString(32)) AS m
SELECT
    -- resolved slot (spec 3.3): first produced block at or after record_slot
    (SELECT min(slot) FROM default.blocks_metadata
      WHERE slot >= {record_slot:UInt64})                        AS resolved_slot,
    -- default minSlot: first transaction that names the mint
    (SELECT min(slot) FROM default.gsfa WHERE address = m)        AS first_mint_slot,
    -- rows the register cannot trust
    (SELECT count() FROM default.transactions
      WHERE slot BETWEEN {min_slot:UInt64} AND {record_slot:UInt64}
        AND (meta_pre_token_balances_present = 0
             OR meta_post_token_balances_present = 0))            AS rows_missing_meta;
```

The handler refuses, rather than answers, when `minSlot > first_mint_slot` (holders active only
before `minSlot` would be missing), when `resolved_slot` is NULL or above the latest finalized
ingested slot, when `rows_missing_meta > 0`, when any Query 1 owner is NULL, or when an owner is the
all-zero address. Excluding such rows instead would return a wrong register without saying so.

### Performance

Query 1 scans every transaction in `[minSlot, slot]`. That is fine for young mints and short
ranges, and too slow for years of mainnet. Two options for the maintainers:

- **B1. Skip index.** Add `INDEX bf_post_mint meta_post_token_mint TYPE bloom_filter(0.01)` and the
  same on `meta_pre_token_mint`. Smallest change; ClickHouse skips granules without the mint.
- **B2. Mint-keyed MV** `token_mint_activity`, one row per (mint, token account, transaction),
  `ORDER BY (mint, token_account, slot, slot_idx, signature)`, storing `status_ok`, `post_is_mint`,
  `owner`, `amount`. Built like `token_owner_activity`. Query 1 then reads one key range. It needs
  DDL in all three sets (`local`, `cluster`, `replicated`) and a backfill.

I would start with the plain scan from the mint's first slot, and let the maintainers pick B1 or B2.
This is about speed only: the start of the scan is fixed by the guard, not a tuning knob.

## Correctness mapping

| Spec rule | How the SQL or handler enforces it |
|---|---|
| 3.1 Inputs `mint`, `record_slot` | Method params `mint`, `slot`. `distribution` and `salt_seed` stay with Slotbook. |
| 3.2 Mint must be default-frozen (D15) | Not enforced by the method. The query does not need D15 to be complete (A1). Slotbook's attestor still checks it. |
| 3.3 State after successful txs in slots `<= record_slot` | `slot <= {record_slot}` and `meta_status_ok = 1`; last entry per account by `argMax`. |
| 3.3 Finalized commitment | Only `finalized` accepted; ClickHouse holds finalized data (A2). |
| 3.3 Order inside a slot is block position | `tuple(slot, slot_idx)`; `slot_idx` is the index in `getBlock` transactions. |
| 3.3 Failed transactions ignored | `meta_status_ok = 1`; missing-meta rows refused (Query 2). |
| 3.3 `resolved_slot` for a skipped slot | `min(slot)` in `blocks_metadata` at or after S. The register itself is unchanged by a skip, since no transactions exist in skipped slots. |
| 3.4.1 Token accounts of `mint` that exist; closed excluded | Last entry must have a post balance with mint M (`exists_after`). Closed accounts have no post entry. |
| 3.4.2 `owner` field, not delegate or close authority | `meta_post_token_owner` is the account's owner field. |
| 3.4.2 Raw amount; no UI amount; withheld fees excluded | `meta_post_token_amount` cast to u64; `ui_*` columns unused (A3). |
| 3.4.2 Closed and reopened restarts at zero | The reopened account's post balance is its real amount; its pre-close history is superseded by `argMax`. |
| 3.4.3 Group by owner and sum | Outer `GROUP BY owner`, `sum(toUInt128(amount))`. |
| 3.4.4 Drop zero rows | `HAVING balance > 0`. |
| 3.4.5 Frozen accounts included | No filter on freeze state (not in token balances anyway). |
| 3.4.6 All-zero owner refused | Handler check on the result. |
| 3.5 Sort by owner bytes | `ORDER BY owner ASC` on `FixedString(32)`. |
| 3.6 `register_total` u64 | `total` in the response; handler checks it fits u64. |
| 3.7 Empty register | Empty `holders`, `total = "0"`. Slotbook decides not to commit. |
| 3.8 No exclusion list | None in the method either. |

## Verification

1. **Root check.** Feed the method's `holders` into Slotbook's `scripts/reproduce_root.py`
   (input `{"distribution", "salt_seed", "rows": [{"owner", "balance"}]}`) and compare the root with
   the one the attestor committed for the same distribution. The script uses the independent pure
   Python implementation in `docs/vectors/gen_snapshot_vectors.py`.
2. **Against the RPC replay.** Run the Slotbook attestor's RPC replay
   (`crates/attestor`, `fetch_history` then `replay`) and the method on the same devnet mint and
   record slot. The registers must match row for row.
3. **Spec vectors.** `docs/vectors/snapshot_v1.json` starts from token accounts, so it checks the
   tree and leaf rules, not the query. For the query, add ClickHouse fixture rows to Superbank's
   test suite for the replay cases listed in spec 13.1: a closed account, an account reopened at
   the same address, a failed transaction, a skipped record slot, an empty register, plus two
   accounts with one owner and two transactions in the same slot in reverse fetch order.
4. **k6.** A basic scenario as Superbank's CONTRIBUTING asks for new RPC behavior.

## Scope and non-goals

In scope: one method, one query, guards, docs, tests. Optional skip index or MV if the
maintainers want it.

Not in scope:

- Salts, leaves, Merkle roots, manifests. These stay in Slotbook.
- Enforcing Slotbook's D15 mint precondition or any Slotbook-specific rule.
- UI amounts, decimals scaling, or freeze state in the response.
- `confirmed` or `processed` commitment.
- Account state snapshots (lamports, data). Only token balances from transaction metadata.

## Next steps

1. Design issue opened: [solana-rpc/superbank#110](https://github.com/solana-rpc/superbank/issues/110).
   Do not write code before a maintainer replies on shape and indexing.
2. After review, send the method as a tested PR (Conventional Commit title, tests, k6 results,
   docs, DDL in all three sets if an index is added).
3. Slotbook never depends on the merge. The attestor keeps its RPC replay as the default backend.
   The Superbank method is a second, faster backend and an independent cross-check.

Questions for the maintainers:

1. Performance only (the start slot is fixed by the guard): is a plain range scan from the mint's
   first slot acceptable for a first version, or do you want a skip index or a mint-keyed MV?
2. How should the method prove every block in the range is ingested?
3. Is the name, params and response shape (owner cursor pagination, string amounts) in line with
   `getTransactionsForAddress`?
