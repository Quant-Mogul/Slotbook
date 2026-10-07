use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-5. Anyone disputes one owner's balance during the window, with a bond.
#[derive(Accounts)]
pub struct CreateChallenge<'info> {
    #[account(mut)]
    pub challenger: Signer<'info>,

    #[account(
        seeds = [ISSUER_SEED, issuer_config.mint.as_ref()],
        bump = issuer_config.bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    #[account(
        mut,
        has_one = issuer_config,
        seeds = [DISTRIBUTION_SEED, issuer_config.key().as_ref(), &distribution.id.to_le_bytes()],
        bump = distribution.bump
    )]
    pub distribution: Box<Account<'info, Distribution>>,

    #[account(
        init,
        payer = challenger,
        space = 8 + Challenge::INIT_SPACE,
        seeds = [
            CHALLENGE_SEED,
            distribution.key().as_ref(),
            &distribution.challenge_count.to_le_bytes()
        ],
        bump
    )]
    pub challenge: Box<Account<'info, Challenge>>,

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
        token::authority = challenger,
        token::token_program = bond_token_program
    )]
    pub challenger_bond_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub bond_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// TODO(UC-5):
// - state is Committed and Clock.slot < window_end_slot
// - transfer_checked challenger_bond into bond_vault
// - write challenge (index, challenger, root = final_root, owner, claimed_balance, bond,
//   opened_slot); owner == Pubkey::default() disputes register_total (D22)
// - Disputed, dispute_kind = Owner, dispute_opened_slot = slot,
//   slots_remaining = window_end_slot - slot, open_challenge_index, challenge_count += 1 (D1)
pub fn handle_challenge(
    _ctx: Context<CreateChallenge>,
    _owner: Pubkey,
    _claimed_balance: u64,
) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
