//! Checks the crates against docs/vectors/snapshot_v1.json, which is produced
//! by an independent Python implementation of docs/SNAPSHOT_SPEC.md.
//! Regenerate with: python3 docs/vectors/gen_snapshot_vectors.py

use merkle::{keccak, parent, MerkleTree, Node};
use serde_json::Value;
use snapshot::{leaf_hash, leaf_inner, salt, salt_seed_hash, HolderRegister, TokenAccountState};
use std::collections::BTreeMap;

const VECTORS: &str = include_str!("../../../docs/vectors/snapshot_v1.json");

fn vectors() -> Value {
    serde_json::from_str(VECTORS).expect("snapshot_v1.json parses")
}

fn hex(v: &Value) -> Vec<u8> {
    let s = v.as_str().expect("hex string");
    assert!(s.len() % 2 == 0, "odd-length hex");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex digit"))
        .collect()
}

fn b32(v: &Value) -> [u8; 32] {
    hex(v).try_into().expect("32 bytes")
}

/// Spec section 8: verification with siblings only.
fn verify_siblings(leaf: Node, siblings: &[Node], root: Node) -> bool {
    siblings.len() <= 32 && siblings.iter().fold(leaf, |h, s| parent(&h, s)) == root
}

#[test]
fn keccak_known_answers() {
    for ka in vectors()["keccak256_known_answers"].as_array().unwrap() {
        let input = ka["input_utf8"].as_str().unwrap().as_bytes();
        assert_eq!(keccak(&[input]).to_vec(), hex(&ka["hash"]), "keccak({input:?})");
    }
}

#[test]
fn salt_seed_hash_matches() {
    let c = &vectors()["common"];
    assert_eq!(salt_seed_hash(&b32(&c["salt_seed"])), b32(&c["salt_seed_hash"]));
}

#[test]
fn leaf_vectors_match() {
    for v in vectors()["leaf_vectors"].as_array().unwrap() {
        let name = v["name"].as_str().unwrap();
        let owner = b32(&v["owner"]);
        let balance = v["balance"].as_u64().unwrap();
        assert_eq!(balance.to_le_bytes().to_vec(), hex(&v["balance_le_u64"]), "{name}: LE");
        let s = salt(&b32(&v["distribution"]), &owner, &b32(&v["salt_seed"]));
        assert_eq!(s, b32(&v["salt"]), "{name}: salt");
        assert_eq!(leaf_inner(&owner, balance, &s), b32(&v["leaf_inner"]), "{name}: inner");
        assert_eq!(leaf_hash(&owner, balance, &s), b32(&v["leaf"]), "{name}: leaf");
    }
}

#[test]
fn tree_vectors_match() {
    let all = vectors();
    let distribution = b32(&all["common"]["distribution"]);
    let salt_seed = b32(&all["common"]["salt_seed"]);
    let mint = b32(&all["common"]["mint"]);

    for v in all["tree_vectors"].as_array().unwrap() {
        let name = v["name"].as_str().unwrap();

        // Section 3: register from token accounts.
        let mut accounts = BTreeMap::new();
        for a in v["accounts"].as_array().unwrap() {
            accounts.insert(
                b32(&a["account"]),
                TokenAccountState {
                    owner: b32(&a["owner"]),
                    amount: a["amount"].as_u64().unwrap(),
                    frozen: a["frozen"].as_bool().unwrap(),
                },
            );
        }
        let reg = HolderRegister::from_accounts(mint, 0, None, &accounts);
        let rows = v["rows"].as_array().unwrap();
        assert_eq!(reg.rows.len(), rows.len(), "{name}: row count");
        assert_eq!(reg.register_total, v["register_total"].as_u64().unwrap(), "{name}: total");

        // Sections 5-7: salts, leaves, tree.
        let mut leaves = Vec::new();
        for (row, expected) in reg.rows.iter().zip(rows) {
            assert_eq!(row.owner, b32(&expected["owner"]), "{name}: row order");
            assert_eq!(row.balance, expected["balance"].as_u64().unwrap(), "{name}: balance");
            let s = salt(&distribution, &row.owner, &salt_seed);
            assert_eq!(s, b32(&expected["salt"]), "{name}: salt");
            let lf = leaf_hash(&row.owner, row.balance, &s);
            assert_eq!(lf, b32(&expected["leaf"]), "{name}: leaf");
            leaves.push(lf);
        }
        let tree = MerkleTree::from_leaves(leaves.clone()).unwrap();
        let root = b32(&v["root"]);
        assert_eq!(tree.root(), root, "{name}: root");

        // Section 8: proofs are siblings only, bottom-up.
        for (i, expected) in rows.iter().enumerate() {
            let sibs: Vec<Node> = expected["proof"].as_array().unwrap().iter().map(b32).collect();
            assert_eq!(tree.proof(i).unwrap().siblings, sibs, "{name}: proof {i}");
            assert!(verify_siblings(leaves[i], &sibs, root), "{name}: verify {i}");
        }
    }
}

/// Section 11 encoding, written out from the spec. `snapshot::manifest_hash`
/// does not follow v1.0 yet (no distribution field, string spec_version);
/// the attestor track aligns it and then switches this test to call it.
#[test]
fn manifest_vector_matches_spec_encoding() {
    let m = &vectors()["manifest_vector"];
    let f = &m["fields"];
    let u64le = |k: &str| f[k].as_u64().unwrap().to_le_bytes();
    let spec_version = (f["spec_version"].as_u64().unwrap() as u16).to_le_bytes();
    let backend = [f["backend"].as_u64().unwrap() as u8];
    let rows = (f["rows"].as_u64().unwrap() as u32).to_le_bytes();
    let (distribution, mint, root, ssh) =
        (b32(&f["distribution"]), b32(&f["mint"]), b32(&f["root"]), b32(&f["salt_seed_hash"]));
    let h = keccak(&[
        b"slotbook-manifest-v1",
        &spec_version,
        &distribution,
        &mint,
        &u64le("record_slot"),
        &u64le("first_covered_slot"),
        &u64le("resolved_slot"),
        &backend,
        &rows,
        &u64le("register_total"),
        &root,
        &ssh,
    ]);
    assert_eq!(h, b32(&m["manifest_hash"]));
}
