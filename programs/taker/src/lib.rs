#![forbid(unsafe_code)]

mod nft_listing;
mod nft_pool;
mod utils;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use solana_program::pubkey::Pubkey;

pub use utils::*;

// The contract account should have address find_program_address(&[seed], program_id)
#[account]
pub struct NFTPool {
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

#[account]
pub struct NFTListing {
    pub count: u64,
}

#[program]
pub mod taker {
    use super::*;

    pub fn initialize(ctx: Context<AccountsInitialize>) -> Result<()> {
        let (_, bump) = get_pool_address_with_bump(ctx.program_id);

        let accounts = &ctx.accounts;

        utils::verify_pool_address(&ctx.program_id, bump, &accounts.pool.key)?;

        let pool = NFTPool::new(&ctx, bump)?;

        emit!(EventContractAllocated {
            addr: *pool.to_account_info().key
        });

        // Create accounts for this contract on tkr, tai and dai
        for (mint, token) in &[
            (&accounts.dai_mint, &accounts.pool_dai_account),
            (&accounts.tkr_mint, &accounts.pool_tkr_account),
            (&accounts.tai_mint, &accounts.pool_tai_account),
        ] {
            utils::create_associated_token_account(
                &pool.to_account_info(),
                &accounts.pool_owner,
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
            pool,
            user_wallet_account: user_authority,
            nft_mint,
            user_nft_account: nft_src,
            pool_nft_account: nft_dst,
            tkr_mint,
            pool_tkr_account: tkr_src,
            user_tkr_account: tkr_dst,
            ata_program,
            spl_program,
            system,
            rent,
            listing_account: listing,
        } = ctx.accounts;

        assert_eq!(tkr_mint.key, &pool.tkr_mint);
        assert_eq!(tkr_src.mint, pool.tkr_mint);

        // allocate the nft ata for the pool if not allocate
        if !is_account_allocated(nft_dst) {
            utils::create_associated_token_account(
                &pool.to_account_info(),
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
                    authority: pool.to_account_info(),
                },
                &[&[&[pool.bump_seed]]],
            ),
            pool.deposit_incentive,
        )?;

        // create the listing account if not created
        let mut listing = NFTListing::ensure(
            ctx.program_id,
            nft_mint.key,
            user_authority,
            listing,
            rent,
            system,
        )?;

        listing.count += 1;

        // Persistent back the data. Since we created the ProgramAccount by ourselves, we need to do this manually.
        listing.exit(ctx.program_id)?;
        Ok(())
    }

    pub fn withdraw_nft(ctx: Context<AccountsWithdrawNFT>, count: u64) -> Result<()> {
        // TODO: Do we set the minimal nft lock in time?
        let AccountsWithdrawNFT {
            pool,
            user_wallet_account,
            nft_mint,
            user_nft_account,
            pool_nft_account,
            listing_account,
            spl_program,
        } = ctx.accounts;

        // verify the listing account indeed belongs to the user
        let (_, bump) = get_nft_listing_address_with_bump(
            ctx.program_id,
            nft_mint.key,
            user_wallet_account.key,
        );
        verify_nft_listing_address(
            ctx.program_id,
            nft_mint.key,
            user_wallet_account.key,
            bump,
            listing_account.to_account_info().key,
        )?;

        // verify the user indeed listed `count` many NFTs
        assert!(listing_account.count >= count);
        listing_account.count -= count;

        // transfer the NFT back to the user
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: pool_nft_account.to_account_info(),
                    to: user_nft_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                &[&[&[pool.bump_seed]]],
            ),
            count,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AccountsInitialize<'info> {
    #[account(signer)]
    pub pool_owner: AccountInfo<'info>, // also the funder
    #[account(mut)]
    pub pool: AccountInfo<'info>, // We cannot use  ProgramAccount<'info, TakerContract> here because it is not allocated yet

    pub tkr_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_tkr_account: AccountInfo<'info>, // this is not allocated yet

    pub tai_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_tai_account: AccountInfo<'info>, // this is not allocated yet

    pub dai_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_dai_account: AccountInfo<'info>, // this is not allocated yet

    pub ata_program: AccountInfo<'info>,
    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AccountsDepositNFT<'info> {
    pub pool: ProgramAccount<'info, NFTPool>,
    #[account(signer)]
    pub user_wallet_account: AccountInfo<'info>,

    pub nft_mint: AccountInfo<'info>,
    #[account(mut)]
    pub user_nft_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_nft_account: AccountInfo<'info>, // potentially this is not allocated yet

    pub tkr_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_tkr_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_tkr_account: AccountInfo<'info>, // potentially this is not allocated yet

    #[account(mut)]
    pub listing_account: AccountInfo<'info>, // Essentially this is ProgramAccount<NFTListing>, however, we've not allocated the space for it yet. We cannot use ProgramAccount here.

    pub ata_program: AccountInfo<'info>,
    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AccountsWithdrawNFT<'info> {
    pub pool: ProgramAccount<'info, NFTPool>,
    #[account(signer)]
    pub user_wallet_account: AccountInfo<'info>,

    pub nft_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_nft_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_nft_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub listing_account: ProgramAccount<'info, NFTListing>,

    pub spl_program: AccountInfo<'info>,
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
