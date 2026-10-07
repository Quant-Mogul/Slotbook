use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

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

// TODO(UC-8):
// - state is Open and Clock.slot < open_slot + claim_expiry_slots (D10)
// - proof.len() <= MAX_PROOF_DEPTH; leaf = merkle::leaf_hash(holder, balance, salt);
//   fold siblings with merkle::parent; must equal final_root (spec sections 6 to 8)
// - payout = floor(total * balance / register_total) in u128 (D20)
// - write claim_receipt
// - holder_mint_account Initialized and has ImmutableOwner: transfer_checked payout from vault
//   (Distribution PDA signs), claimed_total += payout, status Paid
// - otherwise: create Pending (amount, created_slot, payer = holder), pending_total += payout,
//   status Pending (D10, D11)
// - claimed_total + pending_total + swept_total <= total (D20)
pub fn handle_claim(
    _ctx: Context<Claim>,
    _balance: u64,
    _salt: [u8; 32],
    _proof: Vec<[u8; 32]>,
) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
