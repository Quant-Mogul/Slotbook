use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::SlotbookError, state::*, utils::transfer_from_pda};

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

/// UC-6 (D3, D4, D5, D17, D22).
///
/// Bond movements all come out of the bond vault, signed by the IssuerConfig PDA:
/// - Upheld (D4): every commitment with the challenged root is closed and its attestor
///   slashed exactly attestor_bond (resolver_fee to the resolver, rest to the challenger);
///   the challenger's bond comes back in full. State Declared.
/// - Rejected (D5): resolver_fee from the challenger's bond to the resolver; the rest is
///   split equally into the bonds of the root's attestors (credited to bond_amount, so it
///   stays in the vault until withdraw_bond); the division remainder goes to the resolver.
///   The window resumes with the slots it had left.
/// - Conflict { root } (D3): commitments whose root or register_total differs from the
///   named one are closed and their attestors slashed (fee to resolver, rest to the issuer).
///   Committed if the survivors still reach quorum, else Declared.
/// - Expired (D17): anyone, after resolve_timeout_slots. Challenger bond returned, every
///   commitment closed without slashing. State Declared.
/// Closed commitments send their rent to the Attestor account; withdraw_bond returns it.
pub fn handle_resolve_challenge<'info>(
    mut ctx: Context<'info, ResolveChallenge<'info>>,
    ruling: Ruling,
) -> Result<()> {
    let slot = Clock::get()?.slot;
    let rem = ctx.remaining_accounts;
    let accs = &mut ctx.accounts;
    let cfg = &accs.issuer_config;
    let d_key = accs.distribution.key();
    let cfg_key = cfg.key();

    require!(accs.distribution.state == DistributionState::Disputed, SlotbookError::InvalidState);
    let kind = accs.distribution.dispute_kind.ok_or(SlotbookError::InvalidState)?;

    // Who may rule.
    if ruling == Ruling::Expired {
        let deadline = accs
            .distribution
            .dispute_opened_slot
            .checked_add(cfg.resolve_timeout_slots)
            .ok_or(SlotbookError::MathOverflow)?;
        require!(slot >= deadline, SlotbookError::TimeoutNotReached);
    } else {
        require!(accs.signer.key() == cfg.resolver, SlotbookError::NotResolver);
    }
    let fits = matches!(
        (kind, ruling),
        (DisputeKind::Owner, Ruling::Upheld | Ruling::Rejected | Ruling::Expired)
            | (DisputeKind::Conflict, Ruling::Conflict { .. } | Ruling::Expired)
    );
    require!(fits, SlotbookError::InvalidRuling);

    // An owner dispute binds the open Challenge, its challenger and their bond account.
    let mut challenge_root = [0u8; 32];
    let mut challenge_bond = 0u64;
    if kind == DisputeKind::Owner {
        let c = accs.challenge.as_ref().ok_or(SlotbookError::WrongChallenge)?;
        require!(
            c.distribution == d_key && Some(c.index) == accs.distribution.open_challenge_index,
            SlotbookError::WrongChallenge
        );
        let who = accs.challenger.as_ref().ok_or(SlotbookError::WrongChallenge)?;
        require!(who.key() == c.challenger, SlotbookError::WrongChallenge);
        let bond_acc = accs.challenger_bond_account.as_ref().ok_or(SlotbookError::WrongChallenge)?;
        require!(bond_acc.owner == c.challenger, SlotbookError::WrongChallenge);
        challenge_root = c.root;
        challenge_bond = c.bond;
    } else {
        require!(accs.challenge.is_none(), SlotbookError::WrongChallenge);
    }

    // D22: every live commitment must be handed in, each with its own attestor.
    let count = accs.distribution.commitment_count as usize;
    require!(rem.len() == 2 * count, SlotbookError::CommitmentSetMismatch);
    let mut pairs: Vec<(Account<'info, Commitment>, Account<'info, Attestor>)> = Vec::with_capacity(count);
    for chunk in rem.chunks(2) {
        require!(chunk[0].is_writable && chunk[1].is_writable, SlotbookError::CommitmentSetMismatch);
        let c: Account<'info, Commitment> = Account::try_from(&chunk[0])?;
        let a: Account<'info, Attestor> = Account::try_from(&chunk[1])?;
        let c_addr = Pubkey::create_program_address(
            &[COMMITMENT_SEED, d_key.as_ref(), c.attestor.as_ref(), &[c.bump]],
            &crate::ID,
        )
        .map_err(|_| SlotbookError::CommitmentSetMismatch)?;
        let a_addr = Pubkey::create_program_address(
            &[ATTESTOR_SEED, cfg_key.as_ref(), c.attestor.as_ref(), &[a.bump]],
            &crate::ID,
        )
        .map_err(|_| SlotbookError::CommitmentSetMismatch)?;
        require!(
            c.key() == c_addr
                && c.distribution == d_key
                && a.key() == a_addr
                && a.authority == c.attestor
                && a.issuer_config == cfg_key
                && !pairs.iter().any(|(p, _)| p.key() == c.key()),
            SlotbookError::CommitmentSetMismatch
        );
        pairs.push((c, a));
    }

    // Bond-vault payouts, signed by the IssuerConfig PDA.
    let token_program = accs.bond_token_program.to_account_info();
    let vault = accs.bond_vault.to_account_info();
    let mint = accs.bond_mint.to_account_info();
    let decimals = accs.bond_mint.decimals;
    let cfg_info = cfg.to_account_info();
    let cfg_seeds: &[&[u8]] = &[ISSUER_SEED, cfg.mint.as_ref(), &[cfg.bump]];
    let pay = |to: &AccountInfo<'info>, amount: u64| -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        transfer_from_pda(&token_program, &vault, &mint, to, &cfg_info, &[cfg_seeds], amount, decimals)
    };
    let resolver_acc = accs.resolver_bond_account.to_account_info();
    let challenger_acc = accs.challenger_bond_account.as_ref().map(|a| a.to_account_info());
    let issuer_acc = accs.issuer_bond_account.to_account_info();
    let attestor_bond = cfg.attestor_bond;
    let resolver_fee = cfg.resolver_fee;
    let quorum = cfg.quorum as usize;
    let challenge_window = cfg.challenge_window_slots;

    // Closes a commitment (rent to its Attestor account) and frees the attestor's slot.
    let mut closed = 0u32;
    let mut retire = |c: &Account<'info, Commitment>, a: &mut Account<'info, Attestor>| -> Result<()> {
        c.close(a.to_account_info())?;
        a.active_commitments = a.active_commitments.checked_sub(1).ok_or(SlotbookError::MathOverflow)?;
        closed += 1;
        Ok(())
    };
    // Slashes exactly attestor_bond: fee to the resolver, the rest to `rest_to`.
    let slash = |a: &mut Account<'info, Attestor>, rest_to: &AccountInfo<'info>| -> Result<()> {
        a.bond_amount = a.bond_amount.checked_sub(attestor_bond).ok_or(SlotbookError::MathOverflow)?;
        pay(&resolver_acc, resolver_fee)?;
        pay(rest_to, attestor_bond - resolver_fee)
    };

    let d = &mut accs.distribution;
    match ruling {
        Ruling::Upheld => {
            let to = challenger_acc.as_ref().ok_or(SlotbookError::WrongChallenge)?;
            for (c, a) in pairs.iter_mut() {
                if c.root == challenge_root {
                    slash(a, to)?;
                    retire(c, a)?;
                }
            }
            pay(to, challenge_bond)?;
            d.state = DistributionState::Declared;
        }
        Ruling::Rejected => {
            let backers = pairs.iter().filter(|(c, _)| c.root == challenge_root).count() as u64;
            require!(backers > 0, SlotbookError::CommitmentSetMismatch);
            let rest = challenge_bond.checked_sub(resolver_fee).ok_or(SlotbookError::MathOverflow)?;
            let share = rest / backers;
            pay(&resolver_acc, resolver_fee + rest % backers)?;
            for (c, a) in pairs.iter_mut() {
                if c.root == challenge_root {
                    a.bond_amount = a.bond_amount.checked_add(share).ok_or(SlotbookError::MathOverflow)?;
                }
            }
            d.state = DistributionState::Committed;
            d.window_end_slot = slot.checked_add(d.slots_remaining).ok_or(SlotbookError::MathOverflow)?;
        }
        Ruling::Conflict { root } => {
            let total = pairs.iter().find(|(c, _)| c.root == root).map(|(c, _)| c.register_total);
            let mut survivors = 0usize;
            for (c, a) in pairs.iter_mut() {
                if c.root == root && Some(c.register_total) == total {
                    survivors += 1;
                } else {
                    slash(a, &issuer_acc)?;
                    retire(c, a)?;
                }
            }
            match total {
                Some(t) if survivors > 0 => {
                    d.final_root = root;
                    d.register_total = t;
                }
                _ => {
                    d.final_root = [0; 32];
                    d.register_total = 0;
                }
            }
            if survivors >= quorum {
                d.state = DistributionState::Committed;
                d.window_end_slot = slot.checked_add(challenge_window).ok_or(SlotbookError::MathOverflow)?;
            } else {
                d.state = DistributionState::Declared;
            }
        }
        Ruling::Expired => {
            for (c, a) in pairs.iter_mut() {
                retire(c, a)?;
            }
            if let Some(to) = challenger_acc.as_ref() {
                pay(to, challenge_bond)?;
            }
            d.state = DistributionState::Declared;
        }
    }

    d.commitment_count = d.commitment_count.checked_sub(closed).ok_or(SlotbookError::MathOverflow)?;
    if d.state == DistributionState::Declared && d.commitment_count == 0 {
        // No live commitment: the next commit starts a fresh candidate.
        d.final_root = [0; 32];
        d.register_total = 0;
    }
    d.dispute_kind = None;
    d.open_challenge_index = None;
    d.slots_remaining = 0;

    // Persist the attestors (closed commitments are already gone).
    for (_, a) in pairs.iter() {
        a.exit(&crate::ID)?;
    }
    // The Challenge closes with rent to the challenger (D1).
    if let (Some(c), Some(who)) = (accs.challenge.as_ref(), accs.challenger.as_ref()) {
        c.clone().close(who.to_account_info())?;
    }
    Ok(())
}
