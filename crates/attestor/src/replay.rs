use crate::error::{addr_str, AttestorError};
use crate::ledger::RawTx;
use crate::token::{discovery_accounts, AccountState, TokenOp};
use snapshot::{AddressBytes, HolderRegister, TokenAccountState};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct ReplayOpts {
    pub mint: AddressBytes,
    pub record_slot: u64,
    pub require_frozen_default: bool,
}

pub fn mint_frozen_at_creation(mint: &AddressBytes, txs: &[RawTx]) -> bool {
    let mut ordered: Vec<&RawTx> = txs.iter().filter(|tx| !tx.failed).collect();
    ordered.sort_by_key(|tx| (tx.slot, tx.index));
    for tx in ordered {
        let ops = tx.ops();
        let inits = ops
            .iter()
            .any(|op| matches!(op, TokenOp::InitializeMint { mint: m } if m == mint));
        if inits {
            return ops.iter().any(|op| {
                matches!(
                    op,
                    TokenOp::InitializeDefaultAccountState {
                        mint: m,
                        state: AccountState::Frozen
                    } if m == mint
                )
            });
        }
    }
    false
}

pub fn replay(opts: &ReplayOpts, txs: &[RawTx]) -> Result<HolderRegister, AttestorError> {
    if opts.require_frozen_default && !mint_frozen_at_creation(&opts.mint, txs) {
        return Err(AttestorError::MintNotFrozenDefault(addr_str(&opts.mint)));
    }
    let default_frozen = mint_frozen_at_creation(&opts.mint, txs);

    let mut ordered: Vec<&RawTx> = txs.iter().collect();
    ordered.sort_by_key(|tx| (tx.slot, tx.index));

    let mut discovered: BTreeSet<AddressBytes> = BTreeSet::new();
    let mut accounts: BTreeMap<AddressBytes, TokenAccountState> = BTreeMap::new();
    let mut first_covered_slot: Option<u64> = None;

    for tx in ordered {
        if tx.slot > opts.record_slot || tx.failed {
            continue;
        }
        first_covered_slot = Some(first_covered_slot.unwrap_or(tx.slot));
        let ops = tx.ops();
        for op in &ops {
            for acc in discovery_accounts(op, &opts.mint) {
                discovered.insert(acc);
            }
        }
        apply_ops(
            &opts.mint,
            default_frozen,
            &mut discovered,
            &mut accounts,
            &ops,
        )?;
        apply_balances(&opts.mint, &discovered, &mut accounts, tx);
    }

    Ok(HolderRegister::from_accounts(
        opts.mint,
        opts.record_slot,
        first_covered_slot,
        &accounts,
    ))
}

fn apply_ops(
    mint: &AddressBytes,
    default_frozen: bool,
    discovered: &mut BTreeSet<AddressBytes>,
    accounts: &mut BTreeMap<AddressBytes, TokenAccountState>,
    ops: &[TokenOp],
) -> Result<(), AttestorError> {
    for op in ops {
        match op {
            TokenOp::InitializeAccount {
                account,
                mint: m,
                owner,
            } if m == mint => {
                discovered.insert(*account);
                accounts.entry(*account).or_insert(TokenAccountState {
                    owner: *owner,
                    amount: 0,
                    frozen: default_frozen,
                });
            }
            TokenOp::MintTo {
                account,
                mint: m,
                amount,
            } if m == mint => {
                credit(accounts, account, *amount)?;
            }
            TokenOp::TransferChecked {
                source,
                mint: m,
                dest,
                amount,
            } if m == mint => {
                debit(accounts, source, *amount)?;
                credit(accounts, dest, *amount)?;
            }
            TokenOp::Transfer {
                source,
                dest,
                amount,
            } => {
                if !accounts.contains_key(source) {
                    continue;
                }
                debit(accounts, source, *amount)?;
                if accounts.contains_key(dest) {
                    credit(accounts, dest, *amount)?;
                }
            }
            TokenOp::Burn {
                account,
                mint: m,
                amount,
            } if m == mint => {
                debit(accounts, account, *amount)?;
            }
            TokenOp::Freeze {
                account, mint: m, ..
            } if m == mint => {
                if let Some(st) = accounts.get_mut(account) {
                    st.frozen = true;
                }
            }
            TokenOp::Thaw {
                account, mint: m, ..
            } if m == mint => {
                if let Some(st) = accounts.get_mut(account) {
                    st.frozen = false;
                }
            }
            TokenOp::CloseAccount { account } => {
                accounts.remove(account);
            }
            TokenOp::SetOwner { account, new_owner } => {
                if let Some(st) = accounts.get_mut(account) {
                    st.owner = *new_owner;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn apply_balances(
    mint: &AddressBytes,
    discovered: &BTreeSet<AddressBytes>,
    accounts: &mut BTreeMap<AddressBytes, TokenAccountState>,
    tx: &RawTx,
) {
    if tx.pre_token_balances.is_empty() && tx.post_token_balances.is_empty() {
        return;
    }
    let post_accounts: BTreeSet<AddressBytes> = tx
        .post_token_balances
        .iter()
        .filter(|b| b.mint == *mint)
        .map(|b| b.account)
        .collect();
    for pre in &tx.pre_token_balances {
        if pre.mint == *mint
            && discovered.contains(&pre.account)
            && !post_accounts.contains(&pre.account)
        {
            accounts.remove(&pre.account);
        }
    }
    for post in &tx.post_token_balances {
        if post.mint != *mint || !discovered.contains(&post.account) {
            continue;
        }
        let frozen = accounts
            .get(&post.account)
            .map(|s| s.frozen)
            .unwrap_or(false);
        accounts.insert(
            post.account,
            TokenAccountState {
                owner: post.owner,
                amount: post.amount,
                frozen,
            },
        );
    }
}

fn credit(
    accounts: &mut BTreeMap<AddressBytes, TokenAccountState>,
    account: &AddressBytes,
    amount: u64,
) -> Result<(), AttestorError> {
    let st = accounts
        .get_mut(account)
        .ok_or(AttestorError::UndiscoveredSource)?;
    st.amount = st
        .amount
        .checked_add(amount)
        .ok_or(AttestorError::BalanceOverflow)?;
    Ok(())
}

fn debit(
    accounts: &mut BTreeMap<AddressBytes, TokenAccountState>,
    account: &AddressBytes,
    amount: u64,
) -> Result<(), AttestorError> {
    let st = accounts
        .get_mut(account)
        .ok_or(AttestorError::UndiscoveredSource)?;
    st.amount = st
        .amount
        .checked_sub(amount)
        .ok_or(AttestorError::BalanceOverflow)?;
    Ok(())
}

/// Recompute one owner (resolver path). `claimed` is optional; when set it must match.
pub fn resolve_owner(
    register: &HolderRegister,
    owner: &AddressBytes,
    claimed: Option<u64>,
) -> Result<u64, AttestorError> {
    let replayed = register
        .balance_of(owner)
        .ok_or(AttestorError::OwnerNotInRegister)?;
    if let Some(claimed) = claimed {
        if claimed != replayed {
            return Err(AttestorError::ResolveMismatch { replayed, claimed });
        }
    }
    Ok(replayed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{RawInstruction, RawTx, TokenBalance};
    use crate::token::TOKEN_2022_PROGRAM;
    use snapshot::decode_address;

    fn pk(b: u8) -> AddressBytes {
        let mut a = [0u8; 32];
        a[0] = b;
        a
    }

    fn ix(data: Vec<u8>, accounts: Vec<AddressBytes>) -> RawInstruction {
        RawInstruction {
            program_id: decode_address(TOKEN_2022_PROGRAM).unwrap(),
            accounts,
            data,
        }
    }

    fn tx(slot: u64, index: u32, instructions: Vec<RawInstruction>) -> RawTx {
        RawTx {
            slot,
            index,
            signature: String::new(),
            failed: false,
            instructions,
            pre_token_balances: vec![],
            post_token_balances: vec![],
        }
    }

    fn init_mint_frozen(mint: AddressBytes) -> RawTx {
        tx(
            1,
            0,
            vec![ix(vec![20, 6], vec![mint]), ix(vec![28, 0, 2], vec![mint])],
        )
    }

    fn init_account(
        slot: u64,
        account: AddressBytes,
        mint: AddressBytes,
        owner: AddressBytes,
    ) -> RawTx {
        let mut data = vec![18u8];
        data.extend_from_slice(&owner);
        tx(slot, 0, vec![ix(data, vec![account, mint])])
    }

    fn mint_to(slot: u64, account: AddressBytes, mint: AddressBytes, amount: u64) -> RawTx {
        let mut data = vec![7u8];
        data.extend_from_slice(&amount.to_le_bytes());
        tx(slot, 0, vec![ix(data, vec![account, mint, pk(99)])])
    }

    fn transfer_checked(
        slot: u64,
        src: AddressBytes,
        mint: AddressBytes,
        dest: AddressBytes,
        amount: u64,
    ) -> RawTx {
        let mut data = vec![12u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.push(6);
        tx(slot, 0, vec![ix(data, vec![src, mint, dest, pk(99)])])
    }

    fn freeze(slot: u64, account: AddressBytes, mint: AddressBytes) -> RawTx {
        tx(slot, 0, vec![ix(vec![10], vec![account, mint, pk(99)])])
    }

    #[test]
    fn refuses_unfrozen_default_mint() {
        let mint = pk(9);
        let txs = vec![tx(1, 0, vec![ix(vec![20, 6], vec![mint])])];
        let err = replay(
            &ReplayOpts {
                mint,
                record_slot: 10,
                require_frozen_default: true,
            },
            &txs,
        )
        .unwrap_err();
        assert!(matches!(err, AttestorError::MintNotFrozenDefault(_)));
    }

    #[test]
    fn sums_owners_includes_frozen_drops_closed() {
        let mint = pk(9);
        let alice_ata = pk(10);
        let alice_extra = pk(11);
        let bob_ata = pk(12);
        let alice = pk(1);
        let bob = pk(2);
        let txs = vec![
            init_mint_frozen(mint),
            init_account(2, alice_ata, mint, alice),
            mint_to(3, alice_ata, mint, 1000),
            init_account(4, bob_ata, mint, bob),
            transfer_checked(5, alice_ata, mint, bob_ata, 400),
            freeze(6, bob_ata, mint),
            init_account(7, alice_extra, mint, alice),
            mint_to(8, alice_extra, mint, 50),
            // close none
        ];
        let reg = replay(
            &ReplayOpts {
                mint,
                record_slot: 8,
                require_frozen_default: true,
            },
            &txs,
        )
        .unwrap();
        assert_eq!(reg.balance_of(&alice), Some(650));
        assert_eq!(reg.balance_of(&bob), Some(400));
        assert_eq!(reg.register_total, 1050);
        assert_eq!(resolve_owner(&reg, &bob, Some(400)).unwrap(), 400);
    }

    #[test]
    fn ignores_slots_after_record() {
        let mint = pk(9);
        let ata = pk(10);
        let owner = pk(1);
        let txs = vec![
            init_mint_frozen(mint),
            init_account(2, ata, mint, owner),
            mint_to(3, ata, mint, 100),
            mint_to(9, ata, mint, 50),
        ];
        let reg = replay(
            &ReplayOpts {
                mint,
                record_slot: 3,
                require_frozen_default: true,
            },
            &txs,
        )
        .unwrap();
        assert_eq!(reg.balance_of(&owner), Some(100));
    }

    fn post_balance_tx(
        slot: u64,
        index: u32,
        account: AddressBytes,
        mint: AddressBytes,
        owner: AddressBytes,
        amount: u64,
    ) -> RawTx {
        let mut t = tx(slot, index, vec![]);
        t.post_token_balances = vec![TokenBalance {
            account,
            mint,
            owner,
            amount,
        }];
        t
    }

    #[test]
    fn same_slot_applies_by_index_not_vec_order() {
        let mint = pk(9);
        let ata = pk(10);
        let owner = pk(1);
        let init = vec![init_mint_frozen(mint), init_account(2, ata, mint, owner)];
        // Vec order is reversed; indices say mint-100 then transfer-out to 60.
        let mut txs = init.clone();
        txs.push(post_balance_tx(5, 1, ata, mint, owner, 60));
        txs.push(post_balance_tx(5, 0, ata, mint, owner, 100));
        let reg = replay(
            &ReplayOpts {
                mint,
                record_slot: 5,
                require_frozen_default: true,
            },
            &txs,
        )
        .unwrap();
        assert_eq!(reg.balance_of(&owner), Some(60));

        // Reversed indices: last write is 100, must not end at 60.
        let mut reversed = init;
        reversed.push(post_balance_tx(5, 0, ata, mint, owner, 60));
        reversed.push(post_balance_tx(5, 1, ata, mint, owner, 100));
        let reg = replay(
            &ReplayOpts {
                mint,
                record_slot: 5,
                require_frozen_default: true,
            },
            &reversed,
        )
        .unwrap();
        assert_eq!(reg.balance_of(&owner), Some(100));
        assert_ne!(reg.balance_of(&owner), Some(60));
    }
}
