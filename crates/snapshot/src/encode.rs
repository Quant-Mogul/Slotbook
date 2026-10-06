use crate::register::{AddressBytes, HolderRegister, HolderRow};
use merkle::{keccak, verify, MerkleError, MerkleTree, Node, Proof};
use serde::{Deserialize, Serialize};

pub const SPEC_VERSION: &str = "1.0";
pub const SALT_PREFIX: &[u8] = b"slotbook-salt-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Backend {
    LedgerReplay = 0,
    StateArchive = 1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leaf {
    pub owner: AddressBytes,
    pub balance: u64,
    pub salt: Node,
    pub hash: Node,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub spec_version: String,
    pub mint: AddressBytes,
    pub record_slot: u64,
    pub first_covered_slot: u64,
    pub resolved_slot: u64,
    pub backend: Backend,
    pub register_total: u64,
    pub rows: u32,
    pub root: Node,
    pub salt_seed_hash: Node,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofBundle {
    pub owner: AddressBytes,
    pub balance: u64,
    pub salt: Node,
    pub leaf: Node,
    pub proof: Proof,
    pub root: Node,
}

/// `salt = keccak256("slotbook-salt-v1" || distribution || owner || salt_seed)`
pub fn salt(distribution: &AddressBytes, owner: &AddressBytes, salt_seed: &Node) -> Node {
    keccak(&[SALT_PREFIX, distribution, owner, salt_seed])
}

/// Inner hash: `keccak256(owner || balance_le_u64 || salt_32)`
pub fn leaf_inner(owner: &AddressBytes, balance: u64, salt: &Node) -> Node {
    keccak(&[owner, &balance.to_le_bytes(), salt])
}

/// Leaf: `keccak256(0x00 || inner)`
pub fn leaf_hash(owner: &AddressBytes, balance: u64, salt: &Node) -> Node {
    let inner = leaf_inner(owner, balance, salt);
    keccak(&[&[0x00u8], &inner])
}

pub fn salt_seed_hash(salt_seed: &Node) -> Node {
    keccak(&[salt_seed])
}

pub fn manifest_hash(manifest: &Manifest) -> Node {
    keccak(&[
        &manifest.root,
        &manifest.register_total.to_le_bytes(),
        &manifest.mint,
        &manifest.record_slot.to_le_bytes(),
        &manifest.first_covered_slot.to_le_bytes(),
        &manifest.resolved_slot.to_le_bytes(),
        &manifest.rows.to_le_bytes(),
        manifest.spec_version.as_bytes(),
        &[manifest.backend as u8],
        &manifest.salt_seed_hash,
    ])
}

pub fn build_tree(
    register: &HolderRegister,
    distribution: &AddressBytes,
    salt_seed: &Node,
    first_covered_slot: u64,
    resolved_slot: u64,
) -> Result<(MerkleTree, Manifest, Vec<Leaf>), MerkleError> {
    let mut leaves_meta = Vec::with_capacity(register.rows.len());
    let mut nodes = Vec::with_capacity(register.rows.len());
    for HolderRow { owner, balance } in &register.rows {
        let s = salt(distribution, owner, salt_seed);
        let hash = leaf_hash(owner, *balance, &s);
        nodes.push(hash);
        leaves_meta.push(Leaf {
            owner: *owner,
            balance: *balance,
            salt: s,
            hash,
        });
    }
    let tree = MerkleTree::from_leaves(nodes)?;
    let manifest = Manifest {
        spec_version: SPEC_VERSION.to_string(),
        mint: register.mint,
        record_slot: register.record_slot,
        first_covered_slot,
        resolved_slot,
        backend: Backend::LedgerReplay,
        register_total: register.register_total,
        rows: register.rows.len() as u32,
        root: tree.root(),
        salt_seed_hash: salt_seed_hash(salt_seed),
    };
    Ok((tree, manifest, leaves_meta))
}

pub fn proof_for_owner(
    register: &HolderRegister,
    distribution: &AddressBytes,
    salt_seed: &Node,
    owner: &AddressBytes,
    first_covered_slot: u64,
    resolved_slot: u64,
) -> Result<ProofBundle, EncodeError> {
    let (tree, manifest, leaves) = build_tree(
        register,
        distribution,
        salt_seed,
        first_covered_slot,
        resolved_slot,
    )?;
    let index = leaves
        .iter()
        .position(|l| l.owner == *owner)
        .ok_or(EncodeError::OwnerNotInRegister)?;
    let proof = tree.proof(index)?;
    let leaf = &leaves[index];
    debug_assert!(verify(leaf.hash, &proof, manifest.root));
    Ok(ProofBundle {
        owner: leaf.owner,
        balance: leaf.balance,
        salt: leaf.salt,
        leaf: leaf.hash,
        proof,
        root: manifest.root,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error(transparent)]
    Merkle(#[from] MerkleError),
    #[error("owner is not in the register")]
    OwnerNotInRegister,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HolderRegister, HolderRow};

    fn pk(fill: u8) -> AddressBytes {
        [fill; 32]
    }

    /// Independent vector from brief D12–D13 (not copied from another implementation).
    #[test]
    fn salt_and_leaf_match_brief_formula() {
        let distribution = pk(3);
        let owner = pk(1);
        let salt_seed = pk(2);
        let s = salt(&distribution, &owner, &salt_seed);
        let expected_salt = keccak(&[SALT_PREFIX, &distribution, &owner, &salt_seed]);
        assert_eq!(s, expected_salt);
        let inner = keccak(&[&owner, &1000u64.to_le_bytes(), &s]);
        assert_eq!(leaf_inner(&owner, 1000, &s), inner);
        assert_eq!(leaf_hash(&owner, 1000, &s), keccak(&[&[0x00u8], &inner]));
    }

    #[test]
    fn independent_vector_file_matches_brief() {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../vectors/leaf_v1.json")).unwrap();
        let distribution: AddressBytes =
            serde_json::from_value(v["distribution"].clone()).unwrap();
        let owner: AddressBytes = serde_json::from_value(v["owner"].clone()).unwrap();
        let salt_seed: Node = serde_json::from_value(v["salt_seed"].clone()).unwrap();
        let balance = v["balance"].as_u64().unwrap();
        let s = salt(&distribution, &owner, &salt_seed);
        assert_eq!(
            s,
            keccak(&[SALT_PREFIX, &distribution, &owner, &salt_seed])
        );
        assert_eq!(
            leaf_hash(&owner, balance, &s),
            keccak(&[&[0x00u8], &leaf_inner(&owner, balance, &s)])
        );
    }

    #[test]
    fn tree_and_proof_round_trip() {
        let register = HolderRegister {
            mint: pk(9),
            record_slot: 42,
            first_covered_slot: Some(1),
            rows: vec![
                HolderRow {
                    owner: pk(1),
                    balance: 10,
                },
                HolderRow {
                    owner: pk(4),
                    balance: 90,
                },
            ],
            register_total: 100,
        };
        let bundle =
            proof_for_owner(&register, &pk(8), &pk(7), &pk(4), 1, 42).unwrap();
        assert_eq!(bundle.balance, 90);
        assert!(verify(bundle.leaf, &bundle.proof, bundle.root));
        assert!(!verify(bundle.leaf, &bundle.proof, pk(0)));
    }
}
