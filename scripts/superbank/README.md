# Superbank holders-at-slot test

Tests the SQL in [docs/SUPERBANK_PROPOSAL.md](../../docs/SUPERBANK_PROPOSAL.md) (proposed upstream in
[solana-rpc/superbank#110](https://github.com/solana-rpc/superbank/issues/110)) against Superbank's
real schema, without running Superbank.

- **Engine:** [chdb](https://github.com/chdb-io/chdb), ClickHouse embedded in Python.
- **Schema:** Superbank's own `ddl/local/{transactions,blocks_metadata,gsfa}.sql`, downloaded at
  run time from v0.8.0 (commit `5241078`). Nothing from Superbank is stored in this repo.
- **SQL:** Query 1 (register) and Query 2 (guards) are read from the proposal doc, so the doc and
  the test cannot drift apart.
- **Expected answer:** a small ledger simulator applies token operations and records the pre/post
  token balances a validator would store. The expected register is the simulator's true state at
  the record slot, not something derived from the rows the SQL reads.

## Run

    python3 -m venv .venv
    .venv/bin/pip install chdb
    .venv/bin/python scripts/superbank/test_holders_at_slot.py

It ends with `6 of 6 checks passed` and exits non-zero on any failure.

## What it covers

| Check | Cases |
|---|---|
| Register equals the true state at the record slot, in owner byte order | two transfers in one slot (ordered by `slot_idx`), two accounts with one owner, a failed transaction, an account closed in the slot it was emptied, an address closed and reused for another mint and the reverse, a zero balance, a `SetAuthority` owner change, an account only in v0 loaded addresses, a sum above `u64::MAX`, a transaction after the record slot, rows inserted in random order |
| Guards on clean data | `first_mint_slot`, `resolved_slot`, `rows_missing_meta = 0` |
| A `minSlot` after the mint's first slot | the register silently misses a holder, and the guard exposes it |
| Missing token metadata | the guard counts the transaction |
| NULL owner | comes back as its own row, so the handler must refuse |

Not covered: real mainnet data and performance.
