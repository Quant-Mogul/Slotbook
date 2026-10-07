use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*, utils::transfer_from_wallet};

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

/// UC-5 (D1, D22). Disputes one owner's balance during the window, with a bond.
/// `owner == Pubkey::default()` disputes register_total instead (D22).
/// `claimed_balance` is what the challenger says is correct; 0 means "should be absent",
/// nonzero for an owner not in the tree means "was omitted" (spec section 10).
pub fn handle_challenge(
    ctx: Context<CreateChallenge>,
    owner: Pubkey,
    claimed_balance: u64,
) -> Result<()> {
    let slot = Clock::get()?.slot;
    let cfg = &ctx.accounts.issuer_config;
    let d = &mut ctx.accounts.distribution;

    require!(d.state == DistributionState::Committed, SlotbookError::InvalidState);
    require!(slot < d.window_end_slot, SlotbookError::WindowClosed);

    transfer_from_wallet(
        &ctx.accounts.bond_token_program.to_account_info(),
        &ctx.accounts.challenger_bond_account.to_account_info(),
        &ctx.accounts.bond_mint.to_account_info(),
        &ctx.accounts.bond_vault.to_account_info(),
        &ctx.accounts.challenger.to_account_info(),
        cfg.challenger_bond,
        ctx.accounts.bond_mint.decimals,
    )?;

    let index = d.challenge_count;
    let c = &mut ctx.accounts.challenge;
    c.distribution = d.key();
    c.index = index;
    c.challenger = ctx.accounts.challenger.key();
    c.root = d.final_root;
    c.owner = owner;
    c.claimed_balance = claimed_balance;
    c.bond = cfg.challenger_bond;
    c.opened_slot = slot;
    c.bump = ctx.bumps.challenge;

    // The window pauses: remember how much of it was left (D5).
    d.slots_remaining = d.window_end_slot - slot;
    d.state = DistributionState::Disputed;
    d.dispute_kind = Some(DisputeKind::Owner);
    d.dispute_opened_slot = slot;
    d.open_challenge_index = Some(index);
    d.challenge_count = index.checked_add(1).ok_or(SlotbookError::MathOverflow)?;
    Ok(())
}
