#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::TokenAccount;
use fehler::throw;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

// pub mod error;
// pub mod instruction;
// mod spl_helper;
// mod state;
// pub mod native_mint;
// pub mod processor;
// pub mod state;

// #[cfg(not(feature = "no-entrypoint"))]
// mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
// pub use solana_program;

// The contract account should have address find_program_address(&[seed], program_id)
#[account]
pub struct TakerContract {
    pub seed: Vec<u8>,
    pub bump_seed: u8, // store the bump seed so we don't need to call find_program_address every time.
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
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: [u8; 32]) -> Result<()> {
        let (_, bump_seed) = Pubkey::find_program_address(&[&seed[..]], ctx.program_id);

        let contract = &mut ctx.accounts.contract_account;

        // Create accounts for this contract on tkr, tai and dai
        anchor_spl::token::initialize_account(CpiContext::new(
            ctx.accounts.spl_program.clone(),
            anchor_spl::token::InitializeAccount {
                account: ctx.accounts.tai_token.to_account_info(),
                mint: ctx.accounts.tai_mint.clone(),
                authority: ctx.accounts.authority.clone(),
            },
        ))?;
        anchor_spl::token::initialize_account(CpiContext::new(
            ctx.accounts.spl_program.clone(),
            anchor_spl::token::InitializeAccount {
                account: ctx.accounts.tkr_token.to_account_info(),
                mint: ctx.accounts.tkr_mint.clone(),
                authority: ctx.accounts.authority.clone(),
            },
        ))?;
        anchor_spl::token::initialize_account(CpiContext::new(
            ctx.accounts.spl_program.clone(),
            anchor_spl::token::InitializeAccount {
                account: ctx.accounts.dai_token.to_account_info(),
                mint: ctx.accounts.dai_mint.clone(),
                authority: ctx.accounts.authority.clone(),
            },
        ))?;

        contract.authority = *ctx.accounts.authority.key;
        contract.seed = seed.to_vec();
        contract.bump_seed = bump_seed;
        contract.tkr_mint = *ctx.accounts.tkr_mint.key;
        contract.tai_mint = *ctx.accounts.tai_mint.key;
        contract.dai_mint = *ctx.accounts.dai_mint.key;

        contract.deposit_incentive = 100;
        contract.max_loan_duration = 30;

        // 5%
        contract.service_fee_rate = 500;
        // 1%
        contract.interest_rate = 100;
        contract.total_num_loans = 0;

        Ok(())
    }

    pub fn deposit_nft(ctx: Context<DepositNFT>) -> Result<()> {
        let accounts = ctx.accounts;
        let contract = &mut accounts.contract_account;
        assert!(accounts.tkr_src.mint == contract.tkr_mint);

        anchor_spl::token::transfer(
            CpiContext::new(
                accounts.spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: accounts.nft_src.to_account_info(),
                    to: accounts.nft_dst.to_account_info(),
                    authority: accounts.user_authority.clone(),
                },
            ),
            1,
        )?;

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                accounts.spl_program.clone(),
                anchor_spl::token::Transfer {
                    from: accounts.tkr_src.to_account_info(),
                    to: accounts.tkr_dst.to_account_info(),
                    authority: contract.to_account_info(),
                },
                &[&[&contract.authority.to_bytes()[..], &[contract.bump_seed]]],
            ),
            contract.deposit_incentive,
        )?;

        // contract
        //     .nft_ownership
        //     .insert(*accounts.nft_mint.key, *accounts.user_authority.key);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    pub contract_account: ProgramAccount<'info, TakerContract>,

    #[account(signer)]
    pub authority: AccountInfo<'info>, // also the funder
    pub tkr_mint: AccountInfo<'info>,
    pub tkr_token: CpiAccount<'info, TokenAccount>,
    pub tai_mint: AccountInfo<'info>,
    pub tai_token: CpiAccount<'info, TokenAccount>,
    pub dai_mint: AccountInfo<'info>,
    pub dai_token: CpiAccount<'info, TokenAccount>,
    pub spl_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositNFT<'info> {
    #[account(mut)]
    pub contract_account: ProgramAccount<'info, TakerContract>,

    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    pub nft_mint: AccountInfo<'info>,
    pub nft_src: CpiAccount<'info, TokenAccount>,
    pub nft_dst: CpiAccount<'info, TokenAccount>,

    pub tkr_src: CpiAccount<'info, TokenAccount>,
    pub tkr_dst: CpiAccount<'info, TokenAccount>,
    pub spl_program: AccountInfo<'info>,
}

#[error]
pub enum TakerError {
    #[msg("Not Authorized")]
    NotAuhorized,
}

#[event]
pub struct CalledInitialize {}
