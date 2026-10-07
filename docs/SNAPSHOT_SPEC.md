# Slotbook Snapshot Spec v1.0

`spec_version = 100` (u16, v1.00)

This spec defines, byte for byte, how the holder register of a permissioned
token at a record slot is built and committed as a Merkle root. Two attestors
following it from the same ledger must produce the same root. A resolver
settles a challenge by recomputing one owner under it. Anyone can check a
commitment with the published test vectors.

Sections 3, 5, 6, 7, 8, 10 and 11 are normative. Section 4 is a reference
method, not a requirement.

## 1. Scope

In scope: which accounts count, how balances are read at the record slot, the
salt, the leaf, the tree, the proof, the payout rule, what a challenge means,
and the manifest.

Out of scope: how an attestor fetches data, the proofs API, and on-chain
account layouts (see `docs/STATE_MACHINE.md`).

## 2. Notation

- `||` is byte concatenation.
- `keccak256` is the original Keccak-256 (Ethereum style). It is **not**
  NIST SHA3-256; the padding differs and so do the outputs. Python's
  `hashlib.sha3_256` is SHA3-256 and must not be used.
- `u16_le`, `u32_le`, `u64_le` are little-endian fixed-width integers.
- Addresses (mint, owner, distribution) are their 32 raw bytes, not base58.
- Hex in this document and in the vectors is lowercase with no `0x` prefix.

## 3. The register

### 3.1 Inputs

| Input | Source |
|---|---|
| `mint` | `IssuerConfig.mint` (Token-2022) |
| `record_slot` | `Distribution.record_slot` |
| `distribution` | the Distribution PDA address |
| `salt_seed` | 32 bytes from the issuer; `keccak256(salt_seed)` equals `Distribution.salt_seed_hash` |

### 3.2 Mint precondition

The mint must have had `DefaultAccountState = Frozen` from its creation
transaction (scope D15). Every funded account then had to be thawed first, and
thaw references the mint, which makes discovery (section 4) complete. An
attestor must refuse any other mint.

### 3.3 State at the record slot

The register is read from the state of the ledger **after every successful
transaction in slots `<= record_slot`**, at finalized commitment. Failed
transactions are ignored. If `record_slot` was skipped (no block), the state is
the state after the last block before it.

### 3.4 Rows

1. Take every token account whose mint is `mint` and that exists in that state.
   Closed accounts do not exist and are not included.
2. For each, read its `owner` field (the wallet; not a delegate and not the
   close authority) and its raw `amount`.
3. Group by owner and sum the amounts. An owner holding several token accounts
   gets one row.
4. Drop rows whose sum is 0.
5. Frozen accounts are included like any other. Entitlement does not depend on
   freeze state.
6. The all-zero address (`Pubkey::default()`) is reserved: a challenge naming
   it disputes `register_total` (scope D22). If any token account has the
   all-zero owner, the attestor must refuse.

### 3.5 Order

Rows are sorted by `owner` bytes, ascending (lexicographic over the 32 bytes).
Leaf `i` of the tree is row `i`.

### 3.6 `register_total`

`register_total` is the sum of all row balances, as u64. It cannot overflow,
because it cannot exceed the mint's supply.

### 3.7 Empty register

If there are no rows, there is no tree and no root. Attestors must not commit.
The issuer can close the distribution.

### 3.8 Known limitation (v1)

Every holder is included, including any account the issuer holds itself (for
example a treasury). v1 has no exclusion list. An issuer that wants a treasury
excluded must move those tokens out before the record slot.

## 4. Reference method: ledger replay (informative)

This is how the reference attestor reaches the state in 3.3. Other methods
(for example Superbank SQL) are valid if they reach the same rows.

1. **Discovery** (scope D15). Fetch every transaction that references `mint`.
   Every token account named in these instructions for `mint` is discovered:
   `initialize_account`, `initialize_account2`, `initialize_account3`,
   `mint_to`, `mint_to_checked`, `transfer_checked`, `burn`, `burn_checked`,
   `freeze_account`, `thaw_account` (including Token ACL's thaw).
2. Fetch every transaction that references a discovered account. `close_account`
   and plain `transfer` do not name the mint and are caught this way.
3. Order transactions by `(slot, position in block)`. Position is the index in
   `getBlock(slot)` (with `transactionDetails: "signatures"`, the `signatures`
   array). Never order by fetch order.
4. For each successful transaction, set each discovered account's owner and
   amount from its post-transaction token balance. An account present before the
   transaction and absent after it was closed.
5. Stop after the last transaction in a slot `<= record_slot`, then apply 3.4.

## 5. Salt

```
salt = keccak256( "slotbook-salt-v1" || distribution || owner || salt_seed )
```

- `"slotbook-salt-v1"` is the 16 ASCII bytes, no terminator.
- `distribution`, `owner`, `salt_seed` are 32 bytes each. Total input: 112 bytes.
- `distribution` binds every leaf to one distribution, so a proof can never be
  reused in another.
- Salts are derived, not random, so independent attestors produce the same root
  (scope D12). Only the issuer, attestors and resolver know `salt_seed`.
- The salt stops anyone without the seed from testing guesses against the root
  or proofs. It is not strong privacy: balances at a slot are public on-chain.

## 6. Leaf

```
inner = keccak256( owner || u64_le(balance) || salt )     # 72 bytes in
leaf  = keccak256( 0x00 || inner )                        # 33 bytes in
```

- `balance` is the raw row balance at the record slot (3.4), not the payout.
- The `0x00` prefix and the 33-byte input keep leaf hashes apart from internal
  node hashes (64-byte input). A leaf can never be presented as a node, or the
  reverse.

## 7. Tree

```
parent(a, b) = keccak256( min(a, b) || max(a, b) )     # byte-wise comparison
```

- Level 0 is the leaves in row order (3.5).
- Each next level pairs nodes `(0,1), (2,3), ...`. If a level has an odd number
  of nodes, the last one is **promoted unchanged** to the next level (not
  hashed, not duplicated).
- The root is the single node left. With one row, the root is that row's leaf.
- Depth is at most 32 (at most 2^32 rows).

## 8. Proof and verification

A proof is the **list of sibling hashes, bottom-up**, with nothing else: no
index, no leaf count, no left/right flags. At a level where the node was
promoted, there is no sibling and nothing is added. A proof has at most 32
entries.

```
h = leaf
for s in proof: h = parent(h, s)
valid  <=>  h == root  and  len(proof) <= 32
```

This is safe because `claim` computes the leaf itself from the signer (as
`owner`), `balance` and `salt` (section 6), so a caller cannot submit a node as
a leaf, and `ClaimReceipt ["claim", distribution, owner]` allows one claim per
owner.

## 9. Payout

```
payout = floor( total * balance / register_total )     # u128 intermediates
```

`total` is `Distribution.total`. Computed on-chain in `claim` (scope D20).
Rounding dust stays in the vault and returns to the issuer at close.

## 10. Challenges

- A challenge names an `owner` and a `claimed_balance`. The resolver recomputes
  that owner's balance under this spec.
- An owner with no row in the register has balance **0** under this spec. A
  challenge that an existing holder was left out names that owner and their
  true balance.
- A challenge naming the all-zero owner disputes `register_total`; the resolver
  recomputes the total (scope D22).
- How a ruling moves bonds is defined by the program (scope D3 to D5, D17), not
  by this spec.

## 11. Manifest

The attestor publishes a manifest with each commitment and stores
`manifest_hash` in `Commitment.manifest_hash`.

| # | Field | Encoding | Meaning |
|---|---|---|---|
| 1 | `spec_version` | `u16_le` | 100 for this spec |
| 2 | `distribution` | 32 bytes | Distribution PDA |
| 3 | `mint` | 32 bytes | the mint |
| 4 | `record_slot` | `u64_le` | from the Distribution |
| 5 | `first_covered_slot` | `u64_le` | slot of the mint's creation transaction; replay must start there |
| 6 | `resolved_slot` | `u64_le` | finalized slot at which the ledger was read; `>= record_slot + finality_margin_slots` |
| 7 | `backend` | `u8` | 0 = LedgerReplay, 1 = StateArchive |
| 8 | `rows` | `u32_le` | number of rows |
| 9 | `register_total` | `u64_le` | section 3.6 |
| 10 | `root` | 32 bytes | section 7 |
| 11 | `salt_seed_hash` | 32 bytes | `keccak256(salt_seed)` |

```
manifest_hash = keccak256( "slotbook-manifest-v1" || field 1 || field 2 || ... || field 11 )
```

`"slotbook-manifest-v1"` is 20 ASCII bytes. Fields are concatenated in table
order with the encodings above (167 bytes after the tag).

## 12. Versioning

Any change to sections 3 and 5 to 11 bumps `spec_version` and changes the tags
(`slotbook-salt-v2`, `slotbook-manifest-v2`), so hashes from different
versions can never collide. Every commitment carries the version it used.

## 13. Test vectors

- `docs/vectors/snapshot_v1.json`: the vectors.
- `docs/vectors/gen_snapshot_vectors.py`: an independent implementation in pure
  Python, including its own Keccak-256, sharing no code with the Rust crates.
  Run `python3 docs/vectors/gen_snapshot_vectors.py` to regenerate.
- `crates/snapshot/tests/spec_vectors.rs`: checks the Rust crates against them.

The file contains Keccak known answers, two leaf vectors (one with balance 1,
which shows the little-endian encoding), three trees with every proof, and one
manifest. Key values:

| Vector | Value |
|---|---|
| `brief_example` leaf (owner `01..`, balance 1000) | `195d1aebba10987bc94c9d9a5b19cbf1f3fb1ce0c67ec054aaaa12bce3cd2718` |
| `one_holder_max_balance` root (balance u64::MAX) | `b781906d2f9f2a74f3c294cbb7b28a2be7ed4043ab9bea5f8c74eb42c181c5b3` |
| `two_holders` root | `62cdc8200fd0848b6a8e3572082b6ad3c1d475dbf6b5609a646b311de9688b9a` |
| `five_holders_sum_zero_frozen` root | `4a9e9afb82054ba45809201c72b90d8b823aa05447bd4253bf425b6546b77a8d` |
| manifest of the five-holder tree | `f7b8776949c672c54a544d49b706e3ffb9442a59bf46aca46d899d642a4afe6c` |

The five-holder tree covers summing (one owner, two accounts), a zero balance
(dropped), a frozen holder (included), unsorted input, and odd-node promotion
at two levels.
