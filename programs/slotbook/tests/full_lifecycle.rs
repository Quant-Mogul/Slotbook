//! The Demo Day lifecycle in LiteSVM, plus the Conflict and Expired dispute paths.
//! Run after `anchor build`.

mod common;

use common::*;
use slotbook::state::{
    Attestor as AttestorState, ClaimReceipt, ClaimStatus, Distribution, DistributionState, DisputeKind, IssuerConfig,
    Ruling,
};
use solana_keypair::Keypair;
use solana_signer::Signer;

fn new_challenger(env: &mut Env) -> (Keypair, anchor_lang::prelude::Pubkey) {
    let kp = Keypair::new();
    env.svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    let acc = env.token_account(env.bond_mint, kp.pubkey());
    env.mint_to(env.bond_mint, acc, CHALLENGER_BOND);
    (kp, acc)
}

fn dist(env: &Env) -> Distribution {
    env.read(&env.distribution)
}

fn bond_of(env: &Env, i: usize) -> AttestorState {
    env.read(&env.attestors[i].account)
}

/// Demo steps 1 to 8: a wrong-balance challenge is rejected, an omitted-owner challenge
/// is upheld (both attestors slashed, re-register, corrected commitment), then a paid
/// claim, a frozen claim released after a thaw, a frozen claim swept, close, release, withdraw.
#[test]
fn full_demo() {
    // Holders: A thawed, B frozen (thawed later), C frozen (stays frozen, swept).
    let mut env = Env::new(&[
        (500, HolderState::Initialized),
        (300, HolderState::Frozen),
        (200, HolderState::Frozen),
    ]);
    let (a, b, c) = (0, 1, 2);
    let bad_rows = env.rows(&[a, b]); // C deliberately omitted
    let good_rows = env.rows(&[a, b, c]);

    // Step 2: both attestors commit the same (wrong) root.
    env.warp(RECORD_SLOT + MARGIN);
    env.commit(0, &bad_rows).unwrap();
    env.commit(1, &bad_rows).unwrap();
    let d = dist(&env);
    assert_eq!(d.state, DistributionState::Committed);
    assert_eq!(d.final_root, env.tree(&bad_rows).root());

    // Step 4: a challenger disputes A with a wrong balance. The resolver rejects it.
    env.warp(RECORD_SLOT + MARGIN + 10);
    let (ch1, ch1_bond) = new_challenger(&mut env);
    let owner_a = env.holders[a].kp.pubkey();
    env.challenge(&ch1, ch1_bond, 0, owner_a, 999).unwrap();
    let d = dist(&env);
    assert_eq!((d.state, d.dispute_kind), (DistributionState::Disputed, Some(DisputeKind::Owner)));
    assert_eq!(d.slots_remaining, WINDOW - 10);
    // Claims and finalize are blocked while Disputed.
    assert!(env.finalize().is_err());

    let resolver = env.resolver.insecure_clone();
    let dispute = Some((0, ch1.pubkey(), ch1_bond));
    // Only the resolver rules (D4, D5).
    let stranger = Keypair::new();
    env.svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();
    expect_err(env.resolve(&stranger, Ruling::Rejected, dispute, &[0, 1]), "NotResolver");
    // Every live commitment must be handed in (D22).
    expect_err(env.resolve(&resolver, Ruling::Rejected, dispute, &[0]), "CommitmentSetMismatch");
    // A Conflict ruling does not fit an owner dispute.
    expect_err(
        env.resolve(&resolver, Ruling::Conflict { root: [0; 32] }, dispute, &[0, 1]),
        "InvalidRuling",
    );
    env.warp(RECORD_SLOT + MARGIN + 20);
    env.resolve(&resolver, Ruling::Rejected, dispute, &[0, 1]).unwrap();
    // Fee 1 to the resolver; 9 split as 4 + 4 into the attestors' bonds; remainder 1 to the resolver.
    assert_eq!(env.amount(&env.resolver_bond), 2);
    assert_eq!(bond_of(&env, 0).bond_amount, ATTESTOR_BOND + 4);
    assert_eq!(bond_of(&env, 1).bond_amount, ATTESTOR_BOND + 4);
    assert_eq!(env.amount(&ch1_bond), 0);
    assert!(!env.exists(&env.challenge_pda(0)), "Challenge closes at resolution (D1)");
    let d = dist(&env);
    assert_eq!(d.state, DistributionState::Committed);
    assert_eq!(d.window_end_slot, RECORD_SLOT + MARGIN + 20 + (WINDOW - 10), "window resumes (D5)");

    // Step 5: a second challenge names the omitted owner C with the true balance. Upheld.
    let (ch2, ch2_bond) = new_challenger(&mut env);
    let owner_c = env.holders[c].kp.pubkey();
    env.challenge(&ch2, ch2_bond, 1, owner_c, 200).unwrap();
    let dispute2 = Some((1, ch2.pubkey(), ch2_bond));
    env.resolve(&resolver, Ruling::Upheld, dispute2, &[0, 1]).unwrap();
    // Each attestor slashed exactly 10: 1 to the resolver, 9 to the challenger; bond back in full.
    assert_eq!(env.amount(&env.resolver_bond), 2 + 2);
    assert_eq!(env.amount(&ch2_bond), 9 + 9 + CHALLENGER_BOND);
    for i in 0..2 {
        let s = bond_of(&env, i);
        assert_eq!((s.bond_amount, s.active_commitments), (4, 0));
        assert!(!env.exists(&env.commitment(i)), "voided commitments are closed");
    }
    let d = dist(&env);
    assert_eq!((d.state, d.commitment_count, d.final_root), (DistributionState::Declared, 0, [0; 32]));

    // A slashed attestor cannot commit until it re-bonds (D8).
    expect_err(env.commit(0, &good_rows), "InsufficientBond");
    for i in 0..2 {
        env.withdraw(i).unwrap();
        assert!(!env.exists(&env.attestors[i].account));
        env.register(i).unwrap();
    }
    assert_eq!(env.amount(&env.attestors[0].bond_account), 3 * ATTESTOR_BOND - 10 + 4 - 10);
    env.commit(0, &good_rows).unwrap();
    env.commit(1, &good_rows).unwrap();
    let d = dist(&env);
    assert_eq!(d.state, DistributionState::Committed);
    assert_eq!(d.final_root, env.tree(&good_rows).root());
    assert_eq!(d.register_total, 1_000);

    // Step 6: window elapses, finalize, A claims and is paid.
    expect_err(env.claim(a, &good_rows), "InvalidState");
    env.warp(d.window_end_slot);
    env.finalize().unwrap();
    let open_slot = dist(&env).open_slot;
    env.claim(a, &good_rows).unwrap();
    assert_eq!(env.amount(&env.holders[a].payment_account), 500);

    // Step 7: B is frozen: held in Pending. The gate thaws B; release_pending pays.
    env.claim(b, &good_rows).unwrap();
    let r: ClaimReceipt = env.read(&env.receipt(b));
    assert_eq!(r.status, ClaimStatus::Pending);
    expect_err(env.release_pending(b), "NotTransferable");
    env.set_holder_state(b, HolderState::Initialized);
    env.release_pending(b).unwrap();
    assert_eq!(env.amount(&env.holders[b].payment_account), 300);
    assert!(!env.exists(&env.pending(b)), "Pending closes to its payer (D11)");

    // Step 8: C stays frozen. After the hold window the issuer sweeps C's share.
    env.claim(c, &good_rows).unwrap();
    expect_err(env.sweep_pending(c), "HoldWindowNotElapsed");
    let issuer_before = env.amount(&env.issuer_pay);
    env.warp(open_slot + HOLD + 1);
    expect_err(env.release_pending(c), "HoldWindowElapsed");
    env.sweep_pending(c).unwrap();
    assert_eq!(env.amount(&env.issuer_pay), issuer_before + 200);

    let d = dist(&env);
    assert_eq!((d.claimed_total, d.pending_total, d.swept_total), (800, 0, 200));

    // Close after claim expiry, then release and withdraw the attestors.
    expect_err(env.close_distribution(), "CannotClose");
    env.warp(open_slot + EXPIRY);
    env.close_distribution().unwrap();
    assert_eq!(dist(&env).state, DistributionState::Closed);
    assert!(!env.exists(&env.vault), "vault is closed");
    let cfg: IssuerConfig = env.read(&env.issuer_config);
    assert_eq!(cfg.active_distributions, 0);

    expect_err(env.withdraw(0), "LiveCommitments");
    for i in 0..2 {
        env.release_attestor(i).unwrap();
        env.withdraw(i).unwrap();
    }
    assert_eq!(dist(&env).commitment_count, 0);
    assert_eq!(env.amount(&env.bond_vault), 0, "every bond accounted for");
}

/// D3: roots differ before quorum. The resolver names the right root; the other attestor
/// is slashed (rest to the issuer). One survivor is below quorum, so the candidate stays
/// until the slashed attestor re-bonds and commits the right root.
#[test]
fn conflict_path() {
    let mut env = Env::new(&[(600, HolderState::Initialized), (400, HolderState::Initialized)]);
    let good = env.rows(&[0, 1]);
    let bad = env.rows(&[0]);
    env.warp(RECORD_SLOT + MARGIN);
    env.commit(0, &good).unwrap();
    env.commit(1, &bad).unwrap();
    let d = dist(&env);
    assert_eq!((d.state, d.dispute_kind), (DistributionState::Disputed, Some(DisputeKind::Conflict)));

    let resolver = env.resolver.insecure_clone();
    let good_root = env.tree(&good).root();
    expect_err(env.resolve(&resolver, Ruling::Upheld, None, &[0, 1]), "InvalidRuling");
    env.resolve(&resolver, Ruling::Conflict { root: good_root }, None, &[0, 1]).unwrap();
    assert_eq!(env.amount(&env.resolver_bond), RESOLVER_FEE);
    assert_eq!(env.amount(&env.issuer_bond), ATTESTOR_BOND - RESOLVER_FEE);
    assert_eq!(bond_of(&env, 1).bond_amount, 0);
    assert!(env.exists(&env.commitment(0)) && !env.exists(&env.commitment(1)));
    let d = dist(&env);
    assert_eq!((d.state, d.commitment_count, d.final_root), (DistributionState::Declared, 1, good_root));

    // The slashed attestor re-bonds and commits the named root: quorum, Committed.
    env.withdraw(1).unwrap();
    env.register(1).unwrap();
    env.commit(1, &good).unwrap();
    assert_eq!(dist(&env).state, DistributionState::Committed);
}

/// D17: the resolver never answers. After the timeout anyone rules Expired: the challenger
/// gets the bond back, commitments are voided without slashing, and attestors recommit.
#[test]
fn expired_path() {
    let mut env = Env::new(&[(700, HolderState::Initialized), (300, HolderState::Initialized)]);
    let rows = env.rows(&[0, 1]);
    env.warp(RECORD_SLOT + MARGIN);
    env.commit(0, &rows).unwrap();
    env.commit(1, &rows).unwrap();

    let (ch, ch_bond) = new_challenger(&mut env);
    let owner = env.holders[0].kp.pubkey();
    env.challenge(&ch, ch_bond, 0, owner, 1).unwrap();
    let opened = dist(&env).dispute_opened_slot;
    let dispute = Some((0, ch.pubkey(), ch_bond));

    let cranker = Keypair::new();
    env.svm.airdrop(&cranker.pubkey(), 1_000_000_000).unwrap();
    expect_err(env.resolve(&cranker, Ruling::Expired, dispute, &[0, 1]), "TimeoutNotReached");
    env.warp(opened + TIMEOUT);
    env.resolve(&cranker, Ruling::Expired, dispute, &[0, 1]).unwrap();

    assert_eq!(env.amount(&ch_bond), CHALLENGER_BOND);
    for i in 0..2 {
        let s = bond_of(&env, i);
        assert_eq!((s.bond_amount, s.active_commitments), (ATTESTOR_BOND, 0));
    }
    let d = dist(&env);
    assert_eq!((d.state, d.commitment_count), (DistributionState::Declared, 0));

    env.commit(0, &rows).unwrap();
    env.commit(1, &rows).unwrap();
    assert_eq!(dist(&env).state, DistributionState::Committed);
}
