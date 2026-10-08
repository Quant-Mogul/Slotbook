use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::*,
    error::SlotbookError,
    state::*,
    utils::{create_pda_account, is_transferable, transfer_from_pda},
};

/// UC-8. The leaf owner claims: paid now, or held in Pending if their account is frozen.
#[derive(Accounts)]
pub struct Claim<'info> {
    /// Must be the leaf owner (D10).
    #[account(mut)]
    pub holder: Signer<'info>,

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
        init,
        payer = holder,
        space = 8 + ClaimReceipt::INIT_SPACE,
        seeds = [CLAIM_SEED, distribution.key().as_ref(), holder.key().as_ref()],
        bump
    )]
    pub claim_receipt: Box<Account<'info, ClaimReceipt>>,

    /// Created in the handler only when the holder's account is not transferable,
    /// because Anchor cannot init on one branch.
    /// CHECK: PDA address enforced by seeds; created and written in the handler.
    #[account(
        mut,
        seeds = [PENDING_SEED, distribution.key().as_ref(), holder.key().as_ref()],
        bump
    )]
    pub pending: UncheckedAccount<'info>,

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
        token::authority = holder,
        token::token_program = payment_token_program
    )]
    pub holder_payment_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Any account of the permissioned mint owned by the holder; read for freeze state.
    #[account(
        constraint = holder_mint_account.mint == issuer_config.mint @ SlotbookError::WrongMint,
        constraint = holder_mint_account.owner == holder.key() @ SlotbookError::NotLeafOwner
    )]
    pub holder_mint_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub payment_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

/// UC-8 (D10, D11, D13, D20). Verifies the holder's leaf against `final_root`
/// (spec sections 6 to 8), computes the payout on-chain, then pays or holds it.
pub fn handle_claim(
    ctx: Context<Claim>,
    balance: u64,
    salt: [u8; 32],
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let slot = Clock::get()?.slot;
    let holder = ctx.accounts.holder.key();
    let cfg = &ctx.accounts.issuer_config;
    let d = &mut ctx.accounts.distribution;

    require!(d.state == DistributionState::Open, SlotbookError::InvalidState);
    let expiry = d
        .open_slot
        .checked_add(cfg.claim_expiry_slots)
        .ok_or(SlotbookError::MathOverflow)?;
    require!(slot < expiry, SlotbookError::ClaimExpired);

    // Spec section 8: siblings only, bottom-up, at most 32. The leaf is built here
    // from the signer, so a caller can only prove a row about their own wallet.
    require!(proof.len() <= MAX_PROOF_DEPTH as usize, SlotbookError::ProofTooDeep);
    let leaf = merkle::leaf_hash(&holder.to_bytes(), balance, &salt);
    let root = proof.iter().fold(leaf, |h, s| merkle::parent(&h, s));
    require!(root == d.final_root, SlotbookError::InvalidProof);

    // D20: floor(total * balance / register_total), in u128.
    let payout = (d.total as u128)
        .checked_mul(balance as u128)
        .and_then(|x| x.checked_div(d.register_total as u128))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(SlotbookError::MathOverflow)?;

    let transferable = is_transferable(&ctx.accounts.holder_mint_account.to_account_info())?;

    let r = &mut ctx.accounts.claim_receipt;
    r.distribution = d.key();
    r.owner = holder;
    r.balance = balance;
    r.payout = payout;
    r.bump = ctx.bumps.claim_receipt;

    if transferable {
        r.status = ClaimStatus::Paid;
        d.claimed_total = d.claimed_total.checked_add(payout).ok_or(SlotbookError::MathOverflow)?;
        let id = d.id.to_le_bytes();
        let seeds: &[&[u8]] = &[
            DISTRIBUTION_SEED,
            d.issuer_config.as_ref(),
            &id,
            &[d.bump],
        ];
        transfer_from_pda(
            &ctx.accounts.payment_token_program.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.payment_mint.to_account_info(),
            &ctx.accounts.holder_payment_account.to_account_info(),
            &d.to_account_info(),
            &[seeds],
            payout,
            ctx.accounts.payment_mint.decimals,
        )?;
    } else {
        // D10, D11: frozen (or not ImmutableOwner) means held, not paid and not forfeited.
        r.status = ClaimStatus::Pending;
        d.pending_total = d.pending_total.checked_add(payout).ok_or(SlotbookError::MathOverflow)?;
        let distribution_key = d.key();
        let bump = [ctx.bumps.pending];
        let seeds: &[&[u8]] = &[PENDING_SEED, distribution_key.as_ref(), holder.as_ref(), &bump];
        let space = 8 + Pending::INIT_SPACE;
        create_pda_account(
            &ctx.accounts.holder.to_account_info(),
            &ctx.accounts.pending.to_account_info(),
            &[seeds],
            space,
            &crate::ID,
        )?;
        let pending = Pending {
            distribution: distribution_key,
            owner: holder,
            amount: payout,
            created_slot: slot,
            payer: holder,
            bump: ctx.bumps.pending,
        };
        let info = ctx.accounts.pending.to_account_info();
        let mut data = info.try_borrow_mut_data()?;
        pending.try_serialize(&mut &mut data[..])?;
    }

    // D20 invariant.
    let committed = d
        .claimed_total
        .checked_add(d.pending_total)
        .and_then(|x| x.checked_add(d.swept_total))
        .ok_or(SlotbookError::MathOverflow)?;
    require!(committed <= d.total, SlotbookError::InvariantViolated);
    Ok(())
}
