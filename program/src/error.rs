//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TakerError {
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// Insufficient funds for the operation requested.
    #[error("Insufficient funds")]
    InsufficientFunds,
    /// Invalid Mint.
    #[error("Invalid Mint")]
    InvalidMint,
    /// Account not associated with this Mint.
    #[error("Account not associated with this Mint")]
    MintMismatch,
    /// Owner does not match.
    #[error("Owner does not match")]
    OwnerMismatch,
    /// This token's supply is fixed and new tokens cannot be minted.
    #[error("Fixed supply")]
    FixedSupply,
    /// The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// Invalid number of provided signers.
    #[error("Invalid number of provided signers")]
    InvalidNumberOfProvidedSigners,
    /// Invalid number of required signers.
    #[error("Invalid number of required signers")]
    InvalidNumberOfRequiredSigners,
    /// State is uninitialized.
    #[error("State is unititialized")]
    UninitializedState,
    /// Instruction does not support native tokens
    #[error("Instruction does not support native tokens")]
    NativeNotSupported,
    /// Non-native account can only be closed if its balance is zero
    #[error("Non-native account can only be closed if its balance is zero")]
    NonNativeHasBalance,
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// State is invalid for requested operation.
    #[error("State is invalid for requested operation")]
    InvalidState,
    /// Operation overflowed
    #[error("Operation overflowed")]
    Overflow,
    /// Account does not support specified authority type.
    #[error("Account does not support specified authority type")]
    AuthorityTypeNotSupported,
    /// This token mint cannot freeze accounts.
    #[error("This token mint cannot freeze accounts")]
    MintCannotFreeze,
    /// Account is frozen; all account operations will fail
    #[error("Account is frozen")]
    AccountFrozen,
    /// Mint decimals mismatch between the client and mint
    #[error("The provided decimals value different from the Mint decimals")]
    MintDecimalsMismatch,
}
impl From<TakerError> for ProgramError {
    fn from(e: TakerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TakerError {
    fn type_of() -> &'static str {
        "TakerError"
    }
}

impl PrintProgramError for TakerError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TakerError::NotRentExempt => msg!("Error: Lamport balance below rent-exempt threshold"),
            TakerError::InsufficientFunds => msg!("Error: insufficient funds"),
            TakerError::InvalidMint => msg!("Error: Invalid Mint"),
            TakerError::MintMismatch => msg!("Error: Account not associated with this Mint"),
            TakerError::OwnerMismatch => msg!("Error: owner does not match"),
            TakerError::FixedSupply => msg!("Error: the total supply of this token is fixed"),
            TakerError::AlreadyInUse => msg!("Error: account or token already in use"),
            TakerError::InvalidNumberOfProvidedSigners => {
                msg!("Error: Invalid number of provided signers")
            }
            TakerError::InvalidNumberOfRequiredSigners => {
                msg!("Error: Invalid number of required signers")
            }
            TakerError::UninitializedState => msg!("Error: State is uninitialized"),
            TakerError::NativeNotSupported => {
                msg!("Error: Instruction does not support native tokens")
            }
            TakerError::NonNativeHasBalance => {
                msg!("Error: Non-native account can only be closed if its balance is zero")
            }
            TakerError::InvalidInstruction => msg!("Error: Invalid instruction"),
            TakerError::InvalidState => msg!("Error: Invalid account state for operation"),
            TakerError::Overflow => msg!("Error: Operation overflowed"),
            TakerError::AuthorityTypeNotSupported => {
                msg!("Error: Account does not support specified authority type")
            }
            TakerError::MintCannotFreeze => msg!("Error: This token mint cannot freeze accounts"),
            TakerError::AccountFrozen => msg!("Error: Account is frozen"),
            TakerError::MintDecimalsMismatch => {
                msg!("Error: decimals different from the Mint decimals")
            }
        }
    }
}
