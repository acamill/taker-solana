//! Program state processor

use crate::{error::TakerError, instruction::TakerInstruction};
use fehler::throws;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke,
    program_error::{PrintProgramError, ProgramError},
    program_option::COption,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::TokenInstruction;

// Reward each NFT deposit with 100 TKR
static NFT_DEPOSIT_REWARD: u64 = 100;

/// Program state handler.
pub struct Processor {}

impl Processor {
    #[throws(ProgramError)]
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) {
        let instruction = TakerInstruction::unpack(input)?;
        msg!("[Invoke] {:?}", instruction);

        match instruction {
            TakerInstruction::DepositNFT { ref token_id } => {
                process_deposit(program_id, token_id, accounts)?
            }
        }
    }
}

/// Accounts expected by this instruction:
///
///   0. `[s]` The user wallet account.
///   1. `[w]` The user NFT account.
///   2. `[w]` The taker NFT account.
///   3. `[w]` The user TKR account.
///   4. `[w]` The taker TKR account.
///   5. `[]` The SPL-Token Program Account.
///
#[throws(ProgramError)]
fn process_deposit(program_id: &Pubkey, token_id: &Pubkey, accounts: &[AccountInfo]) {
    let accounts = &mut accounts.iter();

    // Get accounts
    let user_wallet_account = next_account_info(accounts)?;
    let user_nft_account = next_account_info(accounts)?;
    let taker_nft_account = next_account_info(accounts)?;
    let user_tkr_account = next_account_info(accounts)?;
    let taker_tkr_account = next_account_info(accounts)?;
    let spl_token_account = next_account_info(accounts)?;

    // Call the SPL-Token program to transfer the NFT to Taker Protocol

    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            user_nft_account.key,
            taker_nft_account.key,
            user_wallet_account.key,
            &[],
            1,
        )?,
        &vec![
            spl_token_account.clone(),
            user_wallet_account.clone(),
            user_nft_account.clone(),
            taker_nft_account.clone(),
        ],
    )?;

    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            taker_tkr_account.key,
            user_tkr_account.key,
            program_id,
            &[],
            NFT_DEPOSIT_REWARD,
        )?,
        &vec![
            spl_token_account.clone(),
            taker_tkr_account.clone(),
            user_tkr_account.clone(),
            // taker_wallet_account.clone(),
        ],
    )?
}
