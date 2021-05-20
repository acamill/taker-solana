use crate::state::contract_account::ContractAccount;
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
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::TokenInstruction;

/// Initialize the Contract Account
/// Accounts expected by this instruction:
///
///   1. `[s]` The fee payer for creating the Contract Account.
///   2. `[]` The TKR Mint.
///   3. `[]` The TAI Mint.
///   4. `[]` The DAI Mint.
///   5. `[w]` The Taker Contract Account (uninitialized).
///   6. `[]` System program
///   7. `[]` Rent Sysvar
///
#[throws(ProgramError)]
fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) {
    let accounts = &mut accounts.iter();

    // Get accounts
    let funder_info = next_account_info(accounts)?;
    let tkr_mint_info = next_account_info(accounts)?;
    let tai_mint_info = next_account_info(accounts)?;
    let dai_mint_info = next_account_info(accounts)?;
    let taker_info = next_account_info(accounts)?;
    let system_program_info = next_account_info(accounts)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    // Create the Contract Account
    let acc_size = ContractAccount::get_packed_len();

    invoke_signed(
        &system_instruction::create_account(
            funder_info.key,
            taker_info.key,
            1.max(rent.minimum_balance(acc_size)),
            acc_size as u64,
            &program_id,
        ),
        &[
            funder_info.clone(),
            taker_info.clone(),
            system_program_info.clone(),
        ],
        &[&distributor_token_signer_seeds],
    )?;
}
