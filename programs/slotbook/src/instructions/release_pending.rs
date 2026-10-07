use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::*,
    error::SlotbookError,
    state::*,
    utils::{is_transferable, transfer_from_pda},
};

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

/// UC-9 (D10, D11). A crank: pays a held claim once the holder's account is transferable.
/// The ClaimReceipt keeps status Pending as a record of how the claim started.
pub fn handle_release_pending(ctx: Context<ReleasePending>) -> Result<()> {
    let slot = Clock::get()?.slot;
    let hold = ctx.accounts.issuer_config.hold_window_slots;
    let p = &ctx.accounts.pending;
    let end = p.created_slot.checked_add(hold).ok_or(SlotbookError::MathOverflow)?;
    require!(slot < end, SlotbookError::HoldWindowElapsed);
    require!(
        is_transferable(&ctx.accounts.holder_mint_account.to_account_info())?,
        SlotbookError::NotTransferable
    );
    let amount = p.amount;

    let d = &mut ctx.accounts.distribution;
    d.pending_total = d.pending_total.checked_sub(amount).ok_or(SlotbookError::MathOverflow)?;
    d.claimed_total = d.claimed_total.checked_add(amount).ok_or(SlotbookError::MathOverflow)?;

    let id = d.id.to_le_bytes();
    let seeds: &[&[u8]] = &[DISTRIBUTION_SEED, d.issuer_config.as_ref(), &id, &[d.bump]];
    transfer_from_pda(
        &ctx.accounts.payment_token_program.to_account_info(),
        &ctx.accounts.vault.to_account_info(),
        &ctx.accounts.payment_mint.to_account_info(),
        &ctx.accounts.holder_payment_account.to_account_info(),
        &d.to_account_info(),
        &[seeds],
        amount,
        ctx.accounts.payment_mint.decimals,
    )
    // Pending closes to its payer (close = payer).
}
