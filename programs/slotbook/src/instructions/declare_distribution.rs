use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-3. Announces a payment for holders at a future record slot and funds the vault.
#[derive(Accounts)]
pub struct DeclareDistribution<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ SlotbookError::NotAuthority,
        seeds = [ISSUER_SEED, issuer_config.mint.as_ref()],
        bump = issuer_config.bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = authority,
        space = 8 + Distribution::INIT_SPACE,
        seeds = [
            DISTRIBUTION_SEED,
            issuer_config.key().as_ref(),
            &issuer_config.distribution_count.to_le_bytes()
        ],
        bump
    )]
    pub distribution: Box<Account<'info, Distribution>>,

    #[account(
        init,
        payer = authority,
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// TODO(UC-3):
// - record_slot >= Clock.slot + notice_slots (D14)
// - payment mint has no TransferFee, TransferHook, PermanentDelegate, ConfidentialTransfer (D21)
// - id = distribution_count; increment distribution_count and active_distributions
// - write Declared state, payment_mint, record_slot, total, salt_seed_hash, declared_slot
// - transfer_checked total into vault
pub fn handle_declare_distribution(
    _ctx: Context<DeclareDistribution>,
    _total: u64,
    _record_slot: u64,
    _salt_seed_hash: [u8; 32],
) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
