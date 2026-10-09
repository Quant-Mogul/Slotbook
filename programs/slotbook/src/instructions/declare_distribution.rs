use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::*,
    error::SlotbookError,
    state::*,
    utils::{is_plain_vault_mint, transfer_from_wallet},
};

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

    /// init_if_needed: anyone can create an ATA for any address, so a plain `init`
    /// could be blocked by someone creating this vault first.
    #[account(
        init_if_needed,
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

/// UC-3 (D14, D21). Announces a future record slot and funds the vault with `total`.
pub fn handle_declare_distribution(
    ctx: Context<DeclareDistribution>,
    total: u64,
    record_slot: u64,
    salt_seed_hash: [u8; 32],
) -> Result<()> {
    let slot = Clock::get()?.slot;
    let cfg = &mut ctx.accounts.issuer_config;

    // D14: the record slot is public before it happens.
    let earliest = slot.checked_add(cfg.notice_slots).ok_or(SlotbookError::MathOverflow)?;
    require!(record_slot >= earliest, SlotbookError::RecordSlotTooSoon);
    require!(total > 0, SlotbookError::InvalidParams);

    // D21: the vault assumes the full amount arrives and that nobody else can move it.
    require!(
        is_plain_vault_mint(&ctx.accounts.payment_mint.to_account_info())?,
        SlotbookError::UnsupportedPaymentMint
    );

    let id = cfg.distribution_count;
    cfg.distribution_count = id.checked_add(1).ok_or(SlotbookError::MathOverflow)?;
    cfg.active_distributions = cfg
        .active_distributions
        .checked_add(1)
        .ok_or(SlotbookError::MathOverflow)?;

    let d = &mut ctx.accounts.distribution;
    d.issuer_config = cfg.key();
    d.id = id;
    d.payment_mint = ctx.accounts.payment_mint.key();
    d.record_slot = record_slot;
    d.total = total;
    d.state = DistributionState::Declared;
    d.final_root = [0; 32];
    d.register_total = 0;
    d.salt_seed_hash = salt_seed_hash;
    d.claimed_total = 0;
    d.pending_total = 0;
    d.swept_total = 0;
    d.declared_slot = slot;
    d.window_end_slot = 0;
    d.slots_remaining = 0;
    d.open_slot = 0;
    d.commitment_count = 0;
    d.challenge_count = 0;
    d.open_challenge_index = None;
    d.dispute_kind = None;
    d.dispute_opened_slot = 0;
    d.bump = ctx.bumps.distribution;

    transfer_from_wallet(
        &ctx.accounts.payment_token_program.to_account_info(),
        &ctx.accounts.issuer_payment_account.to_account_info(),
        &ctx.accounts.payment_mint.to_account_info(),
        &ctx.accounts.vault.to_account_info(),
        &ctx.accounts.authority.to_account_info(),
        total,
        ctx.accounts.payment_mint.decimals,
    )
}
