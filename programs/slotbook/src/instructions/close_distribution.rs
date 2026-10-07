use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-11. The issuer empties and closes the vault. The Distribution stays, as Closed (D7).
#[derive(Accounts)]
pub struct CloseDistribution<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ SlotbookError::NotAuthority,
        seeds = [ISSUER_SEED, issuer_config.mint.as_ref()],
        bump = issuer_config.bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    #[account(
        mut,
        has_one = issuer_config,
        has_one = payment_mint,
        seeds = [DISTRIBUTION_SEED, issuer_config.key().as_ref(), &distribution.id.to_le_bytes()],
        bump = distribution.bump
    )]
    pub distribution: Box<Account<'info, Distribution>>,

    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = distribution,
        associated_token::token_program = payment_token_program
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = payment_mint,
        token::authority = authority,
        token::token_program = payment_token_program
    )]
    pub issuer_payment_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub payment_token_program: Interface<'info, TokenInterface>,
}

// TODO(UC-11, D7):
// - (Open, pending_total == 0, Clock.slot >= open_slot + claim_expiry_slots) or
//   (Declared, commitment_count == 0 or Clock.slot >= record_slot + claim_expiry_slots)
// - transfer the vault balance (dust included) to the issuer, close the vault
//   (Distribution PDA signs); state Closed; active_distributions -= 1
pub fn handle_close_distribution(_ctx: Context<CloseDistribution>) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
