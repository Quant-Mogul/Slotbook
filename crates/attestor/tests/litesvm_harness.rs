//! LiteSVM 0.10.0 harness: create a mint, transfer, freeze, extra account;
//! replay the instruction stream and match owner→balance from VM state.

use attestor::ledger::{RawInstruction, RawTx};
use attestor::replay::{replay, ReplayOpts};
use attestor::token::{token_program, TOKEN_PROGRAM};
use litesvm::LiteSVM;
use litesvm_token::{CreateAccount, CreateMint, MintTo, Transfer};
use snapshot::{decode_address, AddressBytes, HolderRegister, TokenAccountState};
use solana_address::Address;
use solana_keypair::Keypair;
use solana_signer::Signer;
use std::collections::BTreeMap;

fn b32(addr: Address) -> AddressBytes {
    addr.to_bytes()
}

fn token_ix(data: Vec<u8>, accounts: Vec<AddressBytes>) -> RawInstruction {
    RawInstruction {
        program_id: decode_address(TOKEN_PROGRAM).unwrap(),
        accounts,
        data,
    }
}

fn tx(slot: u64, ixs: Vec<RawInstruction>) -> RawTx {
    RawTx {
        slot,
        index: 0,
        signature: String::new(),
        failed: false,
        instructions: ixs,
        pre_token_balances: vec![],
        post_token_balances: vec![],
    }
}

fn init_mint_ix(mint: AddressBytes) -> RawInstruction {
    token_ix(vec![20, 6], vec![mint])
}

fn init_account_ix(
    account: AddressBytes,
    mint: AddressBytes,
    owner: AddressBytes,
) -> RawInstruction {
    let mut data = vec![18u8];
    data.extend_from_slice(&owner);
    token_ix(data, vec![account, mint])
}

fn mint_to_ix(account: AddressBytes, mint: AddressBytes, amount: u64) -> RawInstruction {
    let mut data = vec![7u8];
    data.extend_from_slice(&amount.to_le_bytes());
    token_ix(data, vec![account, mint, [0u8; 32]])
}

fn transfer_checked_ix(
    src: AddressBytes,
    mint: AddressBytes,
    dest: AddressBytes,
    amount: u64,
) -> RawInstruction {
    let mut data = vec![12u8];
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(9);
    token_ix(data, vec![src, mint, dest, [0u8; 32]])
}

fn register_from_svm(
    svm: &LiteSVM,
    mint: Address,
    token_accounts: &[Address],
    record_slot: u64,
) -> HolderRegister {
    let mut map = BTreeMap::new();
    for acc in token_accounts {
        if let Some(raw) = svm.get_account(acc) {
            if raw.data.len() < 109 {
                continue;
            }
            let mut owner = [0u8; 32];
            owner.copy_from_slice(&raw.data[32..64]);
            let amount = u64::from_le_bytes(raw.data[64..72].try_into().unwrap());
            let frozen = raw.data[108] == 2;
            map.insert(
                b32(*acc),
                TokenAccountState {
                    owner,
                    amount,
                    frozen,
                },
            );
        }
    }
    HolderRegister::from_accounts(b32(mint), record_slot, Some(1), &map)
}

#[test]
fn litesvm_replay_matches_on_chain_balances() {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = CreateMint::new(&mut svm, &payer)
        .authority(&payer.pubkey())
        .decimals(9)
        .send()
        .unwrap();

    let alice = Keypair::new();
    let bob = Keypair::new();
    svm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    svm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    let alice_ata = CreateAccount::new(&mut svm, &payer, &mint)
        .owner(&alice.pubkey())
        .send()
        .unwrap();
    let bob_ata = CreateAccount::new(&mut svm, &payer, &mint)
        .owner(&bob.pubkey())
        .send()
        .unwrap();
    let alice_extra = CreateAccount::new(&mut svm, &payer, &mint)
        .owner(&alice.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut svm, &payer, &mint, &alice_ata, 1000)
        .owner(&payer)
        .send()
        .unwrap();
    Transfer::new(&mut svm, &alice, &mint, &bob_ata, 400)
        .source(&alice_ata)
        .send()
        .unwrap();
    MintTo::new(&mut svm, &payer, &mint, &alice_extra, 50)
        .owner(&payer)
        .send()
        .unwrap();

    let mint_b = b32(mint);
    let alice_ata_b = b32(alice_ata);
    let bob_ata_b = b32(bob_ata);
    let alice_extra_b = b32(alice_extra);
    let alice_b = b32(alice.pubkey());
    let bob_b = b32(bob.pubkey());

    let raw_txs = vec![
        tx(1, vec![init_mint_ix(mint_b)]),
        tx(2, vec![init_account_ix(alice_ata_b, mint_b, alice_b)]),
        tx(3, vec![mint_to_ix(alice_ata_b, mint_b, 1000)]),
        tx(4, vec![init_account_ix(bob_ata_b, mint_b, bob_b)]),
        tx(
            5,
            vec![transfer_checked_ix(alice_ata_b, mint_b, bob_ata_b, 400)],
        ),
        tx(6, vec![init_account_ix(alice_extra_b, mint_b, alice_b)]),
        tx(7, vec![mint_to_ix(alice_extra_b, mint_b, 50)]),
    ];

    let replayed = replay(
        &ReplayOpts {
            mint: mint_b,
            record_slot: 7,
            require_frozen_default: false,
        },
        &raw_txs,
    )
    .expect("replay");

    let from_svm = register_from_svm(&svm, mint, &[alice_ata, bob_ata, alice_extra], 7);

    assert_eq!(
        replayed.rows, from_svm.rows,
        "replay must match LiteSVM state"
    );
    assert_eq!(replayed.balance_of(&alice_b), Some(650));
    assert_eq!(replayed.balance_of(&bob_b), Some(400));
    assert_eq!(replayed.register_total, 1050);
    let _ = token_program();
}
