#![forbid(unsafe_code)]

use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::TokenAccount;
use fehler::throw;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

// The an associated account that stores nft listing
// The address of the account is computed as follow:
// address = find_program_address([taker_nft_pool_address, nft_mint_address, user_wallet_address], program_id)
// only the taker_contract_address can change the data in this account

pub fn taker_nft_listing_address(
    program_id: &Pubkey,
    taker_nft_pool: &Pubkey,
    nft_mint: &Pubkey,
    wallet: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(&[taker_nft_pool, nft_mint, wallet], program_id)
}

#[account]
pub struct TakerNFTListing {
    nft_pool: Pubkey,
    num: u64, // how many of this NFT asset is listed.
}

#[program]
pub mod taker {
    use super::*;

    // create the account for the contract account
    pub fn allocate(ctx: Context<Allocate>, seeds_with_bump: &[&[u8]]) -> Result<()> {
        let accounts = &ctx.accounts;
        let this = &accounts.this;
        let rent = &accounts.rent;
        let acc_size = 8 + TakerNFTListing {
            nft_pool: Pubkey::new_unique(),
            num: 0,
        }
        .try_to_vec()
        .map_err(|_| ProgramError::Custom(1))?
        .len() as u64;

        // allocate the space for this account
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

        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let this = ctx.accounts.this;
        let nft_pool = ctx.accounts.nft_pool;
        this.nft_pool = nft_pool.key;
        Ok(())
    }

    pub fn set_num(mut ctx: Context<SetNum>, num: u64) -> Result<()> {}
}

#[derive(Accounts)]
pub struct Allocate<'info> {
    #[account(mut)]
    pub this: AccountInfo<'info>,
    #[account(signer)]
    pub funder: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub this: ProgramAccount<'info, TakerNFTListing>,
    pub nft_pool: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetNum<'info> {
    #[account(mut)]
    pub this: ProgramAccount<'info, TakerNFTListing>,
    #[account(signer)]
    pub nft_pool: AccountInfo<'info>,
}
