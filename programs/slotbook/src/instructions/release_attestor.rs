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
        close = attestor_account,
        seeds = [COMMITMENT_SEED, distribution.key().as_ref(), commitment.attestor.as_ref()],
        bump = commitment.bump
    )]
    /// Closes to the Attestor account, like a commitment voided in resolve_challenge,
    /// so every commitment's rent reaches the attestor the same way: at withdraw_bond.
    pub commitment: Box<Account<'info, Commitment>>,
}

/// UC-12 (D8). A crank: frees an attestor's commitment once claims have opened.
pub fn handle_release_attestor(ctx: Context<ReleaseAttestor>) -> Result<()> {
    let d = &mut ctx.accounts.distribution;
    require!(
        matches!(d.state, DistributionState::Open | DistributionState::Closed),
        SlotbookError::InvalidState
    );
    let a = &mut ctx.accounts.attestor_account;
    a.active_commitments = a.active_commitments.checked_sub(1).ok_or(SlotbookError::MathOverflow)?;
    d.commitment_count = d.commitment_count.checked_sub(1).ok_or(SlotbookError::MathOverflow)?;
    // The Commitment closes to the Attestor account (close = attestor_account).
    Ok(())
}
