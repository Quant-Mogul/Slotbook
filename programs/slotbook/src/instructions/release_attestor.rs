use anchor_lang::prelude::*;

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-12. Anyone frees an attestor's commitment once claims have opened.
#[derive(Accounts)]
pub struct ReleaseAttestor<'info> {
    #[account(
        mut,
        seeds = [DISTRIBUTION_SEED, distribution.issuer_config.as_ref(), &distribution.id.to_le_bytes()],
        bump = distribution.bump
    )]
    pub distribution: Box<Account<'info, Distribution>>,

    #[account(
        mut,
        seeds = [ATTESTOR_SEED, distribution.issuer_config.as_ref(), commitment.attestor.as_ref()],
        bump = attestor_account.bump
    )]
    pub attestor_account: Box<Account<'info, Attestor>>,

    #[account(
        mut,
        has_one = distribution,
        close = attestor,
        seeds = [COMMITMENT_SEED, distribution.key().as_ref(), commitment.attestor.as_ref()],
        bump = commitment.bump
    )]
    pub commitment: Box<Account<'info, Commitment>>,

    /// Receives the Commitment rent.
    /// CHECK: must equal commitment.attestor (address constraint).
    #[account(mut, address = commitment.attestor)]
    pub attestor: UncheckedAccount<'info>,
}

// TODO(UC-12, D8):
// - distribution state is Open or Closed
// - active_commitments -= 1, commitment_count -= 1; Commitment closes to the attestor
pub fn handle_release_attestor(_ctx: Context<ReleaseAttestor>) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
