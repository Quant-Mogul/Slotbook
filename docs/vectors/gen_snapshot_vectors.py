#!/usr/bin/env python3
"""Independent reference implementation of docs/SNAPSHOT_SPEC.md v1.0.

Generates docs/vectors/snapshot_v1.json. Pure Python 3, no dependencies, and
no code shared with the Rust crates, so the vectors are an independent check.

    python3 docs/vectors/gen_snapshot_vectors.py

Note: Python's hashlib.sha3_256 is NOT keccak256 (different padding).
Keccak-256 is implemented below and checked against known answers first.
"""

import json
import os

# ---------------------------------------------------------------- keccak256

_RC = [
    0x0000000000000001, 0x0000000000008082, 0x800000000000808A, 0x8000000080008000,
    0x000000000000808B, 0x0000000080000001, 0x8000000080008081, 0x8000000000008009,
    0x000000000000008A, 0x0000000000000088, 0x0000000080008009, 0x000000008000000A,
    0x000000008000808B, 0x800000000000008B, 0x8000000000008089, 0x8000000000008003,
    0x8000000000008002, 0x8000000000000080, 0x000000000000800A, 0x800000008000000A,
    0x8000000080008081, 0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
]
_ROT = [
    [0, 36, 3, 41, 18],
    [1, 44, 10, 45, 2],
    [62, 6, 43, 15, 61],
    [28, 55, 25, 21, 56],
    [27, 20, 39, 8, 14],
]
_M = (1 << 64) - 1


def _rol(v, n):
    return ((v << n) | (v >> (64 - n))) & _M if n else v


def _keccak_f(a):
    for rc in _RC:
        c = [a[x][0] ^ a[x][1] ^ a[x][2] ^ a[x][3] ^ a[x][4] for x in range(5)]
        d = [c[(x - 1) % 5] ^ _rol(c[(x + 1) % 5], 1) for x in range(5)]
        a = [[a[x][y] ^ d[x] for y in range(5)] for x in range(5)]
        b = [[0] * 5 for _ in range(5)]
        for x in range(5):
            for y in range(5):
                b[y][(2 * x + 3 * y) % 5] = _rol(a[x][y], _ROT[x][y])
        a = [[b[x][y] ^ ((~b[(x + 1) % 5][y]) & b[(x + 2) % 5][y]) for y in range(5)]
             for x in range(5)]
        a[0][0] ^= rc
    return a


def keccak256(data: bytes) -> bytes:
    rate = 136
    msg = bytearray(data) + b"\x01"  # Keccak padding (SHA3 would use 0x06)
    while len(msg) % rate:
        msg.append(0)
    msg[-1] |= 0x80
    a = [[0] * 5 for _ in range(5)]
    for off in range(0, len(msg), rate):
        block = msg[off:off + rate]
        for i in range(rate // 8):
            x, y = i % 5, i // 5
            a[x][y] ^= int.from_bytes(block[8 * i:8 * i + 8], "little")
        a = _keccak_f(a)
    out = b""
    for i in range(4):
        x, y = i % 5, i // 5
        out += a[x][y].to_bytes(8, "little")
    return out


assert keccak256(b"").hex() == "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
assert keccak256(b"abc").hex() == "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"

# ------------------------------------------------------------------ base58

_B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58(b: bytes) -> str:
    n = int.from_bytes(b, "big")
    s = ""
    while n:
        n, r = divmod(n, 58)
        s = _B58[r] + s
    pad = len(b) - len(b.lstrip(b"\x00"))
    return "1" * pad + s


# ------------------------------------------------------------ spec, v1.0

SALT_TAG = b"slotbook-salt-v1"
MANIFEST_TAG = b"slotbook-manifest-v1"
SPEC_VERSION = 100  # u16, v1.00
ZERO = bytes(32)


def u64(n):
    return n.to_bytes(8, "little")


def salt(distribution, owner, salt_seed):
    return keccak256(SALT_TAG + distribution + owner + salt_seed)


def leaf(owner, balance, s):
    return keccak256(b"\x00" + keccak256(owner + u64(balance) + s))


def parent(a, b):
    return keccak256(a + b) if a <= b else keccak256(b + a)


def register(accounts):
    """accounts: list of dicts {account, owner, amount, frozen}. Spec section 3."""
    summed = {}
    for acc in accounts:
        assert acc["owner"] != ZERO, "all-zero owner is reserved (spec 3.4)"
        summed[acc["owner"]] = summed.get(acc["owner"], 0) + acc["amount"]
    rows = sorted((o, bal) for o, bal in summed.items() if bal > 0)
    total = sum(bal for _, bal in rows)
    assert rows, "empty register: nothing to commit (spec 3.7)"
    assert total < 2 ** 64
    return rows, total


def levels(leaves):
    out = [leaves]
    while len(out[-1]) > 1:
        cur, nxt = out[-1], []
        for i in range(0, len(cur) - 1, 2):
            nxt.append(parent(cur[i], cur[i + 1]))
        if len(cur) % 2:
            nxt.append(cur[-1])  # odd node promoted unhashed
        out.append(nxt)
    return out


def proof(lvls, index):
    sibs = []
    for lvl in lvls[:-1]:
        if not (len(lvl) % 2 == 1 and index == len(lvl) - 1):
            sibs.append(lvl[index ^ 1])
        index //= 2
    assert len(sibs) <= 32
    return sibs


def verify(lf, sibs, root):
    h = lf
    for s in sibs:
        h = parent(h, s)
    return h == root


def manifest_hash(m):
    return keccak256(
        MANIFEST_TAG
        + m["spec_version"].to_bytes(2, "little")
        + m["distribution"]
        + m["mint"]
        + u64(m["record_slot"])
        + u64(m["first_covered_slot"])
        + u64(m["resolved_slot"])
        + bytes([m["backend"]])
        + m["rows"].to_bytes(4, "little")
        + u64(m["register_total"])
        + m["root"]
        + m["salt_seed_hash"]
    )


# ----------------------------------------------------------------- inputs

DISTRIBUTION = bytes([0xD1]) * 32
SALT_SEED = bytes(range(32))
MINT = bytes([0x4D]) * 32


def key(b):
    return bytes([b]) * 32


def H(b):
    return b.hex()


def tree_vector(name, note, accounts):
    rows, total = register(accounts)
    salts = [salt(DISTRIBUTION, o, SALT_SEED) for o, _ in rows]
    leaves = [leaf(o, bal, s) for (o, bal), s in zip(rows, salts)]
    lvls = levels(leaves)
    root = lvls[-1][0]
    out_rows = []
    for i, ((o, bal), s, lf) in enumerate(zip(rows, salts, leaves)):
        sibs = proof(lvls, i)
        assert verify(lf, sibs, root)
        out_rows.append({
            "owner": H(o), "owner_b58": b58(o), "balance": bal,
            "salt": H(s), "leaf": H(lf), "proof": [H(x) for x in sibs],
        })
    return {
        "name": name,
        "note": note,
        "accounts": [
            {"account": H(a["account"]), "owner": H(a["owner"]),
             "amount": a["amount"], "frozen": a["frozen"]}
            for a in accounts
        ],
        "rows": out_rows,
        "register_total": total,
        "root": H(root),
    }, root, total, len(rows)


def acct(a, o, amount, frozen=False):
    return {"account": key(a), "owner": key(o), "amount": amount, "frozen": frozen}


MAX = 2 ** 64 - 1

t1, _, _, _ = tree_vector(
    "one_holder_max_balance",
    "One row: root = leaf, proof is empty. Balance is u64::MAX.",
    [acct(0xA1, 0x11, MAX)],
)
t2, _, _, _ = tree_vector(
    "two_holders",
    "Two rows, given in reverse owner order; rows are sorted by owner bytes.",
    [acct(0xA1, 0xC3, 250), acct(0xA2, 0x22, 750)],
)
t5, root5, total5, rows5 = tree_vector(
    "five_holders_sum_zero_frozen",
    "Seven accounts -> five rows. Owner 0x11.. holds two accounts (summed). "
    "Owner 0x99.. has balance 0 (dropped). Owner 0x5a.. is frozen (included). "
    "Five leaves: odd node promoted at two levels.",
    [
        acct(0xA1, 0xE9, 40),
        acct(0xA2, 0x11, 100),
        acct(0xA3, 0x5A, 7, frozen=True),
        acct(0xA4, 0x99, 0),
        acct(0xA5, 0x07, 1),
        acct(0xA6, 0x11, 23),
        acct(0xA7, 0xAA, 1_000_000),
    ],
)

manifest = {
    "spec_version": SPEC_VERSION,
    "distribution": DISTRIBUTION,
    "mint": MINT,
    "record_slot": 1_000_000,
    "first_covered_slot": 999_000,
    "resolved_slot": 1_000_040,
    "backend": 0,
    "rows": rows5,
    "register_total": total5,
    "root": root5,
    "salt_seed_hash": keccak256(SALT_SEED),
}

vectors = {
    "spec": "docs/SNAPSHOT_SPEC.md",
    "spec_version": SPEC_VERSION,
    "generator": "docs/vectors/gen_snapshot_vectors.py",
    "encoding": "all byte strings are lowercase hex, no 0x prefix",
    "keccak256_known_answers": [
        {"input_utf8": "", "hash": keccak256(b"").hex()},
        {"input_utf8": "abc", "hash": keccak256(b"abc").hex()},
    ],
    "common": {
        "distribution": H(DISTRIBUTION),
        "salt_seed": H(SALT_SEED),
        "salt_seed_hash": H(keccak256(SALT_SEED)),
        "mint": H(MINT),
    },
    "leaf_vectors": [
        {
            "name": "brief_example",
            "distribution": H(key(3)), "owner": H(key(1)), "salt_seed": H(key(2)),
            "balance": 1000,
            "balance_le_u64": u64(1000).hex(),
            "salt": H(salt(key(3), key(1), key(2))),
            "leaf_inner": H(keccak256(key(1) + u64(1000) + salt(key(3), key(1), key(2)))),
            "leaf": H(leaf(key(1), 1000, salt(key(3), key(1), key(2)))),
        },
        {
            "name": "balance_one_shows_little_endian",
            "distribution": H(key(3)), "owner": H(key(1)), "salt_seed": H(key(2)),
            "balance": 1,
            "balance_le_u64": u64(1).hex(),
            "salt": H(salt(key(3), key(1), key(2))),
            "leaf_inner": H(keccak256(key(1) + u64(1) + salt(key(3), key(1), key(2)))),
            "leaf": H(leaf(key(1), 1, salt(key(3), key(1), key(2)))),
        },
    ],
    "tree_vectors": [t1, t2, t5],
    "manifest_vector": {
        "note": "Manifest of five_holders_sum_zero_frozen.",
        "fields": {k: (H(v) if isinstance(v, bytes) else v) for k, v in manifest.items()},
        "manifest_hash": H(manifest_hash(manifest)),
    },
}

here = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(here, "snapshot_v1.json"), "w") as f:
    json.dump(vectors, f, indent=2)
    f.write("\n")
print("wrote", os.path.join(here, "snapshot_v1.json"))
