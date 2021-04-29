#![forbid(unsafe_code)]

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};
use std::time::Instant;

/// Convert the UI representation of a token amount (using the decimals field defined in its mint)
/// to the raw amount
pub fn ui_amount_to_amount(ui_amount: f64, decimals: u8) -> u64 {
    (ui_amount * 10_usize.pow(decimals as u32) as f64) as u64
}

/// Convert a raw amount to its UI representation (usingls t the decimals field defined in its mint)
pub fn amount_to_ui_amount(amount: u64, decimals: u8) -> f64 {
    amount as f64 / 10_usize.pow(decimals as u32) as f64
}

solana_program::declare_id!("dYq3uiw4k91nbzZS7mAyVeV1o1XLDnepAiiyCbVgkpN");

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
    //     // catch the error so we can print it
    //     error.print::<TokenError>();
    //     return Err(error);
    // }

    msg!(
        instruction_data[0],
        instruction_data[1],
        instruction_data[2],
        instruction_data[3],
        instruction_data[4]
    );
    Ok(())
}
