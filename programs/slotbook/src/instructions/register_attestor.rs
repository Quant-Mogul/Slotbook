use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*, utils::transfer_from_wallet};

/// UC-2. An attestor in the issuer's set posts its bond.
#[derive(Accounts)]
pub struct RegisterAttestor<'info> {
    #[account(mut)]
    pub attestor: Signer<'info>,

    #[account(
        seeds = [ISSUER_SEED, issuer_config.mint.as_ref()],
        bump = issuer_config.bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    #[account(
        init,
        payer = attestor,
        space = 8 + Attestor::INIT_SPACE,
        seeds = [ATTESTOR_SEED, issuer_config.key().as_ref(), attestor.key().as_ref()],
        bump
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
    pub system_program: Program<'info, System>,
}

/// UC-2. Posts attestor_bond into the bond vault.
pub fn handle_register_attestor(ctx: Context<RegisterAttestor>, backend: Backend) -> Result<()> {
    let cfg = &ctx.accounts.issuer_config;
    let attestor = ctx.accounts.attestor.key();
    require!(cfg.attestors.contains(&attestor), SlotbookError::NotAnAttestor);

    transfer_from_wallet(
        &ctx.accounts.bond_token_program.to_account_info(),
        &ctx.accounts.attestor_bond_account.to_account_info(),
        &ctx.accounts.bond_mint.to_account_info(),
        &ctx.accounts.bond_vault.to_account_info(),
        &ctx.accounts.attestor.to_account_info(),
        cfg.attestor_bond,
        ctx.accounts.bond_mint.decimals,
    )?;

    let a = &mut ctx.accounts.attestor_account;
    a.issuer_config = cfg.key();
    a.authority = attestor;
    a.bond_amount = cfg.attestor_bond;
    a.backend = backend;
    a.active_commitments = 0;
    a.bump = ctx.bumps.attestor_account;
    Ok(())
}
