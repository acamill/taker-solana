#![forbid(unsafe_code)]

mod nft_bid;
mod nft_listing;
mod nft_loan;
mod nft_pool;
mod utils;

use std::u64;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use fehler::throw;
use nft_loan::LoanState;
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};

pub trait DerivedAccountIdentifier {
    const SEED: &'static [u8];
}

// The contract account should have address find_program_address(&[seed], program_id)
#[account]
#[derive(Debug)]
pub struct NFTPool {
    pub bump_seed: u8,
    pub authority: Pubkey,
    pub tkr_mint: Pubkey,
    pub tai_mint: Pubkey,
    pub dai_mint: Pubkey,
    pub deposit_incentive: u64,
    pub max_loan_duration: i64,
    pub service_fee_rate: u64, // in bp, one ten thousandth
    pub interest_rate: u64,    // in bp, one ten thousandth
}

#[account]
#[derive(Debug)]
pub struct NFTListing {
    pub count: u64,
    pub available: u64, // how many are available to be withdrawn or used as collateral
}

#[account]
#[derive(Debug)]
pub struct NFTBid {
    pub price: u64, // DAI Price
    pub qty: u64,
}

#[account]
#[derive(Debug)]
pub struct NFTLoan {
    cash: u64,                 // amount of dai
    started_at: UnixTimestamp, // in seconds
    expired_at: UnixTimestamp, // in seconds
    state: LoanState,
}

#[program]
pub mod taker {
    use super::*;

    pub fn initialize(ctx: Context<AccountsInitialize>) -> Result<()> {
        let (_, bump) = NFTPool::get_address_with_bump(ctx.program_id);

        let accounts = &ctx.accounts;

        NFTPool::verify_address(&ctx.program_id, bump, &accounts.pool.key)?;

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

    pub fn deposit_nft(ctx: Context<AccountsDepositNFT>, count: u64) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        let AccountsDepositNFT {
            pool,
            user_wallet_account,
            nft_mint,
            user_nft_account,
            pool_nft_account,
            tkr_mint,
            pool_tkr_account,
            user_tkr_account,
            ata_program,
            spl_program,
            system,
            rent,
            listing_account,
        } = ctx.accounts;

        assert_eq!(tkr_mint.to_account_info().key, &pool.tkr_mint);
        assert_eq!(pool_tkr_account.mint, pool.tkr_mint);
        assert_eq!(nft_mint.decimals, 0);

        // allocate the nft ata for the pool if not allocate
        NFTPool::ensure_pool_token_account(
            pool,
            nft_mint,
            pool_nft_account,
            user_wallet_account,
            ata_program,
            spl_program,
            system,
            rent,
        )?;

        // allocate the tkr ata for the user if not allocate
        NFTPool::ensure_user_token_account(
            user_wallet_account,
            tkr_mint,
            user_tkr_account,
            ata_program,
            spl_program,
            system,
            rent,
        )?;

        anchor_spl::token::transfer(
            CpiContext::new(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: user_nft_account.to_account_info(),
                    to: pool_nft_account.clone(),
                    authority: user_wallet_account.clone(),
                },
            ),
            count,
        )?;

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: pool_tkr_account.to_account_info(),
                    to: user_tkr_account.clone(),
                    authority: pool.to_account_info(),
                },
                &[&[&[pool.bump_seed]]],
            ),
            count * pool.deposit_incentive,
        )?;

        // create the listing account if not created
        let mut listing = NFTListing::ensure(
            ctx.program_id,
            nft_mint.to_account_info().key,
            user_wallet_account,
            listing_account,
            rent,
            system,
        )?;
        listing.deposit(count);

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
        let (_, bump) = NFTListing::get_address_with_bump(
            ctx.program_id,
            nft_mint.to_account_info().key,
            user_wallet_account.key,
        );
        NFTListing::verify_address(
            ctx.program_id,
            nft_mint.to_account_info().key,
            user_wallet_account.key,
            bump,
            listing_account.to_account_info().key,
        )?;

        // withdraw also verifies the count
        listing_account.withdraw(count)?;

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

    pub fn bid(ctx: Context<AccountsBid>, price: u64, qty: u64) -> Result<()> {
        if qty == 0 {
            return Ok(());
        }

        let AccountsBid {
            pool,
            user_wallet_account,
            nft_mint,
            user_dai_account,
            bid_account,
            spl_program,
            system,
            rent,
        } = ctx.accounts;

        if qty > nft_mint.supply {
            throw!(TakerError::NFTOverbid);
        }

        assert_eq!(nft_mint.decimals, 0);

        anchor_spl::token::approve(
            CpiContext::new(
                spl_program.clone(),
                anchor_spl::token::Approve {
                    to: user_dai_account.to_account_info(),
                    delegate: pool.to_account_info(),
                    authority: user_wallet_account.to_account_info(),
                },
            ),
            price * qty,
        )?;

        // create the listing account if not created
        let mut bid_account = NFTBid::ensure(
            ctx.program_id,
            nft_mint.to_account_info().key,
            user_wallet_account,
            bid_account,
            rent,
            system,
        )?;
        bid_account.bid(price, qty);

        // Persistent back the data. Since we created the ProgramAccount by ourselves, we need to do this manually.
        bid_account.exit(ctx.program_id)?;
        Ok(())
    }

    pub fn cancel_bid(ctx: Context<AccountsCancelBid>, revoke: bool) -> Result<()> {
        let AccountsCancelBid {
            user_wallet_account,
            nft_mint,
            user_dai_account,
            bid_account,
            spl_program,
        } = ctx.accounts;

        assert_eq!(nft_mint.decimals, 0);

        let (_, bump) = NFTBid::get_address_with_bump(
            ctx.program_id,
            nft_mint.to_account_info().key,
            user_wallet_account.key,
        );
        NFTBid::verify_address(
            ctx.program_id,
            nft_mint.to_account_info().key,
            user_wallet_account.key,
            bump,
            bid_account.to_account_info().key,
        )?;

        bid_account.cancel();

        if revoke {
            solana_program::program::invoke(
                &spl_token::instruction::revoke(
                    &spl_token::id(),
                    user_dai_account.to_account_info().key,
                    user_wallet_account.to_account_info().key,
                    &[user_wallet_account.key],
                )?,
                &[
                    user_dai_account.to_account_info(),
                    user_wallet_account.to_account_info(),
                    spl_program.clone(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn borrow(ctx: Context<AccountsBorrow>, loan_id: Pubkey, amount: u64) -> Result<()> {
        let AccountsBorrow {
            pool,
            borrower_wallet_account,
            lender_wallet_account,
            nft_mint,
            borrower_dai_account,
            lender_dai_account,
            bid_account,
            listing_account,
            loan_account,
            spl_program,
            system,
            rent,
            clock,
        } = ctx.accounts;

        if amount > bid_account.price {
            throw!(TakerError::NFTBorrowExceedBid)
        }

        // transfer DAI to the borrower
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: lender_dai_account.to_account_info(),
                    to: borrower_dai_account.to_account_info(),
                    authority: pool.to_account_info(), // The pool is the delegate
                },
                &[&[&[pool.bump_seed]]],
            ),
            amount,
        )?;

        // decrease the bid qty by 1;
        bid_account.trade(1)?;
        // descrease the availability by 1
        listing_account.borrow_success()?;

        let loan_account = NFTLoan::start_borrow(
            ctx.program_id,
            &loan_id,
            nft_mint.to_account_info().key,
            borrower_wallet_account,
            lender_wallet_account,
            loan_account,
            rent,
            system,
            amount,
            clock.unix_timestamp,
            pool.max_loan_duration,
        )?;

        loan_account.exit(ctx.program_id)?; // manually exit for data persistent
        Ok(())
    }

    pub fn repay(ctx: Context<AccountsRepay>) -> Result<()> {
        let AccountsRepay {
            pool,
            borrower_wallet_account,
            borrower_dai_account,
            lender_dai_account,
            pool_dai_account,
            listing_account,
            loan_account,
            spl_program,
            clock,
        } = ctx.accounts;

        if clock.unix_timestamp > loan_account.expired_at
            || matches!(loan_account.state, LoanState::Liquidated)
        {
            throw!(TakerError::LoanLiquidated)
        }

        // transfer DAI to the lender with interest
        anchor_spl::token::transfer(
            CpiContext::new(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: borrower_dai_account.to_account_info(),
                    to: lender_dai_account.to_account_info(),
                    authority: borrower_wallet_account.to_account_info(),
                },
            ),
            loan_account
                .cash
                .checked_add(
                    loan_account
                        .cash
                        .checked_mul(pool.interest_rate)
                        .unwrap()
                        .checked_div(10000)
                        .unwrap(),
                )
                .unwrap(),
        )?;

        anchor_spl::token::transfer(
            CpiContext::new(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: borrower_dai_account.to_account_info(),
                    to: pool_dai_account.to_account_info(),
                    authority: borrower_wallet_account.to_account_info(),
                },
            ),
            loan_account
                .cash
                .checked_mul(pool.service_fee_rate)
                .unwrap()
                .checked_div(10000)
                .unwrap(),
        )?;

        // set corresponding records
        loan_account.repay()?;
        listing_account.repay_success();

        Ok(())
    }

    pub fn liquidate(ctx: Context<AccountsLiquidate>) -> Result<()> {
        let AccountsLiquidate {
            pool,
            lender_wallet_account,

            nft_mint,
            pool_nft_account,
            lender_nft_account,

            listing_account,
            loan_account,

            ata_program,
            spl_program,
            system,
            rent,
            clock,
        } = ctx.accounts;

        if !matches!(loan_account.state, LoanState::Active) {
            throw!(TakerError::LoanNotActive)
        }

        if clock.unix_timestamp <= loan_account.expired_at {
            throw!(TakerError::LoanNotExpired)
        }

        // allocate the NFT ATA for the lender if not allocate
        NFTPool::ensure_user_token_account(
            lender_wallet_account,
            nft_mint,
            lender_nft_account,
            ata_program,
            spl_program,
            system,
            rent,
        )?;

        // Transfer the NFT to the lender
        anchor_spl::token::transfer(
            CpiContext::new(
                spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: pool_nft_account.to_account_info(),
                    to: lender_nft_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
            ),
            1,
        )?;

        // set corresponding records
        loan_account.liquidate()?;
        listing_account.liquidate(1)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AccountsInitialize<'info> {
    #[account(signer)]
    pub pool_owner: AccountInfo<'info>, // also the funder
    #[account(mut)]
    pub pool: AccountInfo<'info>, // We cannot use  ProgramAccount<'info, TakerContract> here because it is not allocated yet

    pub tkr_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_tkr_account: AccountInfo<'info>, // this is not allocated yet

    pub tai_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_tai_account: AccountInfo<'info>, // this is not allocated yet

    pub dai_mint: CpiAccount<'info, Mint>,
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

    pub nft_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub user_nft_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_nft_account: AccountInfo<'info>, // potentially this is not allocated yet

    pub tkr_mint: CpiAccount<'info, Mint>,
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

    pub nft_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_nft_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_nft_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub listing_account: ProgramAccount<'info, NFTListing>,

    pub spl_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AccountsBid<'info> {
    pub pool: ProgramAccount<'info, NFTPool>,
    #[account(signer)]
    pub user_wallet_account: AccountInfo<'info>,

    pub nft_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub user_dai_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub bid_account: AccountInfo<'info>, // Essentially this is ProgramAccount<NFTBid>, however, we've not allocated the space for it yet. We cannot use ProgramAccount here.

    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AccountsCancelBid<'info> {
    #[account(signer)]
    pub user_wallet_account: AccountInfo<'info>,

    pub nft_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub user_dai_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub bid_account: ProgramAccount<'info, NFTBid>,

    pub spl_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AccountsBorrow<'info> {
    pub pool: ProgramAccount<'info, NFTPool>,
    #[account(signer)]
    pub borrower_wallet_account: AccountInfo<'info>,
    pub lender_wallet_account: AccountInfo<'info>,

    pub nft_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub borrower_dai_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub lender_dai_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub listing_account: ProgramAccount<'info, NFTListing>,
    #[account(mut)]
    pub bid_account: ProgramAccount<'info, NFTBid>,
    #[account(mut)]
    pub loan_account: AccountInfo<'info>, // potentially not allocated

    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct AccountsRepay<'info> {
    pub pool: ProgramAccount<'info, NFTPool>,
    #[account(signer)]
    pub borrower_wallet_account: AccountInfo<'info>,

    #[account(mut)]
    pub borrower_dai_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub lender_dai_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_dai_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub listing_account: ProgramAccount<'info, NFTListing>,
    #[account(mut)]
    pub loan_account: ProgramAccount<'info, NFTLoan>,

    pub spl_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct AccountsLiquidate<'info> {
    pub pool: ProgramAccount<'info, NFTPool>,

    #[account(signer)]
    pub lender_wallet_account: AccountInfo<'info>,

    pub nft_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_nft_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub lender_nft_account: AccountInfo<'info>, // Possibly not allocated

    #[account(mut)]
    pub listing_account: ProgramAccount<'info, NFTListing>,
    #[account(mut)]
    pub loan_account: ProgramAccount<'info, NFTLoan>,

    pub ata_program: AccountInfo<'info>,
    pub spl_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[error]
pub enum TakerError {
    #[msg("Not Authorized")]
    NotAuhorized,
    #[msg("Contract address not correct")]
    ContractAddressNotCorrect,

    #[msg("NFT listing address not correct")]
    NFTListingAddressNotCorrect,

    #[msg("NFT bid address not correct")]
    NFTBidAddressNotCorrect,

    #[msg("NFT loan address not correct")]
    NFTLoanAddressNotCorrect,

    #[msg("NFT overdrawn")]
    NFTOverdrawn,

    #[msg("Empty NFT Reserve")]
    EmptyNFTReserve,

    #[msg("NFT overtrade")]
    NFTOvertrade,

    #[msg("NFT bid amount larger than NFT supply")]
    NFTOverbid,

    #[msg("NFT borrow amount larger than bid amount")]
    NFTBorrowExceedBid,

    #[msg("NFT borrow already started")]
    BorrowAlreadyStarted,

    #[msg("Loan is liquidated")]
    LoanLiquidated,

    #[msg("Loan is not expired yet")]
    LoanNotExpired,

    #[msg("Loan record already exist")]
    LoanAlreadyExist,

    #[msg("Loan already finalized")]
    LoanFinialized,

    #[msg("Loan is not active")]
    LoanNotActive,
}

#[event]
#[derive(Debug)]
pub struct EventContractInitialized {}

#[event]
#[derive(Debug)]
pub struct EventContractAllocated {
    addr: Pubkey,
}
