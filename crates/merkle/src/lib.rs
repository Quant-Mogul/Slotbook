//! Sorted-pair keccak Merkle tree and Slotbook leaf hashing.
//!
//! Brief D13: internal nodes are `keccak256(min(a,b) || max(a,b))` with no
//! prefix. An odd leftover node is promoted unhashed. Proof depth is at most 32.
//!
//! Leaf / salt (D12–D13) live here so the on-chain program can depend on this
//! crate without pulling `snapshot`.

use solana_keccak_hasher::hashv;

pub type Node = [u8; 32];

/// `salt = keccak256("slotbook-salt-v1" || distribution || owner || salt_seed)`
pub const SALT_PREFIX: &[u8] = b"slotbook-salt-v1";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MerkleError {
    #[error("merkle tree has no leaves")]
    Empty,
    #[error("proof depth exceeds 32")]
    TooDeep,
    #[error("leaf index {0} is out of range")]
    IndexOutOfRange(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Proof {
    pub siblings: Vec<Node>,
    pub index: u32,
    pub leaf_count: u32,
}

pub fn keccak(parts: &[&[u8]]) -> Node {
    hashv(parts).to_bytes()
}

pub fn salt(distribution: &[u8; 32], owner: &[u8; 32], salt_seed: &Node) -> Node {
    keccak(&[SALT_PREFIX, distribution, owner, salt_seed])
}

pub fn leaf_inner(owner: &[u8; 32], balance: u64, salt: &Node) -> Node {
    keccak(&[owner, &balance.to_le_bytes(), salt])
}

/// `leaf = keccak256(0x00 || keccak256(owner || balance_le_u64 || salt_32))`
pub fn leaf_hash(owner: &[u8; 32], balance: u64, salt: &Node) -> Node {
    let inner = leaf_inner(owner, balance, salt);
    keccak(&[&[0x00u8], &inner])
}

pub fn parent(a: &Node, b: &Node) -> Node {
    if a <= b {
        keccak(&[a.as_slice(), b.as_slice()])
    } else {
        keccak(&[b.as_slice(), a.as_slice()])
    }
}

#[derive(Clone, Debug)]
pub struct MerkleTree {
    leaves: Vec<Node>,
}

impl MerkleTree {
    pub fn from_leaves(leaves: Vec<Node>) -> Result<Self, MerkleError> {
        if leaves.is_empty() {
            return Err(MerkleError::Empty);
        }
        let depth = proof_depth(leaves.len());
        if depth > 32 {
            return Err(MerkleError::TooDeep);
        }
        Ok(Self { leaves })
    }

    pub fn leaves(&self) -> &[Node] {
        &self.leaves
    }

    pub fn root(&self) -> Node {
        fold_level(&self.leaves)
    }

    pub fn proof(&self, index: usize) -> Result<Proof, MerkleError> {
        if index >= self.leaves.len() {
            return Err(MerkleError::IndexOutOfRange(index));
        }
        let mut siblings = Vec::new();
        let mut level = self.leaves.clone();
        let mut idx = index;
        while level.len() > 1 {
            let odd = level.len() % 2 == 1;
            if !(odd && idx == level.len() - 1) {
                let sib = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
                siblings.push(level[sib]);
            }
            idx /= 2;
            level = next_level(&level);
        }
        if siblings.len() > 32 {
            return Err(MerkleError::TooDeep);
        }
        Ok(Proof {
            siblings,
            index: index as u32,
            leaf_count: self.leaves.len() as u32,
        })
    }
}

pub fn verify(leaf: Node, proof: &Proof, root: Node) -> bool {
    if proof.siblings.len() > 32 {
        return false;
    }
    let mut hash = leaf;
    let mut idx = proof.index as usize;
    let mut level_len = proof.leaf_count as usize;
    let mut sib_i = 0usize;
    while level_len > 1 {
        let odd = level_len % 2 == 1;
        if !(odd && idx == level_len - 1) {
            let Some(sib) = proof.siblings.get(sib_i) else {
                return false;
            };
            hash = parent(&hash, sib);
            sib_i += 1;
        }
        idx /= 2;
        level_len = level_len / 2 + level_len % 2;
    }
    hash == root && sib_i == proof.siblings.len()
}

fn next_level(level: &[Node]) -> Vec<Node> {
    let mut next = Vec::with_capacity(level.len() / 2 + 1);
    let mut i = 0;
    while i + 1 < level.len() {
        next.push(parent(&level[i], &level[i + 1]));
        i += 2;
    }
    if i < level.len() {
        next.push(level[i]);
    }
    next
}

fn fold_level(leaves: &[Node]) -> Node {
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        level = next_level(&level);
    }
    level[0]
}

fn proof_depth(mut n: usize) -> usize {
    let mut d = 0;
    while n > 1 {
        d += 1;
        n = n / 2 + n % 2;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(b: u8) -> Node {
        let mut x = [0u8; 32];
        x[31] = b;
        x
    }

    #[test]
    fn single_leaf_is_root() {
        let t = MerkleTree::from_leaves(vec![n(7)]).unwrap();
        assert_eq!(t.root(), n(7));
        let p = t.proof(0).unwrap();
        assert!(p.siblings.is_empty());
        assert!(verify(n(7), &p, t.root()));
    }

    #[test]
    fn sorted_pair_two_leaves() {
        let t = MerkleTree::from_leaves(vec![n(2), n(1)]).unwrap();
        assert_eq!(t.root(), parent(&n(2), &n(1)));
        for i in 0..2 {
            let p = t.proof(i).unwrap();
            assert!(verify(t.leaves()[i], &p, t.root()));
        }
    }

    #[test]
    fn odd_leaf_promoted_unhashed() {
        let t = MerkleTree::from_leaves(vec![n(1), n(2), n(3)]).unwrap();
        let pair = parent(&n(1), &n(2));
        assert_eq!(t.root(), parent(&pair, &n(3)));
        for i in 0..3 {
            let p = t.proof(i).unwrap();
            assert!(verify(t.leaves()[i], &p, t.root()));
        }
    }

    #[test]
    fn empty_rejected() {
        assert_eq!(
            MerkleTree::from_leaves(vec![]).unwrap_err(),
            MerkleError::Empty
        );
    }

    #[test]
    fn salt_and_leaf_match_brief_formula() {
        let distribution = [3u8; 32];
        let owner = [1u8; 32];
        let salt_seed = [2u8; 32];
        let s = salt(&distribution, &owner, &salt_seed);
        assert_eq!(s, keccak(&[SALT_PREFIX, &distribution, &owner, &salt_seed]));
        let inner = keccak(&[&owner, &1000u64.to_le_bytes(), &s]);
        assert_eq!(leaf_inner(&owner, 1000, &s), inner);
        assert_eq!(leaf_hash(&owner, 1000, &s), keccak(&[&[0x00u8], &inner]));
    }
}
