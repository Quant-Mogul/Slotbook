use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        get_mint_extension_data,
        spl_token_2022::{
            extension::{default_account_state::DefaultAccountState, ExtensionType},
            state::AccountState,
        },
        Mint, TokenAccount, TokenInterface,
    },
};

use crate::{
    constants::*,
    error::SlotbookError,
    state::*,
    utils::{apply_params, is_plain_vault_mint, mint_extensions, validate_params},
};

/// UC-1. Registers a mint. Creates IssuerConfig and the bond vault.
#[derive(Accounts)]
pub struct InitIssuer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The permissioned Token-2022 mint being registered.
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    /// Bonds are paid in this mint (D9, devnet USDC).
    pub bond_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = authority,
        space = 8 + IssuerConfig::INIT_SPACE,
        seeds = [ISSUER_SEED, mint.key().as_ref()],
        bump
    )]
    pub issuer_config: Box<Account<'info, IssuerConfig>>,

    /// init_if_needed: anyone can create an ATA for any address, so a plain `init`
    /// could be blocked by someone creating this vault first.
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = bond_mint,
        associated_token::authority = issuer_config,
        associated_token::token_program = bond_token_program
    )]
    pub bond_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub bond_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// UC-1 (D15, D19, D22).
pub fn handle_init_issuer(ctx: Context<InitIssuer>, params: IssuerParams) -> Result<()> {
    let authority = ctx.accounts.authority.key();
    let mint = &ctx.accounts.mint;

    // D22: only the mint's issuer can register it, so nobody squats a mint they don't issue.
    require!(
        mint.mint_authority == COption::Some(authority),
        SlotbookError::NotMintAuthority
    );

    // D22: freezing is controlled by Token ACL's MintConfig PDA.
    let (mint_config, _) = Pubkey::find_program_address(
        &[TOKEN_ACL_MINT_CONFIG_SEED, mint.key().as_ref()],
        &TOKEN_ACL_PROGRAM_ID,
    );
    require!(
        mint.freeze_authority == COption::Some(mint_config),
        SlotbookError::FreezeAuthorityNotTokenAcl
    );

    // D15: discovery is complete only if every new account starts Frozen.
    // Confidential balances cannot be replayed, so those mints are refused.
    let mint_info = mint.to_account_info();
    let extensions = mint_extensions(&mint_info)?;
    require!(
        !extensions.contains(&ExtensionType::ConfidentialTransferMint),
        SlotbookError::ConfidentialTransferMint
    );
    require!(
        extensions.contains(&ExtensionType::DefaultAccountState),
        SlotbookError::MissingDefaultAccountState
    );
    let default_state = get_mint_extension_data::<DefaultAccountState>(&mint_info)?;
    require!(
        default_state.state == AccountState::Frozen as u8,
        SlotbookError::DefaultStateNotFrozen
    );

    // D21 for the bond vault: attestors and challengers deposit into it, so the bond
    // mint must not let the issuer drain it (PermanentDelegate), tax it (TransferFee),
    // block payouts (TransferHook) or hide its balance (ConfidentialTransfer).
    require!(
        is_plain_vault_mint(&ctx.accounts.bond_mint.to_account_info())?,
        SlotbookError::UnsupportedBondMint
    );

    validate_params(&params, &authority)?;

    let cfg = &mut ctx.accounts.issuer_config;
    cfg.authority = authority;
    cfg.mint = mint.key();
    cfg.bond_mint = ctx.accounts.bond_mint.key();
    apply_params(cfg, params);
    cfg.distribution_count = 0;
    cfg.active_distributions = 0;
    cfg.bump = ctx.bumps.issuer_config;
    Ok(())
}
