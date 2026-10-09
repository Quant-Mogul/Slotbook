pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp");

/// Slotbook: a verifiable, disputable record-date holder register for permissioned
/// SPL tokens. Use cases UC-1 to UC-14 and decisions D1 to D22 are in the scope lock;
/// the snapshot format is docs/SNAPSHOT_SPEC.md.
#[program]
pub mod slotbook {
    use super::*;

    // Issuer and attestor registry

    pub fn init_issuer(ctx: Context<InitIssuer>, params: IssuerParams) -> Result<()> {
        instructions::init_issuer::handle_init_issuer(ctx, params)
    }

    pub fn update_issuer(ctx: Context<UpdateIssuer>, params: IssuerParams) -> Result<()> {
        instructions::update_issuer::handle_update_issuer(ctx, params)
    }

    pub fn register_attestor(ctx: Context<RegisterAttestor>, backend: Backend) -> Result<()> {
        instructions::register_attestor::handle_register_attestor(ctx, backend)
    }

    pub fn withdraw_bond(ctx: Context<WithdrawBond>) -> Result<()> {
        instructions::withdraw_bond::handle_withdraw_bond(ctx)
    }

    // Distribution state machine

    pub fn declare_distribution(
        ctx: Context<DeclareDistribution>,
        total: u64,
        record_slot: u64,
        salt_seed_hash: [u8; 32],
    ) -> Result<()> {
        instructions::declare_distribution::handle_declare_distribution(
            ctx,
            total,
            record_slot,
            salt_seed_hash,
        )
    }

    pub fn commit_root(ctx: Context<CommitRoot>, args: CommitArgs) -> Result<()> {
        instructions::commit_root::handle_commit_root(ctx, args)
    }

    pub fn challenge(ctx: Context<CreateChallenge>, owner: Pubkey, claimed_balance: u64) -> Result<()> {
        instructions::challenge::handle_challenge(ctx, owner, claimed_balance)
    }

    pub fn resolve_challenge<'info>(
        ctx: Context<'info, ResolveChallenge<'info>>,
        ruling: Ruling,
    ) -> Result<()> {
        instructions::resolve_challenge::handle_resolve_challenge(ctx, ruling)
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        instructions::finalize::handle_finalize(ctx)
    }

    pub fn close_distribution(ctx: Context<CloseDistribution>) -> Result<()> {
        instructions::close_distribution::handle_close_distribution(ctx)
    }

    pub fn release_attestor(ctx: Context<ReleaseAttestor>) -> Result<()> {
        instructions::release_attestor::handle_release_attestor(ctx)
    }

    // Per-holder claim path

    pub fn claim(
        ctx: Context<Claim>,
        balance: u64,
        salt: [u8; 32],
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::claim::handle_claim(ctx, balance, salt, proof)
    }

    pub fn release_pending(ctx: Context<ReleasePending>) -> Result<()> {
        instructions::release_pending::handle_release_pending(ctx)
    }

    pub fn sweep_pending(ctx: Context<SweepPending>) -> Result<()> {
        instructions::sweep_pending::handle_sweep_pending(ctx)
    }
}
