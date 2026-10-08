//! Shared LiteSVM setup for the lifecycle tests. Run after `anchor build`.
#![allow(dead_code)]

use anchor_lang::{
    prelude::Pubkey,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_option::COption,
        program_pack::Pack,
        system_program,
    },
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id},
    token,
    token_2022::spl_token_2022::{
        self as t22,
        extension::{
            default_account_state::DefaultAccountState, immutable_owner::ImmutableOwner,
            permanent_delegate::PermanentDelegate,
            BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
        },
        state::{Account as T22Account, AccountState, Mint as T22Mint},
    },
    token_interface::spl_token_2022::state::Account as AnyTokenAccount,
};
use litesvm::LiteSVM;
use litesvm_token::{CreateAccount, CreateAssociatedTokenAccountIdempotent, CreateMint, MintTo};
use merkle::MerkleTree;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use slotbook::{
    state::{Backend, CommitArgs, IssuerParams, Ruling},
    SPEC_VERSION, TOKEN_ACL_MINT_CONFIG_SEED, TOKEN_ACL_PROGRAM_ID,
};
use solana_account::Account;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub use anchor_spl::token_2022::spl_token_2022::state::AccountState as HolderState;

pub const TOTAL: u64 = 1_000;
pub const SALT_SEED: [u8; 32] = [7; 32];
pub const NOTICE: u64 = 100;
pub const MARGIN: u64 = 32;
pub const WINDOW: u64 = 150;
pub const TIMEOUT: u64 = 100;
pub const HOLD: u64 = 1_000;
pub const EXPIRY: u64 = 10_000;
pub const ATTESTOR_BOND: u64 = 10;
pub const CHALLENGER_BOND: u64 = 10;
pub const RESOLVER_FEE: u64 = 1;
pub const DECLARE_SLOT: u64 = 10;
pub const RECORD_SLOT: u64 = 200;

pub fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &slotbook::id()).0
}

pub fn ix<A: ToAccountMetas, D: InstructionData>(accounts: A, data: D) -> Instruction {
    Instruction { program_id: slotbook::id(), accounts: accounts.to_account_metas(None), data: data.data() }
}

pub fn expect_err(r: Result<(), String>, needle: &str) {
    let e = r.expect_err(&format!("expected failure containing {needle}"));
    assert!(e.contains(needle), "expected {needle}, got:\n{e}");
}

fn mint_data(authority: Pubkey, freeze: Pubkey, supply: u64) -> Vec<u8> {
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

pub fn holder_data(mint: Pubkey, owner: Pubkey, amount: u64, state: AccountState) -> Vec<u8> {
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

pub struct Holder {
    pub kp: Keypair,
    pub balance: u64,
    pub mint_account: Pubkey,
    pub payment_account: Pubkey,
}

pub struct Attestor {
    pub kp: Keypair,
    pub bond_account: Pubkey,
    pub account: Pubkey,
}

pub struct Env {
    pub svm: LiteSVM,
    pub issuer: Keypair,
    pub resolver: Keypair,
    pub mint: Pubkey,
    pub bond_mint: Pubkey,
    pub pay_mint: Pubkey,
    pub issuer_config: Pubkey,
    pub bond_vault: Pubkey,
    pub issuer_pay: Pubkey,
    pub issuer_bond: Pubkey,
    pub resolver_bond: Pubkey,
    pub attestors: Vec<Attestor>,
    pub holders: Vec<Holder>,
    pub distribution: Pubkey,
    pub vault: Pubkey,
}

impl Env {
    /// init_issuer, two attestors registered, a distribution declared at DECLARE_SLOT
    /// with record slot RECORD_SLOT. Holders as given (balance, state at claim time).
    pub fn new(holders: &[(u64, AccountState)]) -> Env {
        Env::new_with(holders, 2, 2, false)
    }

    /// Like `new`, with `n` attestors, a quorum, and optionally the bond vault ATA
    /// created by a stranger before init_issuer (griefing check).
    pub fn new_with(holders: &[(u64, AccountState)], n: usize, quorum: u8, pre_create_bond_vault: bool) -> Env {
        let mut env = Env::base(holders, n);
        if pre_create_bond_vault {
            let griefer = Keypair::new();
            env.svm.airdrop(&griefer.pubkey(), 1_000_000_000).unwrap();
            let owner = env.issuer_config;
            let mint = env.bond_mint;
            let made = CreateAssociatedTokenAccountIdempotent::new(&mut env.svm, &griefer, &mint)
                .owner(&owner)
                .send()
                .unwrap();
            assert_eq!(made, env.bond_vault);
        }
        let init = env.init_ix(quorum, token::ID);
        env.send_by_issuer(&[init]).unwrap();
        for i in 0..n {
            env.register(i).unwrap();
        }
        env.svm.warp_to_slot(DECLARE_SLOT);
        let declare = env.declare_ix(env.pay_mint, token::ID, TOTAL, RECORD_SLOT);
        env.send_by_issuer(&[declare]).unwrap();
        env
    }

    /// Everything before init_issuer: mints, holders, funded accounts, `n` attestors.
    pub fn base(holders: &[(u64, AccountState)], n: usize) -> Env {
        let mut svm = LiteSVM::new();
        let program = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/slotbook.so"));
        svm.add_program(slotbook::id(), program).unwrap();

        let issuer = Keypair::new();
        let resolver = Keypair::new();
        svm.airdrop(&issuer.pubkey(), 100_000_000_000).unwrap();
        svm.airdrop(&resolver.pubkey(), 1_000_000_000).unwrap();

        let bond_mint = CreateMint::new(&mut svm, &issuer).authority(&issuer.pubkey()).decimals(0).send().unwrap();
        let pay_mint = CreateMint::new(&mut svm, &issuer).authority(&issuer.pubkey()).decimals(0).send().unwrap();

        let mint = Pubkey::new_unique();
        let (mint_config, _) =
            Pubkey::find_program_address(&[TOKEN_ACL_MINT_CONFIG_SEED, mint.as_ref()], &TOKEN_ACL_PROGRAM_ID);
        let supply = holders.iter().map(|h| h.0).sum();
        let mut env = Env {
            svm,
            issuer,
            resolver,
            mint,
            bond_mint,
            pay_mint,
            issuer_config: Pubkey::default(),
            bond_vault: Pubkey::default(),
            issuer_pay: Pubkey::default(),
            issuer_bond: Pubkey::default(),
            resolver_bond: Pubkey::default(),
            attestors: Vec::new(),
            holders: Vec::new(),
            distribution: Pubkey::default(),
            vault: Pubkey::default(),
        };
        env.set_t22(mint, mint_data(env.issuer.pubkey(), mint_config, supply));

        for (balance, state) in holders {
            let kp = Keypair::new();
            env.svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
            let mint_account = Pubkey::new_unique();
            env.set_t22(mint_account, holder_data(mint, kp.pubkey(), *balance, *state));
            let payment_account = env.token_account(pay_mint, kp.pubkey());
            env.holders.push(Holder { kp, balance: *balance, mint_account, payment_account });
        }

        env.issuer_pay = env.token_account(pay_mint, env.issuer.pubkey());
        env.mint_to(pay_mint, env.issuer_pay, TOTAL);
        env.issuer_bond = env.token_account(bond_mint, env.issuer.pubkey());
        env.resolver_bond = env.token_account(bond_mint, env.resolver.pubkey());

        env.issuer_config = pda(&[b"issuer", mint.as_ref()]);
        env.bond_vault = get_associated_token_address_with_program_id(&env.issuer_config, &bond_mint, &token::ID);
        env.distribution = pda(&[b"distribution", env.issuer_config.as_ref(), &0u64.to_le_bytes()]);
        env.vault = get_associated_token_address_with_program_id(&env.distribution, &pay_mint, &token::ID);

        for _ in 0..n {
            let kp = Keypair::new();
            env.svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
            let bond_account = env.token_account(bond_mint, kp.pubkey());
            env.mint_to(bond_mint, bond_account, 3 * ATTESTOR_BOND);
            let account = pda(&[b"attestor", env.issuer_config.as_ref(), kp.pubkey().as_ref()]);
            env.attestors.push(Attestor { kp, bond_account, account });
        }
        env
    }

    pub fn params(&self, quorum: u8) -> IssuerParams {
        IssuerParams {
            attestors: self.attestors.iter().map(|a| a.kp.pubkey()).collect(),
            quorum,
            require_both_backends: false,
            resolver: self.resolver.pubkey(),
            notice_slots: NOTICE,
            finality_margin_slots: MARGIN,
            challenge_window_slots: WINDOW,
            resolve_timeout_slots: TIMEOUT,
            hold_window_slots: HOLD,
            claim_expiry_slots: EXPIRY,
            attestor_bond: ATTESTOR_BOND,
            challenger_bond: CHALLENGER_BOND,
            resolver_fee: RESOLVER_FEE,
        }
    }

    /// init_issuer with this env's bond mint; the bond vault is the ATA under `bond_program`.
    pub fn init_ix(&self, quorum: u8, bond_program: Pubkey) -> Instruction {
        let bond_vault = get_associated_token_address_with_program_id(&self.issuer_config, &self.bond_mint, &bond_program);
        ix(
            slotbook::accounts::InitIssuer {
                authority: self.issuer.pubkey(),
                mint: self.mint,
                bond_mint: self.bond_mint,
                issuer_config: self.issuer_config,
                bond_vault,
                bond_token_program: bond_program,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::InitIssuer { params: self.params(quorum) },
        )
    }

    /// declare_distribution #0 paid in `pay_mint` (vault is its ATA under `pay_program`).
    pub fn declare_ix(&self, pay_mint: Pubkey, pay_program: Pubkey, total: u64, record_slot: u64) -> Instruction {
        let vault = get_associated_token_address_with_program_id(&self.distribution, &pay_mint, &pay_program);
        let issuer_payment_account =
            get_associated_token_address_with_program_id(&self.issuer.pubkey(), &pay_mint, &pay_program);
        let issuer_payment_account = if pay_mint == self.pay_mint { self.issuer_pay } else { issuer_payment_account };
        ix(
            slotbook::accounts::DeclareDistribution {
                authority: self.issuer.pubkey(),
                issuer_config: self.issuer_config,
                payment_mint: pay_mint,
                distribution: self.distribution,
                vault,
                issuer_payment_account,
                payment_token_program: pay_program,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::DeclareDistribution {
                total,
                record_slot,
                salt_seed_hash: merkle::keccak(&[&SALT_SEED]),
            },
        )
    }

    /// A Token-2022 mint (decimals 0) carrying PermanentDelegate, for D21 checks.
    pub fn t22_mint_with_permanent_delegate(&mut self, delegate: Pubkey) -> Pubkey {
        let len = ExtensionType::try_calculate_account_len::<T22Mint>(&[ExtensionType::PermanentDelegate]).unwrap();
        let mut data = vec![0u8; len];
        let mut s = StateWithExtensionsMut::<T22Mint>::unpack_uninitialized(&mut data).unwrap();
        s.init_extension::<PermanentDelegate>(true).unwrap().delegate =
            OptionalNonZeroPubkey::try_from(Some(delegate)).unwrap();
        s.base = T22Mint {
            mint_authority: COption::Some(self.issuer.pubkey()),
            supply: 0,
            decimals: 0,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        s.pack_base();
        s.init_account_type().unwrap();
        let address = Pubkey::new_unique();
        self.set_t22(address, data);
        address
    }

    // ---------- plumbing

    pub fn send(&mut self, ixs: &[Instruction], payer: &Keypair, signers: &[&Keypair]) -> Result<(), String> {
        self.svm.expire_blockhash();
        let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &self.svm.latest_blockhash());
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
        self.svm
            .send_transaction(tx)
            .map(|_| ())
            .map_err(|e| format!("{:?}\n{}", e.err, e.meta.logs.join("\n")))
    }

    pub fn send_by_issuer(&mut self, ixs: &[Instruction]) -> Result<(), String> {
        let issuer = self.issuer.insecure_clone();
        self.send(ixs, &issuer, &[&issuer])
    }

    pub fn set_t22(&mut self, address: Pubkey, data: Vec<u8>) {
        let lamports = self.svm.minimum_balance_for_rent_exemption(data.len());
        self.svm
            .set_account(address, Account { lamports, data, owner: t22::ID, executable: false, rent_epoch: 0 })
            .unwrap();
    }

    pub fn token_account(&mut self, mint: Pubkey, owner: Pubkey) -> Pubkey {
        let issuer = self.issuer.insecure_clone();
        CreateAccount::new(&mut self.svm, &issuer, &mint).owner(&owner).send().unwrap()
    }

    pub fn mint_to(&mut self, mint: Pubkey, to: Pubkey, amount: u64) {
        let issuer = self.issuer.insecure_clone();
        MintTo::new(&mut self.svm, &issuer, &mint, &to, amount).owner(&issuer).send().unwrap();
    }

    pub fn amount(&self, token_account: &Pubkey) -> u64 {
        let acc = self.svm.get_account(token_account).unwrap();
        AnyTokenAccount::unpack(&acc.data[..165]).unwrap().amount
    }

    pub fn exists(&self, address: &Pubkey) -> bool {
        self.svm.get_account(address).map(|a| a.lamports > 0).unwrap_or(false)
    }

    pub fn read<T: AccountDeserialize>(&self, address: &Pubkey) -> T {
        T::try_deserialize(&mut self.svm.get_account(address).unwrap().data.as_slice()).unwrap()
    }

    pub fn warp(&mut self, slot: u64) {
        self.svm.warp_to_slot(slot);
    }

    /// Sets a holder's permissioned-mint account state (the gate thawing or freezing it).
    pub fn set_holder_state(&mut self, i: usize, state: AccountState) {
        let h = &self.holders[i];
        let data = holder_data(self.mint, h.kp.pubkey(), h.balance, state);
        let addr = h.mint_account;
        self.set_t22(addr, data);
    }

    // ---------- the register

    /// Rows sorted by owner bytes (spec 3.5), from (holder index) list.
    pub fn rows(&self, included: &[usize]) -> Vec<(Pubkey, u64)> {
        let mut rows: Vec<(Pubkey, u64)> =
            included.iter().map(|&i| (self.holders[i].kp.pubkey(), self.holders[i].balance)).collect();
        rows.sort_by_key(|(o, _)| o.to_bytes());
        rows
    }

    pub fn salt(&self, owner: &Pubkey) -> [u8; 32] {
        merkle::salt(&self.distribution.to_bytes(), &owner.to_bytes(), &SALT_SEED)
    }

    pub fn tree(&self, rows: &[(Pubkey, u64)]) -> MerkleTree {
        let leaves =
            rows.iter().map(|(o, b)| merkle::leaf_hash(&o.to_bytes(), *b, &self.salt(o))).collect();
        MerkleTree::from_leaves(leaves).unwrap()
    }

    // ---------- instructions

    pub fn register(&mut self, i: usize) -> Result<(), String> {
        let a = &self.attestors[i];
        let kp = a.kp.insecure_clone();
        let ixn = ix(
            slotbook::accounts::RegisterAttestor {
                attestor: kp.pubkey(),
                issuer_config: self.issuer_config,
                attestor_account: a.account,
                bond_mint: self.bond_mint,
                bond_vault: self.bond_vault,
                attestor_bond_account: a.bond_account,
                bond_token_program: token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::RegisterAttestor { backend: Backend::LedgerReplay },
        );
        self.send(&[ixn], &kp, &[&kp])
    }

    pub fn commitment(&self, i: usize) -> Pubkey {
        pda(&[b"commitment", self.distribution.as_ref(), self.attestors[i].kp.pubkey().as_ref()])
    }

    pub fn commit(&mut self, i: usize, rows: &[(Pubkey, u64)]) -> Result<(), String> {
        let tree = self.tree(rows);
        let kp = self.attestors[i].kp.insecure_clone();
        let ixn = ix(
            slotbook::accounts::CommitRoot {
                attestor: kp.pubkey(),
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                attestor_account: self.attestors[i].account,
                commitment: self.commitment(i),
                system_program: system_program::ID,
            },
            slotbook::instruction::CommitRoot {
                args: CommitArgs {
                    root: tree.root(),
                    manifest_hash: [1; 32],
                    register_total: rows.iter().map(|r| r.1).sum(),
                    resolved_slot: RECORD_SLOT,
                    first_covered_slot: 1,
                    rows: rows.len() as u32,
                    spec_version: SPEC_VERSION,
                },
            },
        );
        self.send(&[ixn], &kp, &[&kp])
    }

    pub fn challenge_pda(&self, index: u32) -> Pubkey {
        pda(&[b"challenge", self.distribution.as_ref(), &index.to_le_bytes()])
    }

    pub fn challenge(
        &mut self,
        challenger: &Keypair,
        challenger_bond: Pubkey,
        index: u32,
        owner: Pubkey,
        claimed_balance: u64,
    ) -> Result<(), String> {
        let ixn = ix(
            slotbook::accounts::CreateChallenge {
                challenger: challenger.pubkey(),
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                challenge: self.challenge_pda(index),
                bond_mint: self.bond_mint,
                bond_vault: self.bond_vault,
                challenger_bond_account: challenger_bond,
                bond_token_program: token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::Challenge { owner, claimed_balance },
        );
        self.send(&[ixn], challenger, &[challenger])
    }

    /// `owner_dispute`: Some((challenge index, challenger, challenger bond account)).
    /// `pairs`: which attestors' (Commitment, Attestor) go in remaining accounts.
    pub fn resolve(
        &mut self,
        signer: &Keypair,
        ruling: Ruling,
        owner_dispute: Option<(u32, Pubkey, Pubkey)>,
        pairs: &[usize],
    ) -> Result<(), String> {
        let mut ixn = ix(
            slotbook::accounts::ResolveChallenge {
                signer: signer.pubkey(),
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                challenge: owner_dispute.map(|(i, _, _)| self.challenge_pda(i)),
                challenger: owner_dispute.map(|(_, c, _)| c),
                bond_mint: self.bond_mint,
                bond_vault: self.bond_vault,
                resolver_bond_account: self.resolver_bond,
                challenger_bond_account: owner_dispute.map(|(_, _, b)| b),
                issuer_bond_account: self.issuer_bond,
                bond_token_program: token::ID,
            },
            slotbook::instruction::ResolveChallenge { ruling },
        );
        for &i in pairs {
            ixn.accounts.push(AccountMeta::new(self.commitment(i), false));
            ixn.accounts.push(AccountMeta::new(self.attestors[i].account, false));
        }
        self.send(&[ixn], signer, &[signer])
    }

    pub fn finalize(&mut self) -> Result<(), String> {
        let ixn = ix(
            slotbook::accounts::Finalize { distribution: self.distribution },
            slotbook::instruction::Finalize {},
        );
        self.send_by_issuer(&[ixn])
    }

    pub fn receipt(&self, i: usize) -> Pubkey {
        pda(&[b"claim", self.distribution.as_ref(), self.holders[i].kp.pubkey().as_ref()])
    }

    pub fn pending(&self, i: usize) -> Pubkey {
        pda(&[b"pending", self.distribution.as_ref(), self.holders[i].kp.pubkey().as_ref()])
    }

    pub fn claim(&mut self, i: usize, rows: &[(Pubkey, u64)]) -> Result<(), String> {
        let h = &self.holders[i];
        let o = h.kp.pubkey();
        let idx = rows.iter().position(|(r, _)| *r == o).expect("holder not in rows");
        let proof = self.tree(rows).proof(idx).unwrap().siblings;
        let kp = h.kp.insecure_clone();
        let ixn = ix(
            slotbook::accounts::Claim {
                holder: o,
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                claim_receipt: self.receipt(i),
                pending: self.pending(i),
                payment_mint: self.pay_mint,
                vault: self.vault,
                holder_payment_account: h.payment_account,
                holder_mint_account: h.mint_account,
                payment_token_program: token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::Claim { balance: rows[idx].1, salt: self.salt(&o), proof },
        );
        self.send(&[ixn], &kp, &[&kp])
    }

    pub fn release_pending(&mut self, i: usize) -> Result<(), String> {
        let h = &self.holders[i];
        let ixn = ix(
            slotbook::accounts::ReleasePending {
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                pending: self.pending(i),
                payer: h.kp.pubkey(),
                payment_mint: self.pay_mint,
                vault: self.vault,
                holder_payment_account: h.payment_account,
                holder_mint_account: h.mint_account,
                payment_token_program: token::ID,
            },
            slotbook::instruction::ReleasePending {},
        );
        self.send_by_issuer(&[ixn])
    }

    /// release_pending paying into an arbitrary account (to test the owner check).
    pub fn release_pending_to(&mut self, i: usize, payment_account: Pubkey) -> Result<(), String> {
        let h = &self.holders[i];
        let ixn = ix(
            slotbook::accounts::ReleasePending {
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                pending: self.pending(i),
                payer: h.kp.pubkey(),
                payment_mint: self.pay_mint,
                vault: self.vault,
                holder_payment_account: payment_account,
                holder_mint_account: h.mint_account,
                payment_token_program: token::ID,
            },
            slotbook::instruction::ReleasePending {},
        );
        self.send_by_issuer(&[ixn])
    }

    pub fn sweep_pending(&mut self, i: usize) -> Result<(), String> {
        let ixn = ix(
            slotbook::accounts::SweepPending {
                authority: self.issuer.pubkey(),
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                pending: self.pending(i),
                payer: self.holders[i].kp.pubkey(),
                payment_mint: self.pay_mint,
                vault: self.vault,
                issuer_payment_account: self.issuer_pay,
                payment_token_program: token::ID,
            },
            slotbook::instruction::SweepPending {},
        );
        self.send_by_issuer(&[ixn])
    }

    pub fn close_distribution(&mut self) -> Result<(), String> {
        let ixn = ix(
            slotbook::accounts::CloseDistribution {
                authority: self.issuer.pubkey(),
                issuer_config: self.issuer_config,
                distribution: self.distribution,
                payment_mint: self.pay_mint,
                vault: self.vault,
                issuer_payment_account: self.issuer_pay,
                payment_token_program: token::ID,
            },
            slotbook::instruction::CloseDistribution {},
        );
        self.send_by_issuer(&[ixn])
    }

    pub fn release_attestor(&mut self, i: usize) -> Result<(), String> {
        let ixn = ix(
            slotbook::accounts::ReleaseAttestor {
                distribution: self.distribution,
                attestor_account: self.attestors[i].account,
                commitment: self.commitment(i),
            },
            slotbook::instruction::ReleaseAttestor {},
        );
        self.send_by_issuer(&[ixn])
    }

    pub fn withdraw(&mut self, i: usize) -> Result<(), String> {
        let a = &self.attestors[i];
        let kp = a.kp.insecure_clone();
        let ixn = ix(
            slotbook::accounts::WithdrawBond {
                attestor: kp.pubkey(),
                issuer_config: self.issuer_config,
                attestor_account: a.account,
                bond_mint: self.bond_mint,
                bond_vault: self.bond_vault,
                attestor_bond_account: a.bond_account,
                bond_token_program: token::ID,
            },
            slotbook::instruction::WithdrawBond {},
        );
        self.send(&[ixn], &kp, &[&kp])
    }
}
