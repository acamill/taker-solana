use crate::EventCreateAccount;
use anchor_lang::prelude::*;
use fehler::throws;
use solana_program::instruction::Instruction;
use solana_program::{program::invoke, system_instruction};

#[throws(ProgramError)]
pub fn create_rent_exempt_account<'info>(
    owner: Pubkey,
    account: AccountInfo<'info>,
    funder: AccountInfo<'info>,
    acc_size: u64,
    rent: AccountInfo<'info>,
    system: AccountInfo<'info>,
) {
    let rent = &Rent::from_account_info(&rent)?;

    let required_lamports = rent.minimum_balance(acc_size as usize).max(1);

    emit!(EventCreateAccount {
        addr: *account.key,
        lamport: required_lamports
    });

    invoke(
        &system_instruction::create_account(
            funder.key,
            account.key,
            required_lamports,
            acc_size,
            &owner,
        ),
        &[funder, account, system],
    )?;
}

#[throws(ProgramError)]
pub fn create_associated_token_account<'info>(
    wallet: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token_address: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    system: AccountInfo<'info>,
    rent: AccountInfo<'info>,
) {
    let ix = Instruction {
        program_id: *token_program.key,
        accounts: vec![
            AccountMeta::new(*authority.key, true),
            AccountMeta::new(*token_address.key, false),
            AccountMeta::new_readonly(*wallet.key, false),
            AccountMeta::new_readonly(*mint.key, false),
            AccountMeta::new_readonly(*system.key, false),
            AccountMeta::new_readonly(*token_program.key, false),
            AccountMeta::new_readonly(*rent.key, false),
        ],
        data: vec![],
    };

    invoke(
        &ix,
        &[
            authority,
            token_address,
            wallet,
            mint,
            system,
            token_program,
            rent,
        ],
    )?;
}
