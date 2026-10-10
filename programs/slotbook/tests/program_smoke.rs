//! Smoke tests for the program stubs: the program loads in LiteSVM, and the IDL
//! (which Kwesi's Codama client is generated from) has the full interface.
//! Run after `anchor build`, which writes target/deploy/slotbook.so and target/idl/slotbook.json.

use litesvm::LiteSVM;
use serde_json::Value;

const INSTRUCTIONS: [&str; 14] = [
    "challenge",
    "claim",
    "close_distribution",
    "commit_root",
    "declare_distribution",
    "finalize",
    "init_issuer",
    "register_attestor",
    "release_attestor",
    "release_pending",
    "resolve_challenge",
    "sweep_pending",
    "update_issuer",
    "withdraw_bond",
];

const ACCOUNTS: [&str; 7] = [
    "Attestor",
    "Challenge",
    "ClaimReceipt",
    "Commitment",
    "Distribution",
    "IssuerConfig",
    "Pending",
];

fn names(idl: &Value, key: &str) -> Vec<String> {
    let mut v: Vec<String> = idl[key]
        .as_array()
        .unwrap_or_else(|| panic!("IDL has no {key}"))
        .iter()
        .map(|x| x["name"].as_str().unwrap().to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn program_loads() {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/slotbook.so"));
    svm.add_program(slotbook::id(), bytes).unwrap();
}

#[test]
fn idl_has_full_interface() {
    let path = concat!(env!("CARGO_TARGET_TMPDIR"), "/../idl/slotbook.json");
    let idl: Value = serde_json::from_str(
        &std::fs::read_to_string(path).expect("run `anchor build` first"),
    )
    .unwrap();
    assert_eq!(names(&idl, "instructions"), INSTRUCTIONS);
    assert_eq!(names(&idl, "accounts"), ACCOUNTS);
    let errors = names(&idl, "errors");
    assert!(errors.contains(&"NotImplemented".to_string()));
}
