use crate::{TakerContract, TakerError};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use fehler::{throw, throws};
use solana_program::{instruction::Instruction, program::invoke_signed};
use solana_program::{program::invoke, system_instruction};

#[throws(ProgramError)]
pub fn verify_contract_address(program_id: &Pubkey, seeds_with_bump: &[&[u8]], account: &Pubkey) {
    let addr = Pubkey::create_program_address(seeds_with_bump, program_id)?;

    if &addr != account {
        throw!(TakerError::ContractAddressNotCorrect);
    }
}

#[throws(ProgramError)]
pub fn create_rent_exempt_account<'info>(
    program_id: &Pubkey, // The program ID of Taker Contract
    funder: &AccountInfo<'info>,
    account: &AccountInfo<'info>,
    seeds_with_bump: &[&[u8]],
    owner: &Pubkey,
    acc_size: u64,
    rent: &Sysvar<'info, Rent>,
    system: &AccountInfo<'info>,
) {
    let required_lamports = rent.minimum_balance(acc_size as usize).max(1);

    invoke_signed(
        &system_instruction::create_account(
            funder.key,
            account.key,
            required_lamports,
            acc_size,
            program_id,
        ),
        &[funder.clone(), account.clone(), system.clone()],
        &[seeds_with_bump],
    )?;
}

#[throws(ProgramError)]
pub fn create_associated_token_account<'info>(
    wallet: &ProgramAccount<'info, TakerContract>,
    funder: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    ata_program: &AccountInfo<'info>,
    spl_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent: &Sysvar<'info, Rent>,
) {
    // Accounts expected by this instruction:
    //
    //   0. `[writeable,signer]` Funding account (must be a system account)
    //   1. `[writeable]` Associated token account address to be created
    //   2. `[]` Wallet address for the new associated token account
    //   3. `[]` The token mint for the new associated token account
    //   4. `[]` System program
    //   5. `[]` SPL Token program
    //   6. `[]` Rent sysvar
    let ix = Instruction {
        program_id: *ata_program.key,
        accounts: vec![
            AccountMeta::new(*funder.key, true),
            AccountMeta::new(*token_account.key, false),
            AccountMeta::new_readonly(*wallet.to_account_info().key, false),
            AccountMeta::new_readonly(*mint.key, false),
            AccountMeta::new_readonly(*system_program.key, false),
            AccountMeta::new_readonly(*spl_program.key, false),
            AccountMeta::new_readonly(*rent.to_account_info().key, false),
        ],
        data: vec![],
    };

    invoke(
        &ix,
        &[
            funder.to_account_info(),
            token_account.to_account_info(),
            wallet.to_account_info(),
            mint.clone(),
            system_program.clone(),
            spl_program.clone(),
            rent.to_account_info(),
        ],
    )?;
}
