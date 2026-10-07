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

// TODO(UC-7):
// - state is Committed and Clock.slot >= window_end_slot
// - state Open, open_slot = slot
pub fn handle_finalize(_ctx: Context<Finalize>) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
