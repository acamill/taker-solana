#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

mod impls;
mod utils;

use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::TokenAccount;
use fehler::throw;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::convert::TryInto;
pub use utils::*;

// The contract account should have address find_program_address(&[seed], program_id)
#[account]
pub struct TakerContract {
    pub bump_seed: u8,
    pub authority: Pubkey,
    pub tkr_mint: Pubkey,
    pub tai_mint: Pubkey,
    pub dai_mint: Pubkey,
    pub deposit_incentive: u64,
    pub max_loan_duration: u64,
    pub service_fee_rate: u64,
    pub interest_rate: u64,
    // Total number of loans generated
    pub total_num_loans: u64,
}

#[program]
pub mod taker {
    use solana_program::{program::invoke_signed, system_instruction};

    use super::*;

    pub fn initialize(ctx: Context<AccountsInitialize>) -> Result<()> {
        let (_, bump) = get_pool_address_with_bump(ctx.program_id);

        let accounts = &ctx.accounts;

        utils::verify_pool_address(&ctx.program_id, bump, &accounts.this.key)?;

        let this = TakerContract::new(&ctx, bump)?;

        emit!(EventContractAllocated {
            addr: *this.to_account_info().key
        });

        // Create accounts for this contract on tkr, tai and dai
        for (mint, token) in &[
            (&accounts.dai_mint, &accounts.dai_token),
            (&accounts.tkr_mint, &accounts.tkr_token),
            (&accounts.tai_mint, &accounts.tai_token),
        ] {
            utils::create_associated_token_account(
                &this.to_account_info(),
                &accounts.authority,
                mint,
                token,
                &accounts.ata_program,
                &accounts.spl_program,
                &accounts.system,
                &accounts.rent,
            )?;
        }

        emit!(EventContractInitialized {});

        Ok(())
    }

    pub fn deposit_nft(ctx: Context<AccountsDepositNFT>) -> Result<()> {
        let AccountsDepositNFT {
            this,
            user_authority,
            nft_mint,
            nft_src,
            nft_dst,
            tkr_mint,
            tkr_src,
            tkr_dst,
            ata_program,
            spl_program,
            system,
            rent,
            listing,
        } = ctx.accounts;

        let seeds_with_bump: &[&[_]] = &[&[this.bump_seed]];

        assert_eq!(tkr_mint.key, &this.tkr_mint);
        assert_eq!(tkr_src.mint, this.tkr_mint);

        // allocate the nft ata for the pool if not allocate
        if !is_account_allocated(nft_dst) {
            utils::create_associated_token_account(
                &this.to_account_info(),
                &user_authority,
                &nft_mint,
                &nft_dst,
                &ata_program,
                &spl_program,
                &system,
                &rent,
            )?;
        }

        // allocate the tkr ata for the user if not allocate
        if !is_account_allocated(tkr_dst) {
            utils::create_associated_token_account(
                &user_authority,
                &user_authority,
                &tkr_mint,
                &tkr_dst,
                &ata_program,
                &spl_program,
                &system,
                &rent,
            )?;
        }

        anchor_spl::token::transfer(
            CpiContext::new(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: nft_src.to_account_info(),
                    to: nft_dst.clone(),
                    authority: user_authority.clone(),
                },
            ),
            1,
        )?;

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: tkr_src.to_account_info(),
                    to: tkr_dst.clone(),
                    authority: this.to_account_info(),
                },
                &[seeds_with_bump],
            ),
            this.deposit_incentive,
        )?;

        // create the listing account if not created
        let (_, bump) =
            get_nft_listing_address_with_bump(ctx.program_id, nft_mint.key, user_authority.key);

        verify_nft_listing_address(
            ctx.program_id,
            nft_mint.key,
            user_authority.key,
            bump,
            listing.key,
        )?;

        if !is_account_allocated(listing) {
            let seeds_with_bump_for_listing: &[&[_]] = &[
                &nft_mint.key.to_bytes(),
                &user_authority.key.to_bytes(),
                &[bump],
            ];
            utils::create_rent_exempt_account(
                ctx.program_id,
                user_authority,
                listing,
                seeds_with_bump_for_listing,
                ctx.program_id,
                8,
                &rent,
                &system,
            )?;
        }

        let mut data = listing.try_borrow_mut_data()?;
        let bytes: [u8; 8] = (&**data).try_into().unwrap();
        let n = u64::from_le_bytes(bytes);
        data.copy_from_slice(&(n + 1).to_le_bytes());

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AccountsInitialize<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>, // also the funder
    #[account(mut)]
    pub this: AccountInfo<'info>, // We cannot use  ProgramAccount<'info, TakerContract> here because it is not allocated yet

    pub tkr_mint: AccountInfo<'info>,
    #[account(mut)]
    pub tkr_token: AccountInfo<'info>,

    pub tai_mint: AccountInfo<'info>,
    #[account(mut)]
    pub tai_token: AccountInfo<'info>,

    pub dai_mint: AccountInfo<'info>,
    #[account(mut)]
    pub dai_token: AccountInfo<'info>,

    pub ata_program: AccountInfo<'info>,
    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AccountsDepositNFT<'info> {
    pub this: ProgramAccount<'info, TakerContract>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,

    pub nft_mint: AccountInfo<'info>,
    #[account(mut)]
    pub nft_src: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub nft_dst: AccountInfo<'info>, // potentially this is not allocated yet

    pub tkr_mint: AccountInfo<'info>,
    #[account(mut)]
    pub tkr_src: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub tkr_dst: AccountInfo<'info>, // potentially this is not allocated yet

    #[account(mut)]
    pub listing: AccountInfo<'info>,

    pub ata_program: AccountInfo<'info>,
    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[error]
pub enum TakerError {
    #[msg("Not Authorized")]
    NotAuhorized,
    #[msg("Contract address not correct")]
    ContractAddressNotCorrect,
    #[msg("NFT listing address not correct")]
    NFTListingAddressNotCorrect,
}

#[event]
#[derive(Debug)]
pub struct EventContractInitialized {}

#[event]
#[derive(Debug)]
pub struct EventContractAllocated {
    addr: Pubkey,
}
