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

/// UC-4 (D2, D3, D6, D8).
///
/// Before quorum, `distribution.final_root` and `register_total` hold the candidate:
/// the first commitment's values. Every later commitment must match both. The
/// candidate becomes final when `quorum` matching commitments exist.
pub fn handle_commit_root(ctx: Context<CommitRoot>, args: CommitArgs) -> Result<()> {
    let slot = Clock::get()?.slot;
    let cfg = &ctx.accounts.issuer_config;
    let attestor = ctx.accounts.attestor.key();
    let d = &mut ctx.accounts.distribution;
    let a = &mut ctx.accounts.attestor_account;

    require!(
        matches!(d.state, DistributionState::Declared | DistributionState::Committed),
        SlotbookError::InvalidState
    );
    // D6: the program never checks finality itself; it waits a margin.
    let earliest = d
        .record_slot
        .checked_add(cfg.finality_margin_slots)
        .ok_or(SlotbookError::MathOverflow)?;
    require!(slot >= earliest, SlotbookError::FinalityMarginNotPassed);

    // D8: only the current set, and each live commitment is backed by one attestor_bond.
    require!(cfg.attestors.contains(&attestor), SlotbookError::NotAnAttestor);
    let needed = (a.active_commitments as u64)
        .checked_add(1)
        .and_then(|n| n.checked_mul(cfg.attestor_bond))
        .ok_or(SlotbookError::MathOverflow)?;
    require!(a.bond_amount >= needed, SlotbookError::InsufficientBond);

    require!(args.spec_version == SPEC_VERSION, SlotbookError::UnsupportedSpecVersion);
    require!(args.rows > 0 && args.register_total > 0, SlotbookError::EmptyRegister);
    // One root per backend is not implemented yet; the sprint demo runs with it off.
    require!(!cfg.require_both_backends, SlotbookError::NotImplemented);

    let first = d.commitment_count == 0 && d.state == DistributionState::Declared;
    let matches = args.root == d.final_root && args.register_total == d.register_total;
    if d.state == DistributionState::Committed {
        // D2: after quorum, a late commitment must equal the final root and total.
        require!(matches, SlotbookError::RootMismatch);
    }

    let c = &mut ctx.accounts.commitment;
    c.distribution = d.key();
    c.attestor = attestor;
    c.root = args.root;
    c.manifest_hash = args.manifest_hash;
    c.backend = a.backend;
    c.register_total = args.register_total;
    c.resolved_slot = args.resolved_slot;
    c.first_covered_slot = args.first_covered_slot;
    c.rows = args.rows;
    c.spec_version = args.spec_version;
    c.posted_slot = slot;
    c.bump = ctx.bumps.commitment;

    a.active_commitments = a.active_commitments.checked_add(1).ok_or(SlotbookError::MathOverflow)?;
    d.commitment_count = d.commitment_count.checked_add(1).ok_or(SlotbookError::MathOverflow)?;

    if d.state == DistributionState::Committed {
        return Ok(());
    }
    if first {
        d.final_root = args.root;
        d.register_total = args.register_total;
    } else if !matches {
        // D3: roots differ before quorum. The resolver names the right one.
        d.state = DistributionState::Disputed;
        d.dispute_kind = Some(DisputeKind::Conflict);
        d.dispute_opened_slot = slot;
        return Ok(());
    }
    if d.commitment_count >= cfg.quorum as u32 {
        d.state = DistributionState::Committed;
        d.window_end_slot = slot
            .checked_add(cfg.challenge_window_slots)
            .ok_or(SlotbookError::MathOverflow)?;
    }
    Ok(())
}
