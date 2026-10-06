use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type AddressBytes = [u8; 32];

/// One owner's summed raw token balance at the record slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolderRow {
    pub owner: AddressBytes,
    pub balance: u64,
}

/// Per-token-account book used while replaying. Frozen accounts stay here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenAccountState {
    pub owner: AddressBytes,
    pub amount: u64,
    pub frozen: bool,
}

/// Deterministic holder register: owners sorted, balances summed, zeros dropped.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolderRegister {
    pub mint: AddressBytes,
    pub record_slot: u64,
    pub first_covered_slot: Option<u64>,
    pub rows: Vec<HolderRow>,
    pub register_total: u64,
}

impl HolderRegister {
    pub fn from_accounts(
        mint: AddressBytes,
        record_slot: u64,
        first_covered_slot: Option<u64>,
        accounts: &BTreeMap<AddressBytes, TokenAccountState>,
    ) -> Self {
        let mut summed: BTreeMap<AddressBytes, u64> = BTreeMap::new();
        for state in accounts.values() {
            if state.amount == 0 {
                continue;
            }
            *summed.entry(state.owner).or_insert(0) =
                summed.get(&state.owner).copied().unwrap_or(0) + state.amount;
        }
        let rows: Vec<HolderRow> = summed
            .into_iter()
            .filter(|(_, balance)| *balance > 0)
            .map(|(owner, balance)| HolderRow { owner, balance })
            .collect();
        let register_total = rows.iter().map(|r| r.balance).sum();
        Self {
            mint,
            record_slot,
            first_covered_slot,
            rows,
            register_total,
        }
    }

    pub fn balance_of(&self, owner: &AddressBytes) -> Option<u64> {
        self.rows
            .iter()
            .find(|r| r.owner == *owner)
            .map(|r| r.balance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(b: u8) -> AddressBytes {
        let mut a = [0u8; 32];
        a[0] = b;
        a
    }

    #[test]
    fn sums_per_owner_and_keeps_frozen() {
        let mut accounts = BTreeMap::new();
        accounts.insert(
            pk(10),
            TokenAccountState {
                owner: pk(1),
                amount: 40,
                frozen: false,
            },
        );
        accounts.insert(
            pk(11),
            TokenAccountState {
                owner: pk(1),
                amount: 10,
                frozen: false,
            },
        );
        accounts.insert(
            pk(12),
            TokenAccountState {
                owner: pk(2),
                amount: 7,
                frozen: true,
            },
        );
        accounts.insert(
            pk(13),
            TokenAccountState {
                owner: pk(3),
                amount: 0,
                frozen: false,
            },
        );
        let reg = HolderRegister::from_accounts(pk(9), 100, Some(1), &accounts);
        assert_eq!(reg.rows.len(), 2);
        assert_eq!(reg.balance_of(&pk(1)), Some(50));
        assert_eq!(reg.balance_of(&pk(2)), Some(7));
        assert_eq!(reg.balance_of(&pk(3)), None);
        assert_eq!(reg.register_total, 57);
    }
}
