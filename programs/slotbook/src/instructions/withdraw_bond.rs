use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*, utils::transfer_from_pda};

/// UC-13. An attestor with no live commitments takes its bond back and deregisters.
#[derive(Accounts)]
pub struct WithdrawBond<'info> {
    #[account(mut)]
    pub attestor: Signer<'info>,

    #[account(
        seeds = [ISSUER_SEED, issuer_config.mint.as_ref()],
        bump = issuer_config.bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    #[account(
        mut,
        close = attestor,
        constraint = attestor_account.authority == attestor.key() @ SlotbookError::NotAnAttestor,
        seeds = [ATTESTOR_SEED, issuer_config.key().as_ref(), attestor.key().as_ref()],
        bump = attestor_account.bump
    )]
    pub attestor_account: Box<Account<'info, Attestor>>,

    #[account(address = issuer_config.bond_mint)]
    pub bond_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = bond_mint,
        associated_token::authority = issuer_config,
        associated_token::token_program = bond_token_program
    )]
    pub bond_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = bond_mint,
        token::authority = attestor,
        token::token_program = bond_token_program
    )]
    pub attestor_bond_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub bond_token_program: Interface<'info, TokenInterface>,
}

/// UC-13 (D8). Returns whatever bond is left (zero after a slash) and deregisters.
pub fn handle_withdraw_bond(ctx: Context<WithdrawBond>) -> Result<()> {
    let a = &ctx.accounts.attestor_account;
    require!(a.active_commitments == 0, SlotbookError::LiveCommitments);
    let amount = a.bond_amount;
    if amount > 0 {
        let cfg = &ctx.accounts.issuer_config;
        let seeds: &[&[u8]] = &[ISSUER_SEED, cfg.mint.as_ref(), &[cfg.bump]];
        transfer_from_pda(
            &ctx.accounts.bond_token_program.to_account_info(),
            &ctx.accounts.bond_vault.to_account_info(),
            &ctx.accounts.bond_mint.to_account_info(),
            &ctx.accounts.attestor_bond_account.to_account_info(),
            &cfg.to_account_info(),
            &[seeds],
            amount,
            ctx.accounts.bond_mint.decimals,
        )?;
    }
    // The Attestor account closes to the attestor (close = attestor).
    Ok(())
}
