use anchor_lang::prelude::*;

#[error_code]
pub enum SlotbookError {
    #[msg("Instruction not implemented yet")]
    NotImplemented,

    // Issuer config (UC-1, UC-14; D18, D19, D22)
    #[msg("Signer is not the mint's mint authority, or it is unset")]
    NotMintAuthority,
    #[msg("Mint freeze authority is not the Token ACL MintConfig PDA")]
    FreezeAuthorityNotTokenAcl,
    #[msg("Mint must carry DefaultAccountState")]
    MissingDefaultAccountState,
    #[msg("Mints with the confidential-transfer extension are not supported")]
    ConfidentialTransferMint,
    #[msg("Invalid issuer parameters")]
    InvalidParams,
    #[msg("Too many attestors")]
    TooManyAttestors,
    #[msg("Resolver must be neither the authority nor an attestor")]
    ResolverConflict,
    #[msg("resolver_fee must be below both bonds")]
    ResolverFeeTooHigh,
    #[msg("Distributions are still active")]
    ActiveDistributions,
    #[msg("Signer is not the issuer authority")]
    NotAuthority,

    // Distribution (UC-3, UC-7, UC-11; D7, D14, D21)
    #[msg("Record slot is not far enough in the future")]
    RecordSlotTooSoon,
    #[msg("Payment mint has a disallowed extension")]
    UnsupportedPaymentMint,
    #[msg("Distribution is not in the required state")]
    InvalidState,
    #[msg("Distribution cannot be closed yet")]
    CannotClose,

    // Attestors and commitments (UC-2, UC-4, UC-12, UC-13; D2, D6, D8)
    #[msg("Signer is not in the issuer's attestor set")]
    NotAnAttestor,
    #[msg("Finality margin after the record slot has not passed")]
    FinalityMarginNotPassed,
    #[msg("Attestor bond does not cover another commitment")]
    InsufficientBond,
    #[msg("Root or register_total does not match the final commitment")]
    RootMismatch,
    #[msg("Spec version is not supported")]
    UnsupportedSpecVersion,
    #[msg("Attestor still has live commitments")]
    LiveCommitments,

    // Challenges (UC-5, UC-6; D1, D3, D4, D5, D17, D22)
    #[msg("Challenge window is closed")]
    WindowClosed,
    #[msg("Challenge window has not elapsed")]
    WindowNotElapsed,
    #[msg("Signer is not the resolver")]
    NotResolver,
    #[msg("Resolve timeout has not been reached")]
    TimeoutNotReached,
    #[msg("Remaining accounts must be every live commitment and its attestor")]
    CommitmentSetMismatch,
    #[msg("Challenge account does not match the open challenge")]
    WrongChallenge,

    // Claims (UC-8, UC-9, UC-10; D10, D11, D13, D20)
    #[msg("Claim period has expired")]
    ClaimExpired,
    #[msg("Merkle proof is too deep")]
    ProofTooDeep,
    #[msg("Merkle proof is invalid")]
    InvalidProof,
    #[msg("Token account is not owned by the leaf owner")]
    NotLeafOwner,
    #[msg("Token account is for the wrong mint")]
    WrongMint,
    #[msg("Hold window has not elapsed")]
    HoldWindowNotElapsed,
    #[msg("Hold window has elapsed")]
    HoldWindowElapsed,
    #[msg("Holder's mint account is not transferable yet")]
    NotTransferable,

    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("claimed + pending + swept would exceed total")]
    InvariantViolated,
}
