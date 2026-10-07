use anchor_lang::prelude::*;

use crate::{constants::*, error::SlotbookError, state::*};

/// UC-4. An attestor commits a root and register_total for a distribution.
#[derive(Accounts)]
pub struct CommitRoot<'info> {
    #[account(mut)]
    pub attestor: Signer<'info>,

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
        mut,
        seeds = [ATTESTOR_SEED, issuer_config.key().as_ref(), attestor.key().as_ref()],
        bump = attestor_account.bump
    )]
    pub attestor_account: Box<Account<'info, Attestor>>,

    #[account(
        init,
        payer = attestor,
        space = 8 + Commitment::INIT_SPACE,
        seeds = [COMMITMENT_SEED, distribution.key().as_ref(), attestor.key().as_ref()],
        bump
    )]
    pub commitment: Box<Account<'info, Commitment>>,

    pub system_program: Program<'info, System>,
}

// TODO(UC-4):
// - state is Declared or Committed; Clock.slot >= record_slot + finality_margin_slots (D6)
// - attestor in current issuer_config.attestors;
//   bond_amount >= (active_commitments + 1) * attestor_bond (D8)
// - spec_version == SPEC_VERSION
// - if Committed: root and register_total must equal final_root and register_total (D2)
// - write commitment; increment active_commitments and commitment_count
// - quorum reached with matching roots and totals (and both backends if required):
//   Committed, store final_root and register_total, window_end_slot = slot + window
// - a differing root before quorum: Disputed, dispute_kind = Conflict, dispute_opened_slot (D3)
pub fn handle_commit_root(_ctx: Context<CommitRoot>, _args: CommitArgs) -> Result<()> {
    err!(SlotbookError::NotImplemented)
}
