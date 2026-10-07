use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::SlotbookError,
    state::*,
    utils::{apply_params, validate_params},
};

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

/// UC-14 (D18). Same validation as init_issuer; mint and bond_mint never change.
pub fn handle_update_issuer(ctx: Context<UpdateIssuer>, params: IssuerParams) -> Result<()> {
    let cfg = &mut ctx.accounts.issuer_config;
    require!(cfg.active_distributions == 0, SlotbookError::ActiveDistributions);
    validate_params(&params, &cfg.authority)?;
    apply_params(cfg, params);
    Ok(())
}
