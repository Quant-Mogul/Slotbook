use anchor_lang::prelude::*;

use crate::constants::MAX_ATTESTORS;

/// Distribution lifecycle. See the brief, section 4.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum DistributionState {
    Declared,
    Committed,
    Disputed,
    Open,
    Closed,
}

/// Why a distribution is Disputed (D3): an owner challenge, or attestor roots that differ.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum DisputeKind {
    Owner,
    Conflict,
}

/// Reconstruction backend an attestor declares. Matches the manifest `backend` byte.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum Backend {
    LedgerReplay,
    StateArchive,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ClaimStatus {
    Paid,
    Pending,
}

/// Resolver ruling for resolve_challenge (D3, D4, D5, D17).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ruling {
    Upheld,
    Rejected,
    /// The root the resolver names as correct.
    Conflict { root: [u8; 32] },
    /// Anyone, after resolve_timeout_slots.
    Expired,
}

/// Parameters set at init_issuer and replaced by update_issuer (D18, D19).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub struct IssuerParams {
    pub attestors: Vec<Pubkey>,
    pub quorum: u8,
    pub require_both_backends: bool,
    pub resolver: Pubkey,
    pub notice_slots: u64,
    pub finality_margin_slots: u64,
    pub challenge_window_slots: u64,
    pub resolve_timeout_slots: u64,
    pub hold_window_slots: u64,
    pub claim_expiry_slots: u64,
    pub attestor_bond: u64,
    pub challenger_bond: u64,
    pub resolver_fee: u64,
}

/// Data an attestor commits (UC-4). Mirrors the manifest (spec section 11).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub struct CommitArgs {
    pub root: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub register_total: u64,
    pub resolved_slot: u64,
    pub first_covered_slot: u64,
    pub rows: u32,
    pub spec_version: u16,
}

/// ["issuer", mint]. Never closed (D18). Owns the bond vault.
#[account]
#[derive(InitSpace)]
pub struct IssuerConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub bond_mint: Pubkey,
    #[max_len(MAX_ATTESTORS)]
    pub attestors: Vec<Pubkey>,
    pub quorum: u8,
    pub require_both_backends: bool,
    pub resolver: Pubkey,
    pub notice_slots: u64,
    pub finality_margin_slots: u64,
    pub challenge_window_slots: u64,
    pub resolve_timeout_slots: u64,
    pub hold_window_slots: u64,
    pub claim_expiry_slots: u64,
    pub attestor_bond: u64,
    pub challenger_bond: u64,
    pub resolver_fee: u64,
    pub distribution_count: u64,
    pub active_distributions: u64,
    pub bump: u8,
}

/// ["attestor", issuer_config, attestor]. Closed by withdraw_bond.
#[account]
#[derive(InitSpace)]
pub struct Attestor {
    pub issuer_config: Pubkey,
    pub authority: Pubkey,
    pub bond_amount: u64,
    pub backend: Backend,
    pub active_commitments: u32,
    pub bump: u8,
}

/// ["distribution", issuer_config, id_u64_le]. Never closed; Closed is terminal (D7).
/// Owns the distribution vault.
#[account]
#[derive(InitSpace)]
pub struct Distribution {
    pub issuer_config: Pubkey,
    pub id: u64,
    pub payment_mint: Pubkey,
    pub record_slot: u64,
    pub total: u64,
    pub state: DistributionState,
    pub final_root: [u8; 32],
    pub register_total: u64,
    pub salt_seed_hash: [u8; 32],
    pub claimed_total: u64,
    pub pending_total: u64,
    pub swept_total: u64,
    pub declared_slot: u64,
    pub window_end_slot: u64,
    pub slots_remaining: u64,
    pub open_slot: u64,
    pub commitment_count: u32,
    pub challenge_count: u32,
    pub open_challenge_index: Option<u32>,
    pub dispute_kind: Option<DisputeKind>,
    pub dispute_opened_slot: u64,
    pub bump: u8,
}

/// ["commitment", distribution, attestor]. Closed at resolve or release_attestor (D8).
#[account]
#[derive(InitSpace)]
pub struct Commitment {
    pub distribution: Pubkey,
    pub attestor: Pubkey,
    pub root: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub backend: Backend,
    pub register_total: u64,
    pub resolved_slot: u64,
    pub first_covered_slot: u64,
    pub rows: u32,
    pub spec_version: u16,
    pub posted_slot: u64,
    pub bump: u8,
}

/// ["challenge", distribution, index_u32_le]. Closed at resolution, rent to challenger (D1).
#[account]
#[derive(InitSpace)]
pub struct Challenge {
    pub distribution: Pubkey,
    pub index: u32,
    pub challenger: Pubkey,
    pub root: [u8; 32],
    /// Pubkey::default() disputes register_total (D22).
    pub owner: Pubkey,
    pub claimed_balance: u64,
    pub bond: u64,
    pub opened_slot: u64,
    pub bump: u8,
}

/// ["claim", distribution, owner]. Never closed; blocks a second claim.
#[account]
#[derive(InitSpace)]
pub struct ClaimReceipt {
    pub distribution: Pubkey,
    pub owner: Pubkey,
    pub balance: u64,
    pub payout: u64,
    pub status: ClaimStatus,
    pub bump: u8,
}

/// ["pending", distribution, owner]. A frozen holder's payout, held (D10, D11).
#[account]
#[derive(InitSpace)]
pub struct Pending {
    pub distribution: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub created_slot: u64,
    pub payer: Pubkey,
    pub bump: u8,
}
