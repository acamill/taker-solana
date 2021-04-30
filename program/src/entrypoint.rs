use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

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
