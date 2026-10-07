use anchor_lang::prelude::*;

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-14. Replaces parameters when no distribution is active (D18).
#[derive(Accounts)]
pub struct UpdateIssuer<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ SlotbookError::NotAuthority,
        seeds = [ISSUER_SEED, issuer_config.mint.as_ref()],
        bump = issuer_config.bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,
}

// TODO(UC-14):
// - active_distributions == 0 (D18)
// - same validation as init_issuer; mint and bond_mint never change
pub fn handle_update_issuer(_ctx: Context<UpdateIssuer>, _params: IssuerParams) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
