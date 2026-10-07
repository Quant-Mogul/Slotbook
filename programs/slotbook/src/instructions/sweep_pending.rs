use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-10. After the hold window, the issuer takes back a still-held payout.
#[derive(Accounts)]
pub struct SweepPending<'info> {
    pub authority: Signer<'info>,

    #[account(
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

    #[account(
        mut,
        has_one = distribution,
        has_one = payer,
        close = payer,
        seeds = [PENDING_SEED, distribution.key().as_ref(), pending.owner.as_ref()],
        bump = pending.bump
    )]
    pub pending: Box<Account<'info, Pending>>,

    /// Receives the Pending rent (D11).
    /// CHECK: must equal pending.payer (has_one above).
    #[account(mut)]
    pub payer: UncheckedAccount<'info>,

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

// TODO(UC-10):
// - Clock.slot >= created_slot + hold_window_slots
// - transfer_checked amount from vault to the issuer (Distribution PDA signs);
//   pending_total -= amount, swept_total += amount; Pending closes to payer
pub fn handle_sweep_pending(_ctx: Context<SweepPending>) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
