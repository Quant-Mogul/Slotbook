use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-6. Resolver rules on a dispute; anyone may rule Expired after the timeout.
///
/// Remaining accounts: one (Commitment, Attestor) pair per live commitment, writable;
/// the number of pairs must equal distribution.commitment_count (D22).
#[derive(Accounts)]
pub struct ResolveChallenge<'info> {
    /// The resolver, or any cranker for Ruling::Expired.
    #[account(mut)]
    pub signer: Signer<'info>,

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

    /// Present for an owner challenge; absent for a root conflict (D3).
    #[account(mut)]
    pub challenge: Option<Box<Account<'info, Challenge>>>,

    /// Receives the Challenge rent; must equal challenge.challenger.
    /// CHECK: checked in the handler against challenge.challenger.
    #[account(mut)]
    pub challenger: Option<UncheckedAccount<'info>>,

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
        token::authority = issuer_config.resolver,
        token::token_program = bond_token_program
    )]
    pub resolver_bond_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Challenger's bond-mint account; owner checked in the handler.
    #[account(mut, token::mint = bond_mint, token::token_program = bond_token_program)]
    pub challenger_bond_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,

    #[account(
        mut,
        token::mint = bond_mint,
        token::authority = issuer_config.authority,
        token::token_program = bond_token_program
    )]
    pub issuer_bond_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub bond_token_program: Interface<'info, TokenInterface>,
}

// TODO(UC-6):
// - state is Disputed; remaining accounts are exactly commitment_count pairs (D22)
// - Owner dispute: challenge is present, challenge.distribution == distribution and
//   challenge.index == open_challenge_index, else WrongChallenge; challenger account and
//   challenger_bond_account belong to challenge.challenger
// - Conflict dispute: challenge is absent
// - Upheld / Rejected / Conflict: signer == issuer_config.resolver
// - Expired: Clock.slot >= dispute_opened_slot + resolve_timeout_slots (D17)
// - Upheld (D4): close every Commitment with the challenged root, slash attestor_bond each,
//   resolver_fee to resolver, rest to challenger; challenger bond returned; state Declared
// - Rejected (D5): fee to resolver, rest of challenger bond split among the root's attestors,
//   remainder to resolver; state Committed, window_end_slot = slot + slots_remaining
// - Conflict { root } (D3): slash attestors whose root differs, fee to resolver,
//   rest to issuer; Committed if quorum still holds, else Declared
// - Expired (D17): challenger bond returned, every Commitment closed without slashing; Declared
// - decrement active_commitments / commitment_count for every closed Commitment (D8)
// - close Challenge with rent to challenger; clear open_challenge_index and dispute_kind
pub fn handle_resolve_challenge(_ctx: Context<ResolveChallenge>, _ruling: Ruling) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
