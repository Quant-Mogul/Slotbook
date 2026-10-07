use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-9. Anyone pays a held claim once the holder's account is transferable.
#[derive(Accounts)]
pub struct ReleasePending<'info> {
    #[account(
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
        token::token_program = payment_token_program,
        constraint = holder_payment_account.owner == pending.owner @ SlotbookError::NotLeafOwner
    )]
    pub holder_payment_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = holder_mint_account.mint == issuer_config.mint @ SlotbookError::WrongMint,
        constraint = holder_mint_account.owner == pending.owner @ SlotbookError::NotLeafOwner
    )]
    pub holder_mint_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub payment_token_program: Interface<'info, TokenInterface>,
}

// TODO(UC-9):
// - Clock.slot < created_slot + hold_window_slots
// - holder_mint_account Initialized and has ImmutableOwner (D10)
// - transfer_checked amount from vault (Distribution PDA signs);
//   pending_total -= amount, claimed_total += amount; Pending closes to payer
pub fn handle_release_pending(_ctx: Context<ReleasePending>) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
