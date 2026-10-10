use anchor_lang::prelude::*;
use anchor_spl::token_interface::{close_account, CloseAccount, Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*, utils::transfer_from_pda};

/// UC-11. The issuer empties and closes the vault. The Distribution stays, as Closed (D7).
#[derive(Accounts)]
pub struct CloseDistribution<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
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

/// UC-11 (D7). Empties the vault to the issuer (dust included), closes it, marks Closed.
/// The Distribution account stays so attestors can still be released.
pub fn handle_close_distribution(ctx: Context<CloseDistribution>) -> Result<()> {
    let slot = Clock::get()?.slot;
    let cfg = &mut ctx.accounts.issuer_config;
    let d = &mut ctx.accounts.distribution;

    let allowed = match d.state {
        DistributionState::Open => {
            let expiry = d
                .open_slot
                .checked_add(cfg.claim_expiry_slots)
                .ok_or(SlotbookError::MathOverflow)?;
            d.pending_total == 0 && slot >= expiry
        }
        DistributionState::Declared => {
            let expiry = d
                .record_slot
                .checked_add(cfg.claim_expiry_slots)
                .ok_or(SlotbookError::MathOverflow)?;
            d.commitment_count == 0 || slot >= expiry
        }
        _ => false,
    };
    require!(allowed, SlotbookError::CannotClose);

    let id = d.id.to_le_bytes();
    let seeds: &[&[u8]] = &[DISTRIBUTION_SEED, d.issuer_config.as_ref(), &id, &[d.bump]];
    let remaining = ctx.accounts.vault.amount;
    if remaining > 0 {
        transfer_from_pda(
            &ctx.accounts.payment_token_program.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.payment_mint.to_account_info(),
            &ctx.accounts.issuer_payment_account.to_account_info(),
            &d.to_account_info(),
            &[seeds],
            remaining,
            ctx.accounts.payment_mint.decimals,
        )?;
    }
    close_account(CpiContext::new_with_signer(
        ctx.accounts.payment_token_program.key(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.authority.to_account_info(),
            authority: d.to_account_info(),
        },
        &[seeds],
    ))?;

    d.state = DistributionState::Closed;
    cfg.active_distributions = cfg
        .active_distributions
        .checked_sub(1)
        .ok_or(SlotbookError::MathOverflow)?;
    Ok(())
}
