//! Happy path, demo steps 1, 2 and 6 in LiteSVM:
//! init_issuer -> register two attestors -> declare -> commit matching roots ->
//! finalize -> a thawed holder is paid, a frozen holder lands in Pending.
//! Run after `anchor build` (loads target/deploy/slotbook.so).

use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, program_option::COption, system_program},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id},
    token,
    token_2022::spl_token_2022::{
        self as t22,
        extension::{
            default_account_state::DefaultAccountState, immutable_owner::ImmutableOwner,
            BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
        },
        state::{Account as T22Account, AccountState, Mint as T22Mint},
    },
    token_interface::spl_token_2022::state::Account as AnyTokenAccount,
};
use litesvm::LiteSVM;
use litesvm_token::{CreateAccount, CreateAssociatedTokenAccountIdempotent, CreateMint, MintTo};
use merkle::MerkleTree;
use slotbook::{
    state::{ClaimReceipt, ClaimStatus, CommitArgs, Distribution, DistributionState, IssuerParams, Pending},
    SPEC_VERSION, TOKEN_ACL_MINT_CONFIG_SEED, TOKEN_ACL_PROGRAM_ID,
};
use solana_account::Account;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

const TOTAL: u64 = 1_000;
const SALT_SEED: [u8; 32] = [7; 32];
const NOTICE: u64 = 100;
const MARGIN: u64 = 32;
const WINDOW: u64 = 150;

fn send(svm: &mut LiteSVM, ixs: &[Instruction], payer: &Keypair, signers: &[&Keypair]) -> Result<(), String> {
    svm.expire_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}\n{}", e.err, e.meta.logs.join("\n")))
}

fn ix<A: ToAccountMetas, D: InstructionData>(accounts: A, data: D) -> Instruction {
    Instruction { program_id: slotbook::id(), accounts: accounts.to_account_metas(None), data: data.data() }
}

fn expect_err(r: Result<(), String>, needle: &str) {
    let e = r.expect_err(&format!("expected failure containing {needle}"));
    assert!(e.contains(needle), "expected {needle}, got:\n{e}");
}

fn set_t22(svm: &mut LiteSVM, address: Pubkey, data: Vec<u8>) {
    let lamports = svm.minimum_balance_for_rent_exemption(data.len());
    svm.set_account(address, Account { lamports, data, owner: t22::ID, executable: false, rent_epoch: 0 })
        .unwrap();
}

/// Token-2022 mint with DefaultAccountState = Frozen (D15), freeze authority = Token ACL PDA (D22).
fn permissioned_mint(authority: Pubkey, freeze: Pubkey, supply: u64) -> Vec<u8> {
    let len = ExtensionType::try_calculate_account_len::<T22Mint>(&[ExtensionType::DefaultAccountState]).unwrap();
    let mut data = vec![0u8; len];
    let mut s = StateWithExtensionsMut::<T22Mint>::unpack_uninitialized(&mut data).unwrap();
    s.init_extension::<DefaultAccountState>(true).unwrap().state = AccountState::Frozen as u8;
    s.base = T22Mint {
        mint_authority: COption::Some(authority),
        supply,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::Some(freeze),
    };
    s.pack_base();
    s.init_account_type().unwrap();
    data
}

/// Token-2022 holder account with ImmutableOwner, in the given state.
fn holder_account(mint: Pubkey, owner: Pubkey, amount: u64, state: AccountState) -> Vec<u8> {
    let len = ExtensionType::try_calculate_account_len::<T22Account>(&[ExtensionType::ImmutableOwner]).unwrap();
    let mut data = vec![0u8; len];
    let mut s = StateWithExtensionsMut::<T22Account>::unpack_uninitialized(&mut data).unwrap();
    s.init_extension::<ImmutableOwner>(true).unwrap();
    s.base = T22Account {
        mint,
        owner,
        amount,
        delegate: COption::None,
        state,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    s.pack_base();
    s.init_account_type().unwrap();
    data
}

fn token_amount(svm: &LiteSVM, address: &Pubkey) -> u64 {
    let acc = svm.get_account(address).unwrap();
    anchor_lang::solana_program::program_pack::Pack::unpack(&acc.data[..165])
        .map(|a: AnyTokenAccount| a.amount)
        .unwrap()
}

fn read<T: AccountDeserialize>(svm: &LiteSVM, address: &Pubkey) -> T {
    T::try_deserialize(&mut svm.get_account(address).unwrap().data.as_slice()).unwrap()
}

struct Holder {
    kp: Keypair,
    balance: u64,
    mint_account: Pubkey,
    payment_account: Pubkey,
}

#[test]
fn happy_path() {
    let mut svm = LiteSVM::new();
    let program = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/slotbook.so"));
    svm.add_program(slotbook::id(), program).unwrap();

    let issuer = Keypair::new();
    let att1 = Keypair::new();
    let att2 = Keypair::new();
    let resolver = Keypair::new();
    for k in [&issuer, &att1, &att2] {
        svm.airdrop(&k.pubkey(), 10_000_000_000).unwrap();
    }

    // Bond and payment mints: legacy SPL, like devnet USDC.
    let bond_mint = CreateMint::new(&mut svm, &issuer).authority(&issuer.pubkey()).decimals(0).send().unwrap();
    let pay_mint = CreateMint::new(&mut svm, &issuer).authority(&issuer.pubkey()).decimals(0).send().unwrap();

    // The permissioned mint.
    let mint = Pubkey::new_unique();
    let (mint_config, _) =
        Pubkey::find_program_address(&[TOKEN_ACL_MINT_CONFIG_SEED, mint.as_ref()], &TOKEN_ACL_PROGRAM_ID);
    set_t22(&mut svm, mint, permissioned_mint(issuer.pubkey(), mint_config, 1_000));

    // Holders at the record slot: one thawed (paid), one frozen (held in Pending).
    let mut holders = Vec::new();
    for (balance, state) in [(600, AccountState::Initialized), (400, AccountState::Frozen)] {
        let kp = Keypair::new();
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
        let mint_account = Pubkey::new_unique();
        set_t22(&mut svm, mint_account, holder_account(mint, kp.pubkey(), balance, state));
        let payment_account = CreateAccount::new(&mut svm, &issuer, &pay_mint).owner(&kp.pubkey()).send().unwrap();
        holders.push(Holder { kp, balance, mint_account, payment_account });
    }

    // Funding: issuer pays TOTAL; attestors post bonds.
    let issuer_pay = CreateAccount::new(&mut svm, &issuer, &pay_mint).owner(&issuer.pubkey()).send().unwrap();
    MintTo::new(&mut svm, &issuer, &pay_mint, &issuer_pay, TOTAL).owner(&issuer).send().unwrap();
    let mut bond_accounts = Vec::new();
    for a in [&att1, &att2] {
        let acc = CreateAccount::new(&mut svm, &issuer, &bond_mint).owner(&a.pubkey()).send().unwrap();
        MintTo::new(&mut svm, &issuer, &bond_mint, &acc, 10).owner(&issuer).send().unwrap();
        bond_accounts.push(acc);
    }

    let (issuer_config, _) = Pubkey::find_program_address(&[b"issuer", mint.as_ref()], &slotbook::id());
    let bond_vault = get_associated_token_address_with_program_id(&issuer_config, &bond_mint, &token::ID);

    // --- UC-1 init_issuer
    let params = IssuerParams {
        attestors: vec![att1.pubkey(), att2.pubkey()],
        quorum: 2,
        require_both_backends: false,
        resolver: resolver.pubkey(),
        notice_slots: NOTICE,
        finality_margin_slots: MARGIN,
        challenge_window_slots: WINDOW,
        resolve_timeout_slots: 100,
        hold_window_slots: 1_000,
        claim_expiry_slots: 10_000,
        attestor_bond: 10,
        challenger_bond: 10,
        resolver_fee: 1,
    };
    let init = |authority: Pubkey, params: IssuerParams| {
        ix(
            slotbook::accounts::InitIssuer {
                authority,
                mint,
                bond_mint,
                issuer_config,
                bond_vault,
                bond_token_program: token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::InitIssuer { params },
        )
    };
    // D22: someone who is not the mint authority cannot register the mint.
    let squatter = Keypair::new();
    svm.airdrop(&squatter.pubkey(), 1_000_000_000).unwrap();
    expect_err(send(&mut svm, &[init(squatter.pubkey(), params.clone())], &squatter, &[&squatter]), "NotMintAuthority");
    // D19: resolver_fee must be below both bonds.
    let mut bad = params.clone();
    bad.resolver_fee = 10;
    expect_err(send(&mut svm, &[init(issuer.pubkey(), bad)], &issuer, &[&issuer]), "ResolverFeeTooHigh");
    send(&mut svm, &[init(issuer.pubkey(), params.clone())], &issuer, &[&issuer]).unwrap();

    // --- UC-2 register_attestor (x2)
    let mut attestor_accounts = Vec::new();
    for (a, bond_acc) in [&att1, &att2].iter().zip(&bond_accounts) {
        let (attestor_account, _) = Pubkey::find_program_address(
            &[b"attestor", issuer_config.as_ref(), a.pubkey().as_ref()],
            &slotbook::id(),
        );
        send(
            &mut svm,
            &[ix(
                slotbook::accounts::RegisterAttestor {
                    attestor: a.pubkey(),
                    issuer_config,
                    attestor_account,
                    bond_mint,
                    bond_vault,
                    attestor_bond_account: *bond_acc,
                    bond_token_program: token::ID,
                    system_program: system_program::ID,
                },
                slotbook::instruction::RegisterAttestor { backend: slotbook::state::Backend::LedgerReplay },
            )],
            a,
            &[*a],
        )
        .unwrap();
        attestor_accounts.push(attestor_account);
    }
    assert_eq!(token_amount(&svm, &bond_vault), 20);

    // --- UC-3 declare_distribution
    svm.warp_to_slot(10);
    let (distribution, _) = Pubkey::find_program_address(
        &[b"distribution", issuer_config.as_ref(), &0u64.to_le_bytes()],
        &slotbook::id(),
    );
    let vault = get_associated_token_address_with_program_id(&distribution, &pay_mint, &token::ID);
    let declare = |record_slot: u64| {
        ix(
            slotbook::accounts::DeclareDistribution {
                authority: issuer.pubkey(),
                issuer_config,
                payment_mint: pay_mint,
                distribution,
                vault,
                issuer_payment_account: issuer_pay,
                payment_token_program: token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::DeclareDistribution {
                total: TOTAL,
                record_slot,
                salt_seed_hash: merkle::keccak(&[&SALT_SEED]),
            },
        )
    };
    // D14: the record slot must be announced at least notice_slots ahead.
    expect_err(send(&mut svm, &[declare(10 + NOTICE - 1)], &issuer, &[&issuer]), "RecordSlotTooSoon");
    // Griefing check: anyone can create the vault ATA before declare. The program
    // must still be able to declare (init_if_needed on the vault).
    let griefer = Keypair::new();
    svm.airdrop(&griefer.pubkey(), 1_000_000_000).unwrap();
    let pre = CreateAssociatedTokenAccountIdempotent::new(&mut svm, &griefer, &pay_mint)
        .owner(&distribution)
        .send()
        .unwrap();
    assert_eq!(pre, vault);
    let record_slot = 200;
    send(&mut svm, &[declare(record_slot)], &issuer, &[&issuer]).unwrap();
    assert_eq!(token_amount(&svm, &vault), TOTAL);

    // --- The register and its tree, built off-chain exactly as the spec says.
    let mut rows: Vec<(Pubkey, u64)> = holders.iter().map(|h| (h.kp.pubkey(), h.balance)).collect();
    rows.sort_by_key(|(o, _)| o.to_bytes());
    let salts: Vec<[u8; 32]> =
        rows.iter().map(|(o, _)| merkle::salt(&distribution.to_bytes(), &o.to_bytes(), &SALT_SEED)).collect();
    let leaves: Vec<[u8; 32]> =
        rows.iter().zip(&salts).map(|((o, b), s)| merkle::leaf_hash(&o.to_bytes(), *b, s)).collect();
    let tree = MerkleTree::from_leaves(leaves).unwrap();
    let register_total: u64 = rows.iter().map(|(_, b)| b).sum();

    // --- UC-4 commit_root (x2)
    let commit = |a: &Keypair, attestor_account: Pubkey| {
        let (commitment, _) = Pubkey::find_program_address(
            &[b"commitment", distribution.as_ref(), a.pubkey().as_ref()],
            &slotbook::id(),
        );
        ix(
            slotbook::accounts::CommitRoot {
                attestor: a.pubkey(),
                issuer_config,
                distribution,
                attestor_account,
                commitment,
                system_program: system_program::ID,
            },
            slotbook::instruction::CommitRoot {
                args: CommitArgs {
                    root: tree.root(),
                    manifest_hash: [1; 32],
                    register_total,
                    resolved_slot: record_slot,
                    first_covered_slot: 1,
                    rows: rows.len() as u32,
                    spec_version: SPEC_VERSION,
                },
            },
        )
    };
    // D6: not before record_slot + finality margin.
    svm.warp_to_slot(record_slot + MARGIN - 1);
    expect_err(send(&mut svm, &[commit(&att1, attestor_accounts[0])], &att1, &[&att1]), "FinalityMarginNotPassed");
    svm.warp_to_slot(record_slot + MARGIN);
    send(&mut svm, &[commit(&att1, attestor_accounts[0])], &att1, &[&att1]).unwrap();
    assert_eq!(read::<Distribution>(&svm, &distribution).state, DistributionState::Declared);
    send(&mut svm, &[commit(&att2, attestor_accounts[1])], &att2, &[&att2]).unwrap();
    let d: Distribution = read(&svm, &distribution);
    assert_eq!(d.state, DistributionState::Committed);
    assert_eq!(d.final_root, tree.root());
    assert_eq!(d.register_total, register_total);
    let window_end = d.window_end_slot;

    // --- claim helpers
    let claim = |h: &Holder, balance: u64, proof: Vec<[u8; 32]>| {
        let o = h.kp.pubkey();
        let i = rows.iter().position(|(r, _)| *r == o).unwrap();
        let (claim_receipt, _) =
            Pubkey::find_program_address(&[b"claim", distribution.as_ref(), o.as_ref()], &slotbook::id());
        let (pending, _) =
            Pubkey::find_program_address(&[b"pending", distribution.as_ref(), o.as_ref()], &slotbook::id());
        (
            ix(
                slotbook::accounts::Claim {
                    holder: o,
                    issuer_config,
                    distribution,
                    claim_receipt,
                    pending,
                    payment_mint: pay_mint,
                    vault,
                    holder_payment_account: h.payment_account,
                    holder_mint_account: h.mint_account,
                    payment_token_program: token::ID,
                    system_program: system_program::ID,
                },
                slotbook::instruction::Claim { balance, salt: salts[i], proof },
            ),
            claim_receipt,
            pending,
        )
    };
    let proof_of = |h: &Holder| {
        let i = rows.iter().position(|(r, _)| *r == h.kp.pubkey()).unwrap();
        tree.proof(i).unwrap().siblings
    };

    // Claims are blocked until finalize.
    let (early, _, _) = claim(&holders[0], holders[0].balance, proof_of(&holders[0]));
    expect_err(send(&mut svm, &[early], &holders[0].kp, &[&holders[0].kp]), "InvalidState");

    // --- UC-7 finalize (crank, no signer)
    let fin = ix(slotbook::accounts::Finalize { distribution }, slotbook::instruction::Finalize {});
    expect_err(send(&mut svm, &[fin.clone()], &issuer, &[&issuer]), "WindowNotElapsed");
    svm.warp_to_slot(window_end);
    send(&mut svm, &[fin], &issuer, &[&issuer]).unwrap();
    assert_eq!(read::<Distribution>(&svm, &distribution).state, DistributionState::Open);

    // --- UC-8 claim
    let a = &holders[0];
    // A wrong balance does not prove.
    let (bad_claim, _, _) = claim(a, a.balance + 1, proof_of(a));
    expect_err(send(&mut svm, &[bad_claim], &a.kp, &[&a.kp]), "InvalidProof");

    // Thawed holder with ImmutableOwner: paid now.
    let (good, receipt_a, _) = claim(a, a.balance, proof_of(a));
    send(&mut svm, &[good], &a.kp, &[&a.kp]).unwrap();
    let expected_a = TOTAL * a.balance / register_total;
    assert_eq!(token_amount(&svm, &a.payment_account), expected_a);
    let r: ClaimReceipt = read(&svm, &receipt_a);
    assert_eq!((r.payout, r.status), (expected_a, ClaimStatus::Paid));

    // A second claim by the same owner fails: the ClaimReceipt already exists.
    let (again, _, _) = claim(a, a.balance, proof_of(a));
    assert!(send(&mut svm, &[again], &a.kp, &[&a.kp]).is_err());

    // Frozen holder: held in Pending, not paid, not forfeited (D10, D11).
    let b = &holders[1];
    let (held, receipt_b, pending_b) = claim(b, b.balance, proof_of(b));
    send(&mut svm, &[held], &b.kp, &[&b.kp]).unwrap();
    let expected_b = TOTAL * b.balance / register_total;
    assert_eq!(token_amount(&svm, &b.payment_account), 0);
    let r: ClaimReceipt = read(&svm, &receipt_b);
    assert_eq!((r.payout, r.status), (expected_b, ClaimStatus::Pending));
    let p: Pending = read(&svm, &pending_b);
    assert_eq!((p.amount, p.owner, p.payer), (expected_b, b.kp.pubkey(), b.kp.pubkey()));

    // D20: books balance.
    let d: Distribution = read(&svm, &distribution);
    assert_eq!(d.claimed_total, expected_a);
    assert_eq!(d.pending_total, expected_b);
    assert!(d.claimed_total + d.pending_total + d.swept_total <= d.total);
    assert_eq!(token_amount(&svm, &vault), TOTAL - expected_a);
}
