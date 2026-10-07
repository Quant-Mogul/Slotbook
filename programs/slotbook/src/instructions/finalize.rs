use anchor_lang::prelude::*;

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-7. Anyone opens claims once the challenge window has elapsed.
#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(
        mut,
        seeds = [DISTRIBUTION_SEED, distribution.issuer_config.as_ref(), &distribution.id.to_le_bytes()],
        bump = distribution.bump
    )]
    pub distribution: Box<Account<'info, Distribution>>,
}

/// UC-7. A crank: no signer beyond the fee payer.
pub fn handle_finalize(ctx: Context<Finalize>) -> Result<()> {
    let slot = Clock::get()?.slot;
    let d = &mut ctx.accounts.distribution;
    require!(d.state == DistributionState::Committed, SlotbookError::InvalidState);
    require!(slot >= d.window_end_slot, SlotbookError::WindowNotElapsed);
    d.state = DistributionState::Open;
    d.open_slot = slot;
    Ok(())
}
