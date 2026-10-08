use anchor_lang::prelude::*;

#[constant]
pub const ISSUER_SEED: &[u8] = b"issuer";
#[constant]
pub const ATTESTOR_SEED: &[u8] = b"attestor";
#[constant]
pub const DISTRIBUTION_SEED: &[u8] = b"distribution";
#[constant]
pub const COMMITMENT_SEED: &[u8] = b"commitment";
#[constant]
pub const CHALLENGE_SEED: &[u8] = b"challenge";
#[constant]
pub const CLAIM_SEED: &[u8] = b"claim";
#[constant]
pub const PENDING_SEED: &[u8] = b"pending";

/// IssuerConfig.attestors holds at most this many keys.
pub const MAX_ATTESTORS: usize = 3;

/// Merkle proofs are bounded before the loop (D13, spec section 8).
#[constant]
pub const MAX_PROOF_DEPTH: u8 = 32;

/// docs/SNAPSHOT_SPEC.md v1.0.
#[constant]
pub const SPEC_VERSION: u16 = 100;

/// Token ACL program. The mint's freeze authority must be its ["MINT_CONFIG", mint] PDA (D22).
pub const TOKEN_ACL_PROGRAM_ID: Pubkey = pubkey!("TACLkU6CiCdkQN2MjoyDkVg2yAH9zkxiHDsiztQ52TP");
pub const TOKEN_ACL_MINT_CONFIG_SEED: &[u8] = b"MINT_CONFIG";
