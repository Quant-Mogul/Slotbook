"""Tests the getTokenHoldersAtSlot SQL in docs/SUPERBANK_PROPOSAL.md against Superbank's real schema.

Runs Query 1 (the register) and Query 2 (the guards) exactly as written in the doc, in chdb
(embedded ClickHouse), on Superbank's own DDL for `transactions`, `blocks_metadata` and `gsfa`,
downloaded at a pinned commit. Nothing from Superbank is stored in this repo.

The expected register does not come from the token-balance rows the SQL reads. A small ledger
simulator applies token operations (create, mint, transfer, close, set owner, ...) and writes the
pre/post token balances a validator would record; the expected register is the simulator's true
state at the record slot.

    python3 -m venv .venv && .venv/bin/pip install chdb
    .venv/bin/python scripts/superbank/test_holders_at_slot.py
"""
import copy
import hashlib
import json
import os
import random
import re
import shutil
import sys
import tempfile
import urllib.request

from chdb import session as chs

SUPERBANK_COMMIT = "5241078a0385f20bb81d13c40597937062c42e03"  # v0.8.0
DDL = ["transactions", "blocks_metadata", "gsfa"]
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
PROPOSAL = os.path.join(ROOT, "docs", "SUPERBANK_PROPOSAL.md")
RECORD_SLOT = 200
NEAR_U64_MAX = 18446744073709000000

B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58(b):
    n = int.from_bytes(b, "big")
    s = ""
    while n:
        n, r = divmod(n, 58)
        s = B58[r] + s
    return "1" * (len(b) - len(b.lstrip(b"\0"))) + s


def key(name):
    """32-byte address for a readable name."""
    return hashlib.sha256(name.encode()).digest()


# ---------------------------------------------------------------- SQL from the proposal

def proposal_sql():
    doc = open(PROPOSAL).read()
    q1 = re.search(r"### Query 1.*?```sql\n(.*?)```", doc, re.S).group(1)
    q2 = re.search(r"### Query 2.*?```sql\n(.*?)```", doc, re.S).group(1)
    return q1, q2


def bind(sql, mint, min_slot, record_slot):
    sql = re.sub(r"--[^\n]*", "", sql).strip().rstrip(";")
    return (sql.replace("{mint:String}", f"'{b58(key(mint))}'")
               .replace("{min_slot:UInt64}", str(min_slot))
               .replace("{record_slot:UInt64}", str(record_slot)))


# ---------------------------------------------------------------- ledger simulator

class Ledger:
    def __init__(self):
        self.acc = {}  # address name -> {mint, owner, amount}
        self.txs = []

    def tx(self, slot, idx, ops, ok=True, loaded=(), extra=(), metadata=True, null_owner=False):
        """Applies `ops` (if ok) and records the transaction as a validator would.
        Token accounts in `loaded` go in the v0 loaded-writable list instead of the static keys;
        `extra` adds non-token keys (for example the mint)."""
        touched = []
        for _, args in ops:
            for a in args:
                if isinstance(a, str) and a.startswith("acct:") and a not in touched:
                    touched.append(a)
        keys = ["payer", *extra, *[a for a in touched if a not in loaded]]
        loaded_w = [a for a in touched if a in loaded]
        all_keys = keys + loaded_w
        pre = [(i, dict(self.acc[k])) for i, k in enumerate(all_keys) if k in self.acc]
        if ok:
            for op, args in ops:
                getattr(self, op)(*args)
        post = [(i, dict(self.acc[k])) for i, k in enumerate(all_keys) if k in self.acc]
        self.txs.append(dict(slot=slot, idx=idx, ok=ok, keys=keys, loaded_w=loaded_w,
                             pre=pre, post=post, metadata=metadata, null_owner=null_owner))

    def create(self, a, mint, owner):
        assert a not in self.acc
        self.acc[a] = dict(mint=mint, owner=owner, amount=0)

    def mint_to(self, a, n):
        self.acc[a]["amount"] += n

    def burn(self, a, n):
        self.acc[a]["amount"] -= n

    def transfer(self, src, dst, n):
        assert self.acc[src]["mint"] == self.acc[dst]["mint"] and self.acc[src]["amount"] >= n
        self.acc[src]["amount"] -= n
        self.acc[dst]["amount"] += n

    def close(self, a):
        assert self.acc[a]["amount"] == 0
        del self.acc[a]

    def set_owner(self, a, owner):
        self.acc[a]["owner"] = owner


def true_register(accounts, mint):
    """Spec section 3: owner -> summed raw balance, zero dropped, sorted by owner bytes."""
    out = {}
    for a in accounts.values():
        if a["mint"] == mint and a["amount"] > 0:
            out[a["owner"]] = out.get(a["owner"], 0) + a["amount"]
    return sorted(((key(o).hex(), b) for o, b in out.items()), key=lambda r: bytes.fromhex(r[0]))


def build(extra=None):
    L, M, N = Ledger(), "mint:M", "mint:N"
    L.tx(100, 0, [], extra=[M])  # InitializeMint names only the mint
    L.tx(101, 0, [("create", ("acct:A1", M, "alice")), ("create", ("acct:B1", M, "bob")),
                  ("mint_to", ("acct:A1", 700)), ("mint_to", ("acct:B1", 300))], extra=[M])
    L.tx(101, 1, [("create", ("acct:L1", M, "leo")), ("mint_to", ("acct:L1", 77))], extra=[M])  # never touched again
    L.tx(102, 0, [("create", ("acct:C1", M, "carol")), ("transfer", ("acct:A1", "acct:C1", 150))])
    L.tx(102, 1, [("transfer", ("acct:A1", "acct:C1", 50))])  # same slot, later index
    L.tx(103, 0, [("create", ("acct:A2", M, "alice")), ("transfer", ("acct:B1", "acct:A2", 40))])  # two accounts
    L.tx(104, 0, [("transfer", ("acct:C1", "acct:B1", 100))], ok=False)  # failed
    L.tx(105, 0, [("create", ("acct:X", M, "dave")), ("transfer", ("acct:A1", "acct:X", 30))])
    L.tx(105, 1, [("transfer", ("acct:X", "acct:B1", 30)), ("close", ("acct:X",))])  # closed in the same slot
    L.tx(106, 0, [("create", ("acct:R", M, "erin"))])
    L.tx(106, 1, [("close", ("acct:R",))])
    L.tx(107, 0, [("create", ("acct:R", N, "erin")), ("mint_to", ("acct:R", 999))], extra=[N])  # reused for mint N
    L.tx(108, 0, [("create", ("acct:Z", M, "frank"))])  # zero balance
    L.tx(109, 0, [("set_owner", ("acct:C1", "gina"))])  # SetAuthority owner change
    L.tx(110, 0, [("create", ("acct:H1", M, "hank")), ("transfer", ("acct:A1", "acct:H1", 20))],
         loaded=["acct:H1"])  # only in v0 loaded addresses
    L.tx(111, 0, [("create", ("acct:R2", N, "ivy")), ("mint_to", ("acct:R2", 5))], extra=[N])
    L.tx(112, 0, [("burn", ("acct:R2", 5)), ("close", ("acct:R2",))])
    L.tx(113, 0, [("create", ("acct:R2", M, "ivy")), ("transfer", ("acct:B1", "acct:R2", 10))])  # reused for mint M
    L.tx(120, 0, [("create", ("acct:Q", N, "judy")), ("mint_to", ("acct:Q", 50))], extra=[N])  # other mint only
    L.tx(150, 0, [("create", ("acct:K1", M, "kim")), ("mint_to", ("acct:K1", NEAR_U64_MAX))], extra=[M])
    L.tx(150, 1, [("create", ("acct:K2", M, "kim")), ("mint_to", ("acct:K2", NEAR_U64_MAX))], extra=[M])  # sum > u64
    if extra:
        extra(L)
    at_record_slot = copy.deepcopy(L.acc)
    L.tx(250, 0, [("transfer", ("acct:A1", "acct:B1", 100))])  # after the record slot
    return L, at_record_slot


# ---------------------------------------------------------------- chdb

COLS = ("signature, slot, slot_idx, meta_status_ok, tx_account_keys, meta_loaded_addresses_writable, "
        "meta_pre_token_balances_present, meta_pre_token_account_index, meta_pre_token_mint, "
        "meta_pre_token_owner, meta_pre_token_amount, "
        "meta_post_token_balances_present, meta_post_token_account_index, meta_post_token_mint, "
        "meta_post_token_owner, meta_post_token_amount")


def addr(name):
    return f"unhex('{key(name).hex()}')"


def arr(xs):
    return "[" + ",".join(xs) + "]"


def row(n, t):
    def balances(side):
        if not t["metadata"]:
            return ["[]"] * 4
        es = t[side]
        null = t["null_owner"] and side == "post"
        return [arr(str(i) for i, _ in es), arr(addr(e["mint"]) for _, e in es),
                arr("NULL" if null else addr(e["owner"]) for _, e in es),
                arr(f"'{e['amount']}'" for _, e in es)]
    sig = f"unhex('{hashlib.sha512(str(n).encode()).hexdigest()}')"
    flag = str(int(t["metadata"]))
    return "(" + ",".join([sig, str(t["slot"]), str(t["idx"]), str(int(t["ok"])),
                           arr(addr(k) for k in t["keys"]), arr(addr(k) for k in t["loaded_w"]),
                           flag, *balances("pre"), flag, *balances("post")]) + ")"


def ddl_statements(cache):
    for name in DDL:
        path = os.path.join(cache, f"{name}.sql")
        if not os.path.exists(path):
            url = f"https://raw.githubusercontent.com/solana-rpc/superbank/{SUPERBANK_COMMIT}/ddl/local/{name}.sql"
            with urllib.request.urlopen(url) as r, open(path, "wb") as f:
                f.write(r.read())
        text = re.sub(r"--[^\n]*", "", open(path).read())
        for stmt in re.split(r";[ \t]*(?:\n|$)", text):
            if stmt.strip() and "MODIFY QUERY" not in stmt:  # MV re-definition; same query on a fresh table
                yield stmt


def load(ledger, cache):
    d = tempfile.mkdtemp()
    sess = chs.Session(d)
    for stmt in ddl_statements(cache):
        sess.query(stmt)
    rows = list(enumerate(ledger.txs))
    random.Random(7).shuffle(rows)  # insert order must not matter
    sess.query(f"INSERT INTO default.transactions ({COLS}) VALUES " + ",".join(row(n, t) for n, t in rows))
    sess.query("INSERT INTO default.blocks_metadata (slot, parent_slot, blockhash, parent_blockhash, "
               "executed_transaction_count, entry_count, rewards_present) VALUES "
               + ",".join(f"({s},{s - 1},unhex(repeat('00',32)),unhex(repeat('00',32)),1,1,0)"
                          for s in sorted({t["slot"] for t in ledger.txs})))
    return sess, d


def register(sess, min_slot):
    q1, _ = proposal_sql()
    q = bind(q1, "mint:M", min_slot, RECORD_SLOT)
    out = sess.query(f"SELECT if(isNull(owner), 'NULL', lower(hex(owner))), toString(balance) FROM ({q}) "
                     "FORMAT TSV").bytes().decode()
    return [(o, int(b)) for o, b in (line.split("\t") for line in out.strip().splitlines())]


def guards(sess, min_slot):
    _, q2 = proposal_sql()
    return json.loads(sess.query(bind(q2, "mint:M", min_slot, RECORD_SLOT) + " FORMAT JSONEachRow").bytes())


# ---------------------------------------------------------------- checks

def main():
    cache = tempfile.mkdtemp()
    names = {key(n).hex(): n for n in ["alice", "bob", "carol", "dave", "erin", "frank", "gina",
                                       "hank", "ivy", "judy", "kim", "leo", "nobody"]}
    readable = lambda rows: [(names.get(o, o), b) for o, b in rows]
    results = []

    def check(name, ok, detail=""):
        results.append(ok)
        print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))

    ledger, at_s = build()
    want = true_register(at_s, "mint:M")
    sess, d = load(ledger, cache)
    got = register(sess, 100)
    check("register equals the true state at the record slot, in owner byte order", got == want,
          f"got {readable(got)}, want {readable(want)}")
    g = guards(sess, 100)
    check("guards on clean data", g["first_mint_slot"] == 100 and g["rows_missing_meta"] == 0
          and g["resolved_slot"] == 250, json.dumps(g))
    late = dict(register(sess, 103))
    check("a minSlot after the mint's first slot misses a holder (leo)",
          key("leo").hex() not in late and key("leo").hex() in dict(want))
    check("the guard exposes it: first_mint_slot < minSlot", guards(sess, 103)["first_mint_slot"] < 103)
    sess.close()
    shutil.rmtree(d)

    ledger, _ = build(lambda L: L.tx(130, 0, [("transfer", ("acct:A1", "acct:B1", 1))], metadata=False))
    sess, d = load(ledger, cache)
    g = guards(sess, 100)
    check("the guard counts a transaction without token metadata", g["rows_missing_meta"] == 1, json.dumps(g))
    sess.close()
    shutil.rmtree(d)

    ledger, _ = build(lambda L: L.tx(131, 0, [("create", ("acct:NUL", "mint:M", "nobody")),
                                              ("transfer", ("acct:A1", "acct:NUL", 5))], null_owner=True))
    sess, d = load(ledger, cache)
    nulls = [r for r in register(sess, 100) if r[0] == "NULL"]
    check("a NULL owner comes back as its own row, so the handler must refuse it", bool(nulls), str(nulls))
    sess.close()
    shutil.rmtree(d)

    shutil.rmtree(cache)
    print(f"\n{sum(results)} of {len(results)} checks passed")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
