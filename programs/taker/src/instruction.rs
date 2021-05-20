use crate::error::TakerError::{self, InvalidInstruction};
use fehler::{throw, throws};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    program_option::COption,
    pubkey::Pubkey,
    sysvar,
};
use std::convert::TryInto;
use std::mem::size_of;

/// Minimum number of multisignature signers (min N)
pub const MIN_SIGNERS: usize = 1;
/// Maximum number of multisignature signers (max N)
pub const MAX_SIGNERS: usize = 11;

/// Instructions supported by the token program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum TakerInstruction {
    /// Initialize the Contract Account
    /// Accounts expected by this instruction:
    ///
    ///   1. `[s]` The fee payer for creating the Contract Account.
    ///   2. `[]` The TKR Mint.
    ///   3. `[]` The TAI Mint.
    ///   4. `[]` The DAI Mint.
    ///   5. `[w]` The Contract Account (uninitialized).
    ///
    Initialize,
    /// Accounts expected by this instruction:
    ///
    ///   1. `[signer]` The user authority account.
    ///   2. `[writable]` The user's NFT Token Account.
    ///   3. `[writable]` The Taker's NFT Token Account.
    ///   4. `[writable]` The Taker's TKR Token Account.
    ///   5. `[writable]` The user's TKR Token Account.
    ///
    DepositNFT { token_id: Pubkey },
}

// TODO: add tests for pack and unpack
// pack and unpack the instruction from their binary representation
impl TakerInstruction {
    #[throws(ProgramError)]
    pub fn unpack(input: &[u8]) -> Self {
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        match tag {
            0 => Self::Initialize,
            1 => Self::DepositNFT {
                token_id: unpack_pubkey(rest)?.0,
            },
            _ => throw!(InvalidInstruction),
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize => {
                buf.push(0);
            }
            Self::DepositNFT { token_id } => {
                buf.push(1);
                buf.extend_from_slice(token_id.as_ref());
            }
        };
        buf
    }
}

#[throws(ProgramError)]
fn unpack_pubkey(input: &[u8]) -> (Pubkey, &[u8]) {
    if input.len() >= 32 {
        let (key, rest) = input.split_at(32);
        let pk = Pubkey::new(key);
        (pk, rest)
    } else {
        throw!(TakerError::InvalidInstruction)
    }
}

#[throws(ProgramError)]
fn unpack_pubkey_option(input: &[u8]) -> (COption<Pubkey>, &[u8]) {
    match input.split_first() {
        Option::Some((&0, rest)) => (COption::None, rest),
        Option::Some((&1, rest)) if rest.len() >= 32 => {
            let (key, rest) = rest.split_at(32);
            let pk = Pubkey::new(key);
            (COption::Some(pk), rest)
        }
        _ => throw!(TakerError::InvalidInstruction),
    }
}

fn pack_pubkey_option(value: &COption<Pubkey>, buf: &mut Vec<u8>) {
    match *value {
        COption::Some(ref key) => {
            buf.push(1);
            buf.extend_from_slice(&key.to_bytes());
        }
        COption::None => buf.push(0),
    }
}
