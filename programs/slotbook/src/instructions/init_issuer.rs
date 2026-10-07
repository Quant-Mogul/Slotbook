use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-1. Registers a mint. Creates IssuerConfig and the bond vault.
#[derive(Accounts)]
pub struct InitIssuer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The permissioned Token-2022 mint being registered.
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    /// Bonds are paid in this mint (D9, devnet USDC).
    pub bond_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = authority,
        space = 8 + IssuerConfig::INIT_SPACE,
        seeds = [ISSUER_SEED, mint.key().as_ref()],
        bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = bond_mint,
        associated_token::authority = issuer_config,
        associated_token::token_program = bond_token_program
    )]
    pub bond_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub bond_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// TODO(UC-1):
// - authority == mint.mint_authority, which must be set (D22)
// - mint.freeze_authority == PDA ["MINT_CONFIG", mint] under TOKEN_ACL_PROGRAM_ID (D22)
// - mint carries DefaultAccountState and not ConfidentialTransfer (UC-1, D15)
// - validate params: 1 <= attestors <= MAX_ATTESTORS, 1 <= quorum <= attestors,
//   windows nonzero, resolver != authority and not an attestor,
//   resolver_fee < min(attestor_bond, challenger_bond) (D19)
// - write all fields, distribution_count = 0, active_distributions = 0, bump
pub fn handle_init_issuer(_ctx: Context<InitIssuer>, _params: IssuerParams) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
