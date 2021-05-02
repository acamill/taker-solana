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
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The SPL-Token Program Account.
    ///   1. `[]` The user wallet account.
    ///   2. `[writable]` The user token account.
    ///   3. `[writable]` The taker token account.
    ///
    DepositNFT { token_id: Pubkey },
}

pub fn deposit_nft(token_id: Pubkey, nft_holder_keypair: Pubkey) -> Self {
    Instruction::new_with_bytes(
        crate::id(),
        &TakerInstruction::DepositNFT { token_id }.pack(),
        vec![
            AccountMeta::new(nft_holder_keypair.pubkey(), true),
            AccountMeta::new(
                get_associated_token_address(&nft_holder_keypair.pubkey(), &opt.nft_mint_address),
                false,
            ),
            AccountMeta::new(
                get_associated_token_address(&taker_owner_keypair.pubkey(), &opt.nft_mint_address),
                false,
            ),
            AccountMeta::new(spl_token::id(), false),
        ],
    );
}

impl TakerInstruction {
    #[throws(ProgramError)]
    pub fn unpack(input: &[u8]) -> Self {
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        match tag {
            0 => Self::DepositNFT {
                token_id: unpack_pubkey(rest)?.0,
            },
            _ => throw!(InvalidInstruction),
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::DepositNFT { token_id } => {
                buf.push(0);
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
