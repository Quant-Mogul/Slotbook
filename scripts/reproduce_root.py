#!/usr/bin/env python3
"""Demo step 3: reproduce a committed root from the register, using only the spec.

Uses the independent pure-Python implementation in docs/vectors/gen_snapshot_vectors.py
(its own Keccak-256, no code shared with the Rust crates).

    python3 scripts/reproduce_root.py <register.json> [expected_root_hex]

register.json:
    {"distribution": "<base58>", "salt_seed": "<64 hex>",
     "rows": [{"owner": "<base58>", "balance": <u64>}, ...]}

Prints the root. With an expected root, exits 1 if they differ.
"""

import importlib.util
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SPEC = os.path.join(HERE, "..", "docs", "vectors", "gen_snapshot_vectors.py")
_spec = importlib.util.spec_from_file_location("snapshot_spec", SPEC)
spec = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(spec)

_B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58decode(s: str) -> bytes:
    n = 0
    for ch in s:
        n = n * 58 + _B58.index(ch)
    raw = n.to_bytes((n.bit_length() + 7) // 8, "big") if n else b""
    pad = len(s) - len(s.lstrip("1"))
    out = b"\x00" * pad + raw
    if len(out) != 32:
        raise ValueError(f"{s} is not a 32-byte address")
    return out


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    reg = json.load(open(sys.argv[1]))
    distribution = b58decode(reg["distribution"])
    salt_seed = bytes.fromhex(reg["salt_seed"])
    accounts = [
        {"account": bytes([i]) * 32, "owner": b58decode(r["owner"]), "amount": int(r["balance"]), "frozen": False}
        for i, r in enumerate(reg["rows"])
    ]
    rows, total = spec.register(accounts)  # spec 3.4 to 3.7: sum per owner, drop zeros, sort
    leaves = [spec.leaf(o, b, spec.salt(distribution, o, salt_seed)) for o, b in rows]
    root = spec.levels(leaves)[-1][0].hex()
    print(f"rows {len(rows)}  register_total {total}  root {root}")
    if len(sys.argv) == 3:
        expected = sys.argv[2].lower().removeprefix("0x")
        if root != expected:
            print(f"MISMATCH: on-chain root is {expected}")
            sys.exit(1)
        print("MATCH: the on-chain root is reproduced byte for byte from the spec.")


if __name__ == "__main__":
    main()
