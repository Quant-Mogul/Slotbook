use anchor_lang::{prelude::*, system_program};
use anchor_spl::token_interface::{
    spl_token_2022::{
        extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
        state::{Account as SplAccount, AccountState, Mint as SplMint},
    },
    transfer_checked, TransferChecked,
};

/// Extension types on a mint. A legacy SPL mint has none.
pub fn mint_extensions(mint: &AccountInfo) -> Result<Vec<ExtensionType>> {
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<SplMint>::unpack(&data)?;
    Ok(state.get_extension_types()?)
}

/// D21: a vault mint must not let anyone take, tax, block or hide what the vault holds.
/// Applied to the payment mint (distribution vault) and the bond mint (bond vault).
pub fn is_plain_vault_mint(mint: &AccountInfo) -> Result<bool> {
    let extensions = mint_extensions(mint)?;
    Ok(![
        ExtensionType::TransferFeeConfig,
        ExtensionType::TransferHook,
        ExtensionType::PermanentDelegate,
        ExtensionType::ConfidentialTransferMint,
    ]
    .iter()
    .any(|banned| extensions.contains(banned)))
}

/// D10: a mint token account counts as transferable only if it is Initialized
/// (not Frozen) and carries ImmutableOwner, so its owner can never be reassigned.
pub fn is_transferable(token_account: &AccountInfo) -> Result<bool> {
    let data = token_account.try_borrow_data()?;
    let state = StateWithExtensions::<SplAccount>::unpack(&data)?;
    Ok(state.base.state == AccountState::Initialized
        && state.get_extension_types()?.contains(&ExtensionType::ImmutableOwner))
}

/// transfer_checked signed by the wallet that owns `from`.
#[allow(clippy::too_many_arguments)]
pub fn transfer_from_wallet<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let accounts = TransferChecked {
        from: from.clone(),
        mint: mint.clone(),
        to: to.clone(),
        authority: authority.clone(),
    };
    transfer_checked(CpiContext::new(token_program.key(), accounts), amount, decimals)
}

/// transfer_checked signed by a program PDA (a vault's authority).
#[allow(clippy::too_many_arguments)]
pub fn transfer_from_pda<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let accounts = TransferChecked {
        from: from.clone(),
        mint: mint.clone(),
        to: to.clone(),
        authority: authority.clone(),
    };
    transfer_checked(
        CpiContext::new_with_signer(token_program.key(), accounts, signer_seeds),
        amount,
        decimals,
    )
}

/// Creates a program-owned PDA account by hand, for accounts Anchor cannot `init`
/// on only one branch (Pending in claim). Works even if someone pre-funded the
/// address: then it tops up, allocates and assigns instead of create_account.
pub fn create_pda_account<'info>(
    payer: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    space: usize,
    owner: &Pubkey,
) -> Result<()> {
    let required = Rent::get()?.minimum_balance(space);
    let current = target.lamports();
    if current == 0 {
        system_program::create_account(
            CpiContext::new_with_signer(
                system_program::ID,
                system_program::CreateAccount { from: payer.clone(), to: target.clone() },
                signer_seeds,
            ),
            required,
            space as u64,
            owner,
        )
    } else {
        if current < required {
            system_program::transfer(
                CpiContext::new(
                    system_program::ID,
                    system_program::Transfer { from: payer.clone(), to: target.clone() },
                ),
                required - current,
            )?;
        }
        system_program::allocate(
            CpiContext::new_with_signer(
                system_program::ID,
                system_program::Allocate { account_to_allocate: target.clone() },
                signer_seeds,
            ),
            space as u64,
        )?;
        system_program::assign(
            CpiContext::new_with_signer(
                system_program::ID,
                system_program::Assign { account_to_assign: target.clone() },
                signer_seeds,
            ),
            owner,
        )
    }
}

/// D19 and UC-1: issuer parameter rules, shared by init_issuer and update_issuer.
pub fn validate_params(params: &crate::state::IssuerParams, authority: &Pubkey) -> Result<()> {
    use crate::{constants::MAX_ATTESTORS, error::SlotbookError};

    // One root per backend is not implemented yet, so an issuer could never reach
    // Committed with it on. Refuse it here instead of failing later in commit_root.
    require!(!params.require_both_backends, SlotbookError::NotImplemented);

    let n = params.attestors.len();
    require!(n >= 1, SlotbookError::InvalidParams);
    require!(n <= MAX_ATTESTORS, SlotbookError::TooManyAttestors);
    for (i, a) in params.attestors.iter().enumerate() {
        require!(!params.attestors[..i].contains(a), SlotbookError::InvalidParams);
    }
    require!(
        params.quorum >= 1 && (params.quorum as usize) <= n,
        SlotbookError::InvalidParams
    );
    require!(
        params.notice_slots > 0
            && params.finality_margin_slots > 0
            && params.challenge_window_slots > 0
            && params.resolve_timeout_slots > 0
            && params.hold_window_slots > 0
            && params.claim_expiry_slots > 0,
        SlotbookError::InvalidParams
    );
    require!(
        params.resolver != *authority && !params.attestors.contains(&params.resolver),
        SlotbookError::ResolverConflict
    );
    require!(
        params.resolver_fee < params.attestor_bond.min(params.challenger_bond),
        SlotbookError::ResolverFeeTooHigh
    );
    Ok(())
}

/// Copies parameters into IssuerConfig. Mint and bond mint are set once, at init.
pub fn apply_params(cfg: &mut crate::state::IssuerConfig, params: crate::state::IssuerParams) {
    cfg.attestors = params.attestors;
    cfg.quorum = params.quorum;
    cfg.require_both_backends = params.require_both_backends;
    cfg.resolver = params.resolver;
    cfg.notice_slots = params.notice_slots;
    cfg.finality_margin_slots = params.finality_margin_slots;
    cfg.challenge_window_slots = params.challenge_window_slots;
    cfg.resolve_timeout_slots = params.resolve_timeout_slots;
    cfg.hold_window_slots = params.hold_window_slots;
    cfg.claim_expiry_slots = params.claim_expiry_slots;
    cfg.attestor_bond = params.attestor_bond;
    cfg.challenger_bond = params.challenger_bond;
    cfg.resolver_fee = params.resolver_fee;
}
