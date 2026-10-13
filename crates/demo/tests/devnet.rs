//! Devnet tests: run the whole Slotbook lifecycle once against a live cluster
//! (devnet by default) with the deployed program, and assert each check from that run.
//! State checks read accounts and token balances after each step; refusal checks
//! simulate a transaction the program must refuse and match the program error.
//!
//!     cargo test -p demo --test devnet -- --ignored --test-threads=1
//!
//! Ignored by default because it spends devnet SOL (about 1 SOL) and takes several minutes.
//! SLOTBOOK_RPC_URL overrides the cluster, SLOTBOOK_KEYPAIR the issuer keypair
//! (default ~/.config/solana/id.json, needs at least 2 SOL). The full transaction log
//! with explorer links is written to target/devnet-test/DEMO_RUN.md.

use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::OnceLock,
};

use clap::Parser;
use demo::{execute, Args, Check};

static RUN: OnceLock<Result<Vec<Check>, String>> = OnceLock::new();

fn run() -> &'static Result<Vec<Check>, String> {
    RUN.get_or_init(|| {
        let url = std::env::var("SLOTBOOK_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".into());
        let out = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/devnet-test");
        let mut argv = vec![
            "slotbook-demo".to_string(),
            "--rpc-url".into(),
            url,
            "--out".into(),
            out.into(),
            "--notice-slots".into(),
            "60".into(),
            "--challenge-window-slots".into(),
            "100".into(),
            "--resolve-timeout-slots".into(),
            "100".into(),
            "--hold-window-slots".into(),
            "60".into(),
            "--claim-expiry-slots".into(),
            "200".into(),
        ];
        if let Ok(k) = std::env::var("SLOTBOOK_KEYPAIR") {
            argv.extend(["--keypair".into(), k]);
        }
        let args = Args::parse_from(argv);
        match catch_unwind(AssertUnwindSafe(|| execute(&args))) {
            Ok(Ok(checks)) => Ok(checks),
            Ok(Err(e)) => Err(format!("{e:#}")),
            Err(_) => Err("the run panicked".into()),
        }
    })
}

fn expect(name: &str) {
    let checks = run().as_ref().unwrap_or_else(|e| panic!("devnet run stopped: {e}"));
    let c = checks
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("check {name} did not run"));
    assert!(c.ok, "{name}: {}", c.detail);
}

macro_rules! devnet_tests {
    ($($name:ident),* $(,)?) => {
        $(
            #[test]
            #[ignore = "runs on devnet and spends SOL"]
            fn $name() {
                expect(stringify!($name));
            }
        )*

        /// Every check the run recorded has a test above, and all of them passed.
        #[test]
        #[ignore = "runs on devnet and spends SOL"]
        fn zz_every_check_listed_and_passed() {
            let listed = [$(stringify!($name)),*];
            let checks = run().as_ref().unwrap_or_else(|e| panic!("devnet run stopped: {e}"));
            for c in checks {
                assert!(listed.contains(&c.name), "check {} has no test", c.name);
                assert!(c.ok, "{}: {}", c.name, c.detail);
            }
            assert_eq!(checks.len(), listed.len());
        }
    };
}

devnet_tests!(
    // step 1: issuer, attestors, distribution
    issuer_registered,
    attestors_bonded,
    declare_rejects_short_notice,
    distribution_declared_and_funded,
    history_gives_expected_balances,
    // step 2: commit
    commit_before_finality_margin,
    commit_rejects_unknown_spec_version,
    quorum_commits_root,
    finalize_before_window,
    claim_before_finalize,
    // step 3: independent reproduction
    root_reproduced_independently,
    // step 4: rejected challenge
    challenge_opens_dispute,
    second_challenge_while_disputed,
    resolve_by_non_resolver,
    rejected_challenge_pays_resolver_and_attestors,
    // step 5: upheld challenge
    upheld_challenge_slashes_attestors,
    no_cancel_after_record_slot,
    corrected_root_committed_and_reproduced,
    // step 6: finalize and claim
    challenge_after_window,
    finalize_opens_claims,
    claim_with_inflated_balance,
    claim_with_someone_elses_proof,
    holder_paid_pro_rata,
    double_claim,
    withdraw_bond_with_live_commitment,
    // step 7: frozen holder, Pending, release
    frozen_holder_cannot_claim_through_side_account,
    frozen_holder_goes_to_pending,
    pending_released_after_thaw,
    // step 8: sweep, close, attestors exit
    release_while_frozen,
    sweep_before_hold_window,
    unclaimed_pending_swept_to_issuer,
    close_before_claim_expiry,
    distribution_closed,
    attestors_exit_with_bonds,
);
