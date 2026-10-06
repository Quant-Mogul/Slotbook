//! Off-chain snapshot types owned by the attestor track.
//!
//! Replay output is `owner -> balance`, summed per owner, frozen holders
//! included. Leaf / salt / tree encoding follows team brief D12–D13. Spec PR
//! may still refine canonical bytes; vectors are computed from the brief.

mod encode;
mod register;

pub use encode::{
    build_tree, leaf_hash, leaf_inner, manifest_hash, proof_for_owner, salt, salt_seed_hash,
    Backend, EncodeError, Leaf, Manifest, ProofBundle, SPEC_VERSION,
};
pub use register::{AddressBytes, HolderRegister, HolderRow, TokenAccountState};

pub fn encode_address(bytes: &AddressBytes) -> String {
    bs58::encode(bytes).into_string()
}

pub fn decode_address(s: &str) -> Result<AddressBytes, DecodeError> {
    let data = bs58::decode(s)
        .into_vec()
        .map_err(|_| DecodeError::InvalidBase58)?;
    AddressBytes::try_from(data.as_slice()).map_err(|_| DecodeError::Not32Bytes)
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("invalid base58")]
    InvalidBase58,
    #[error("address is not 32 bytes")]
    Not32Bytes,
}
