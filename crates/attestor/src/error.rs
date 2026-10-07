use snapshot::AddressBytes;

#[derive(Debug, thiserror::Error)]
pub enum AttestorError {
    #[error("mint {0} is not DefaultAccountState=Frozen at creation; refusing replay")]
    MintNotFrozenDefault(String),
    #[error("mint account is missing or not Token-2022/SPL token mint")]
    InvalidMint,
    #[error("arithmetic overflow applying token instruction")]
    BalanceOverflow,
    #[error("transfer from undiscovered token account")]
    UndiscoveredSource,
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("invalid address: {0}")]
    Address(String),
    #[error("owner is not in the register at this slot")]
    OwnerNotInRegister,
    #[error("owner balance mismatch: replayed {replayed}, claimed {claimed}")]
    ResolveMismatch { replayed: u64, claimed: u64 },
    #[error(transparent)]
    Snapshot(#[from] snapshot::DecodeError),
    #[error(transparent)]
    Encode(#[from] snapshot::EncodeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Merkle(#[from] merkle::MerkleError),
}

pub fn addr_str(a: &AddressBytes) -> String {
    snapshot::encode_address(a)
}
