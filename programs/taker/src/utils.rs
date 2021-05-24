use crate::{TakerContract, TakerError};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use fehler::{throw, throws};
use solana_program::{instruction::Instruction, program::invoke_signed, system_program};
use solana_program::{program::invoke, system_instruction};

// An program derived account that stores nft listing
// The address of the account is computed as follow:
// address = find_program_address([nft_mint_address, user_wallet_address], program_id)
// only the taker_contract_address can change the data in this account
pub fn get_nft_listing_address(program_id: &Pubkey, nft_mint: &Pubkey, wallet: &Pubkey) -> Pubkey {
    get_nft_listing_address_with_bump(program_id, nft_mint, wallet).0
}

pub(crate) fn get_nft_listing_address_with_bump(
    program_id: &Pubkey,
    nft_mint: &Pubkey,
    wallet: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&nft_mint.to_bytes(), &wallet.to_bytes()], program_id)
}

#[throws(ProgramError)]
pub fn verify_nft_listing_address(
    program_id: &Pubkey,
    nft_mint: &Pubkey,
    wallet: &Pubkey,
    bump: u8,
    listing_address: &Pubkey,
) {
    let addr = Pubkey::create_program_address(
        &[&nft_mint.to_bytes(), &wallet.to_bytes(), &[bump]],
        program_id,
    )?;

    if &addr != listing_address {
        throw!(TakerError::NFTListingAddressNotCorrect);
    }
}

pub fn get_pool_address(program_id: &Pubkey) -> Pubkey {
    get_pool_address_with_bump(program_id).0
}

pub(crate) fn get_pool_address_with_bump(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[], program_id)
}

#[throws(ProgramError)]
pub fn verify_pool_address(program_id: &Pubkey, bump: u8, pool_address: &Pubkey) {
    let addr = Pubkey::create_program_address(&[&[bump]], program_id)?;

    if &addr != pool_address {
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
    wallet: &AccountInfo<'info>,
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
            AccountMeta::new_readonly(*wallet.key, false),
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
            wallet.clone(),
            mint.clone(),
            system_program.clone(),
            spl_program.clone(),
            rent.to_account_info(),
        ],
    )?;
}

pub fn is_account_allocated(acc: &AccountInfo) -> bool {
    //    if the account has non zero lamports or has data stored or has the owner != system_program, then this account is already allocated
    acc.lamports() != 0 || !acc.data_is_empty() || !system_program::check_id(&acc.owner)
}
