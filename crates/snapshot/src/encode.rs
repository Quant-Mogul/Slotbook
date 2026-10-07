use crate::register::{AddressBytes, HolderRegister, HolderRow};
use merkle::{keccak, verify, MerkleError, MerkleTree, Node, Proof};
use serde::{Deserialize, Serialize};

pub const SPEC_VERSION: &str = "1.0";
pub use merkle::{leaf_hash, leaf_inner, salt, SALT_PREFIX};

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
        let bundle = proof_for_owner(&register, &pk(8), &pk(7), &pk(4), 1, 42).unwrap();
        assert_eq!(bundle.balance, 90);
        assert!(verify(bundle.leaf, &bundle.proof, bundle.root));
        assert!(!verify(bundle.leaf, &bundle.proof, pk(0)));
    }
}
