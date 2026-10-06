use serde::{Deserialize, Serialize};
use snapshot::AddressBytes;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawInstruction {
    pub program_id: AddressBytes,
    pub accounts: Vec<AddressBytes>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBalance {
    pub account: AddressBytes,
    pub mint: AddressBytes,
    pub owner: AddressBytes,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTx {
    pub slot: u64,
    pub index: u32,
    pub failed: bool,
    pub instructions: Vec<RawInstruction>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
}

impl RawTx {
    pub fn ops(&self) -> Vec<crate::token::TokenOp> {
        self.instructions
            .iter()
            .filter_map(|ix| crate::token::decode_token_ix(&ix.program_id, &ix.accounts, &ix.data))
            .collect()
    }
}
