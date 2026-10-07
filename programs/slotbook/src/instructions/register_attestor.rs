use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

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

// TODO(UC-2):
// - attestor is in issuer_config.attestors
// - transfer_checked attestor_bond into bond_vault
// - write authority, issuer_config, bond_amount, backend, active_commitments = 0, bump
pub fn handle_register_attestor(_ctx: Context<RegisterAttestor>, _backend: Backend) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
