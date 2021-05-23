use anchor_lang::prelude::*;
use fehler::throws;
use solana_program::{instruction::Instruction, program::invoke_signed};
use solana_program::{program::invoke, system_instruction};

#[throws(ProgramError)]
pub fn create_rent_exempt_account<'info>(
    program_id: &Pubkey, // The program ID of Taker Contract
    funder: &AccountInfo<'info>,
    account: &AccountInfo<'info>,
    seed: &[u8],
    owner: &Pubkey,
    acc_size: u64,
    rent: &Sysvar<'info, Rent>,
    system: &AccountInfo<'info>,
) {
    let (addr, bump_seed) = Pubkey::find_program_address(&[seed], program_id);
    assert_eq!(&addr, account.key);

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
        &[&[&seed[..], &[bump_seed]]],
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
