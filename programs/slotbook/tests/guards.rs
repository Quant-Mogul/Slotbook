//! One test per guard that the lifecycle tests do not reach. Run after `anchor build`.

mod common;

use anchor_lang::{prelude::Pubkey, solana_program::system_program};
use anchor_spl::token_2022;
use common::*;
use litesvm_token::CreateAssociatedTokenAccountIdempotent;
use slotbook::state::{Distribution, DistributionState, IssuerConfig, Pending, Ruling};
use solana_keypair::Keypair;
use solana_signer::Signer;

fn new_challenger(env: &mut Env) -> (Keypair, Pubkey) {
    let kp = Keypair::new();
    env.svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    let acc = env.token_account(env.bond_mint, kp.pubkey());
    env.mint_to(env.bond_mint, acc, CHALLENGER_BOND);
    (kp, acc)
}

fn committed(holders: &[(u64, HolderState)]) -> (Env, Vec<(Pubkey, u64)>) {
    let mut env = Env::new(holders);
    let idx: Vec<usize> = (0..holders.len()).collect();
    let rows = env.rows(&idx);
    env.warp(RECORD_SLOT + MARGIN);
    env.commit(0, &rows).unwrap();
    env.commit(1, &rows).unwrap();
    (env, rows)
}

/// D21 for the bond vault: a bond mint with PermanentDelegate would let its delegate
/// drain attestor and challenger bonds.
#[test]
fn bond_mint_with_permanent_delegate_is_refused() {
    let mut env = Env::base(&[(100, HolderState::Initialized)], 2);
    let delegate = env.issuer.pubkey();
    env.bond_mint = env.t22_mint_with_permanent_delegate(delegate);
    let init = env.init_ix(2, token_2022::ID);
    expect_err(env.send_by_issuer(&[init]), "UnsupportedBondMint");
}

/// D21 for the distribution vault.
#[test]
fn payment_mint_with_permanent_delegate_is_refused() {
    let mut env = Env::base(&[(100, HolderState::Initialized)], 2);
    let init = env.init_ix(2, anchor_spl::token::ID);
    env.send_by_issuer(&[init]).unwrap();
    let delegate = env.issuer.pubkey();
    let bad_mint = env.t22_mint_with_permanent_delegate(delegate);
    let issuer = env.issuer.insecure_clone();
    CreateAssociatedTokenAccountIdempotent::new(&mut env.svm, &issuer, &bad_mint)
        .owner(&issuer.pubkey())
        .token_program_id(&token_2022::ID)
        .send()
        .unwrap();
    env.warp(DECLARE_SLOT);
    let declare = env.declare_ix(bad_mint, token_2022::ID, TOTAL, RECORD_SLOT);
    expect_err(env.send_by_issuer(&[declare]), "UnsupportedPaymentMint");
}

/// Anyone can create the bond vault ATA first; init_issuer must still work.
#[test]
fn bond_vault_pre_created_by_a_stranger() {
    let env = Env::new_with(&[(100, HolderState::Initialized)], 2, 2, true);
    let cfg: IssuerConfig = env.read(&env.issuer_config);
    assert_eq!(cfg.distribution_count, 1);
}

/// D2: after quorum, a late commitment must match the final root and total.
#[test]
fn late_commit_must_match() {
    let mut env = Env::new_with(&[(600, HolderState::Initialized), (400, HolderState::Initialized)], 3, 2, false);
    let good = env.rows(&[0, 1]);
    let bad = env.rows(&[0]);
    env.warp(RECORD_SLOT + MARGIN);
    env.commit(0, &good).unwrap();
    env.commit(1, &good).unwrap();
    expect_err(env.commit(2, &bad), "RootMismatch");
    env.commit(2, &good).unwrap();
    let d: Distribution = env.read(&env.distribution);
    assert_eq!((d.state, d.commitment_count), (DistributionState::Committed, 3));
}

/// UC-5: no challenge once the window has closed.
#[test]
fn challenge_after_window_is_refused() {
    let (mut env, _) = committed(&[(1_000, HolderState::Initialized)]);
    let end = env.read::<Distribution>(&env.distribution).window_end_slot;
    env.warp(end);
    let (ch, bond) = new_challenger(&mut env);
    let owner = env.holders[0].kp.pubkey();
    expect_err(env.challenge(&ch, bond, 0, owner, 1), "WindowClosed");
}

/// UC-6: the resolution must name the real challenger, and every commitment once.
#[test]
fn resolve_binds_the_challenge_and_each_commitment_once() {
    let (mut env, _) = committed(&[(1_000, HolderState::Initialized)]);
    let (ch, bond) = new_challenger(&mut env);
    let owner = env.holders[0].kp.pubkey();
    env.challenge(&ch, bond, 0, owner, 1).unwrap();
    let resolver = env.resolver.insecure_clone();

    let impostor = Keypair::new().pubkey();
    expect_err(env.resolve(&resolver, Ruling::Rejected, Some((0, impostor, bond)), &[0, 1]), "WrongChallenge");
    expect_err(
        env.resolve(&resolver, Ruling::Rejected, Some((0, ch.pubkey(), bond)), &[0, 0]),
        "CommitmentSetMismatch",
    );
    env.resolve(&resolver, Ruling::Rejected, Some((0, ch.pubkey(), bond)), &[0, 1]).unwrap();
}

/// D22: owner = Pubkey::default() disputes register_total. On-chain it is an ordinary
/// owner challenge; the resolver recomputes the total off-chain.
#[test]
fn register_total_challenge() {
    let (mut env, _) = committed(&[(1_000, HolderState::Initialized)]);
    let (ch, bond) = new_challenger(&mut env);
    env.challenge(&ch, bond, 0, Pubkey::default(), 999).unwrap();
    let resolver = env.resolver.insecure_clone();
    env.resolve(&resolver, Ruling::Rejected, Some((0, ch.pubkey(), bond)), &[0, 1]).unwrap();
    assert_eq!(env.read::<Distribution>(&env.distribution).state, DistributionState::Committed);
}

/// A stranger sends lamports to a frozen holder's Pending address first. The claim must
/// still create Pending (create_pda_account tops up, allocates and assigns).
#[test]
fn pre_funded_pending_address() {
    let (mut env, rows) = committed(&[(1_000, HolderState::Frozen)]);
    let end = env.read::<Distribution>(&env.distribution).window_end_slot;
    env.warp(end);
    env.finalize().unwrap();
    let pending = env.pending(0);
    env.svm.airdrop(&pending, 1_000).unwrap();
    let before = env.svm.get_account(&pending).unwrap();
    assert_eq!((before.owner, before.data.len()), (system_program::ID, 0));
    env.claim(0, &rows).unwrap();
    let p: Pending = env.read(&pending);
    assert_eq!(p.amount, TOTAL);
}

/// UC-9: a held payout only goes to an account the leaf owner owns.
#[test]
fn release_only_to_the_owner() {
    let (mut env, rows) = committed(&[(600, HolderState::Frozen), (400, HolderState::Initialized)]);
    let end = env.read::<Distribution>(&env.distribution).window_end_slot;
    env.warp(end);
    env.finalize().unwrap();
    env.claim(0, &rows).unwrap();
    env.set_holder_state(0, HolderState::Initialized);
    let someone_else = env.holders[1].payment_account;
    expect_err(env.release_pending_to(0, someone_else), "NotLeafOwner");
    env.release_pending(0).unwrap();
}

/// D7: a Declared distribution with no commitment can be closed at once, refunding the
/// issuer; then update_issuer is allowed again (D18).
#[test]
fn close_declared_then_update_issuer() {
    let mut env = Env::new(&[(1_000, HolderState::Initialized)]);
    let update = |env: &Env| {
        ix(
            slotbook::accounts::UpdateIssuer { authority: env.issuer.pubkey(), issuer_config: env.issuer_config },
            slotbook::instruction::UpdateIssuer { params: env.params(1) },
        )
    };
    expect_err(env.send_by_issuer(&[update(&env)]), "ActiveDistributions");
    let before = env.amount(&env.issuer_pay);
    env.close_distribution().unwrap();
    assert_eq!(env.amount(&env.issuer_pay), before + TOTAL);
    env.send_by_issuer(&[update(&env)]).unwrap();
    assert_eq!(env.read::<IssuerConfig>(&env.issuer_config).quorum, 1);
}
