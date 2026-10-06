use snapshot::{decode_address, AddressBytes};

pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

pub fn token_program() -> AddressBytes {
    decode_address(TOKEN_PROGRAM).expect("token program id")
}

pub fn token_2022_program() -> AddressBytes {
    decode_address(TOKEN_2022_PROGRAM).expect("token-2022 program id")
}

pub fn is_token_program(id: &AddressBytes) -> bool {
    *id == token_program() || *id == token_2022_program()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountState {
    Uninitialized = 0,
    Initialized = 1,
    Frozen = 2,
}

impl AccountState {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Uninitialized),
            1 => Some(Self::Initialized),
            2 => Some(Self::Frozen),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenOp {
    InitializeMint {
        mint: AddressBytes,
    },
    InitializeDefaultAccountState {
        mint: AddressBytes,
        state: AccountState,
    },
    InitializeAccount {
        account: AddressBytes,
        mint: AddressBytes,
        owner: AddressBytes,
    },
    MintTo {
        account: AddressBytes,
        mint: AddressBytes,
        amount: u64,
    },
    TransferChecked {
        source: AddressBytes,
        mint: AddressBytes,
        dest: AddressBytes,
        amount: u64,
    },
    Transfer {
        source: AddressBytes,
        dest: AddressBytes,
        amount: u64,
    },
    Burn {
        account: AddressBytes,
        mint: AddressBytes,
        amount: u64,
    },
    Freeze {
        account: AddressBytes,
        mint: AddressBytes,
    },
    Thaw {
        account: AddressBytes,
        mint: AddressBytes,
    },
    CloseAccount {
        account: AddressBytes,
    },
    SetOwner {
        account: AddressBytes,
        new_owner: AddressBytes,
    },
}

pub fn decode_token_ix(
    program_id: &AddressBytes,
    accounts: &[AddressBytes],
    data: &[u8],
) -> Option<TokenOp> {
    if !is_token_program(program_id) || data.is_empty() {
        return None;
    }
    let tag = data[0];
    match tag {
        0 | 20 => accounts.first().copied().map(|mint| TokenOp::InitializeMint { mint }),
        1 => {
            if accounts.len() < 3 {
                return None;
            }
            Some(TokenOp::InitializeAccount {
                account: accounts[0],
                mint: accounts[1],
                owner: accounts[2],
            })
        }
        16 | 18 => {
            if accounts.len() < 2 || data.len() < 33 {
                return None;
            }
            let mut owner = [0u8; 32];
            owner.copy_from_slice(&data[1..33]);
            Some(TokenOp::InitializeAccount {
                account: accounts[0],
                mint: accounts[1],
                owner,
            })
        }
        3 => {
            if accounts.len() < 2 || data.len() < 9 {
                return None;
            }
            Some(TokenOp::Transfer {
                source: accounts[0],
                dest: accounts[1],
                amount: u64::from_le_bytes(data[1..9].try_into().ok()?),
            })
        }
        7 | 14 => {
            if accounts.len() < 2 || data.len() < 9 {
                return None;
            }
            Some(TokenOp::MintTo {
                account: accounts[0],
                mint: accounts[1],
                amount: u64::from_le_bytes(data[1..9].try_into().ok()?),
            })
        }
        8 | 15 => {
            if accounts.len() < 2 || data.len() < 9 {
                return None;
            }
            Some(TokenOp::Burn {
                account: accounts[0],
                mint: accounts[1],
                amount: u64::from_le_bytes(data[1..9].try_into().ok()?),
            })
        }
        9 => accounts.first().copied().map(|account| TokenOp::CloseAccount { account }),
        10 => {
            if accounts.len() < 2 {
                return None;
            }
            Some(TokenOp::Freeze {
                account: accounts[0],
                mint: accounts[1],
            })
        }
        11 => {
            if accounts.len() < 2 {
                return None;
            }
            Some(TokenOp::Thaw {
                account: accounts[0],
                mint: accounts[1],
            })
        }
        12 => {
            if accounts.len() < 3 || data.len() < 9 {
                return None;
            }
            Some(TokenOp::TransferChecked {
                source: accounts[0],
                mint: accounts[1],
                dest: accounts[2],
                amount: u64::from_le_bytes(data[1..9].try_into().ok()?),
            })
        }
        6 => {
            if accounts.is_empty() || data.len() < 2 {
                return None;
            }
            // AuthorityType::AccountOwner = 2
            if data[1] != 2 {
                return None;
            }
            let new_owner = parse_coption_pubkey(&data[2..])?;
            Some(TokenOp::SetOwner {
                account: accounts[0],
                new_owner,
            })
        }
        28 => {
            if accounts.is_empty() || data.len() < 3 {
                return None;
            }
            let state = AccountState::from_u8(data[2])?;
            Some(TokenOp::InitializeDefaultAccountState {
                mint: accounts[0],
                state,
            })
        }
        _ => None,
    }
}

fn parse_coption_pubkey(data: &[u8]) -> Option<AddressBytes> {
    if data.len() < 36 {
        return None;
    }
    let tag = u32::from_le_bytes(data[0..4].try_into().ok()?);
    if tag != 1 {
        return None;
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&data[4..36]);
    Some(pk)
}

/// Token-2022 mint: base 82 bytes, account type, TLV. ExtensionType::DefaultAccountState = 6.
pub fn mint_default_account_state(data: &[u8]) -> Option<AccountState> {
    if data.len() <= 83 {
        return None;
    }
    // byte 82 is AccountType (Mint = 2) for Token-2022
    let mut i = 83usize;
    while i + 4 <= data.len() {
        let typ = u16::from_le_bytes(data[i..i + 2].try_into().ok()?);
        let len = u16::from_le_bytes(data[i + 2..i + 4].try_into().ok()?) as usize;
        i += 4;
        if i + len > data.len() {
            break;
        }
        if typ == 6 && len >= 1 {
            return AccountState::from_u8(data[i]);
        }
        i += len;
    }
    None
}

pub fn discovery_accounts<'a>(op: &'a TokenOp, mint: &AddressBytes) -> Vec<AddressBytes> {
    match op {
        TokenOp::InitializeAccount { account, mint: m, .. } if m == mint => vec![*account],
        TokenOp::MintTo { account, mint: m, .. } if m == mint => vec![*account],
        TokenOp::TransferChecked {
            source,
            mint: m,
            dest,
            ..
        } if m == mint => vec![*source, *dest],
        TokenOp::Burn { account, mint: m, .. } if m == mint => vec![*account],
        TokenOp::Freeze { account, mint: m, .. } if m == mint => vec![*account],
        TokenOp::Thaw { account, mint: m, .. } if m == mint => vec![*account],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_account_state_tlv() {
        let mut data = vec![0u8; 83];
        data[82] = 2;
        data.extend_from_slice(&6u16.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.push(2);
        assert_eq!(mint_default_account_state(&data), Some(AccountState::Frozen));
    }
}
