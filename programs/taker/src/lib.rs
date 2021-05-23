#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

mod utils;

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
    use solana_program::{program::invoke_signed, system_instruction};

    use super::*;

    // create the account for the contract account
    pub fn allocate(ctx: Context<AccountsAllocate>, seed: [u8; 32], bump: u8) -> Result<()> {
        let seeds_with_bump = &[&seed[..], &[bump]];

        let accounts = &ctx.accounts;
        let contract = &accounts.contract;
        let contract_key = contract.to_account_info().key;

        utils::verify_contract_address(&ctx.program_id, seeds_with_bump, contract_key)?;

        // allocate the space for the contract account
        utils::create_rent_exempt_account(
            ctx.program_id, // The program ID of Taker Contract
            &accounts.authority,
            &contract.to_account_info(),
            &[&seed[..], &[bump]],
            ctx.program_id,
            10240,
            &accounts.rent,
            &accounts.system,
        )?;

        emit!(EventContractAllocated {
            addr: *contract_key
        });

        Ok(())
    }

    pub fn initialize(
        mut ctx: Context<AccountsInitialize>,
        seed: [u8; 32],
        bump: u8,
    ) -> Result<()> {
        let seeds_with_bump = &[&seed[..], &[bump]];

        let accounts = &mut ctx.accounts;
        let contract = &mut accounts.contract;
        let contract_key = contract.to_account_info().key;

        utils::verify_contract_address(&ctx.program_id, seeds_with_bump, contract_key)?;

        // Create accounts for this contract on tkr, tai and dai
        utils::create_associated_token_account(
            &contract,
            &accounts.authority,
            &accounts.tai_mint,
            &accounts.tai_token,
            &accounts.ata_program,
            &accounts.spl_program,
            &accounts.system,
            &accounts.rent,
        )?;

        utils::create_associated_token_account(
            &contract,
            &accounts.authority,
            &accounts.dai_mint,
            &accounts.dai_token,
            &accounts.ata_program,
            &accounts.spl_program,
            &accounts.system,
            &accounts.rent,
        )?;

        utils::create_associated_token_account(
            &contract,
            &accounts.authority,
            &accounts.tkr_mint,
            &accounts.tkr_token,
            &accounts.ata_program,
            &accounts.spl_program,
            &accounts.system,
            &accounts.rent,
        )?;

        // set corresponding fields
        contract.authority = *accounts.authority.key;
        contract.seed = seed.to_vec();
        contract.bump_seed = bump;
        contract.tkr_mint = *accounts.tkr_mint.key;
        contract.tai_mint = *accounts.tai_mint.key;
        contract.dai_mint = *accounts.dai_mint.key;

        contract.deposit_incentive = 100;
        contract.max_loan_duration = 30;

        // 5%
        contract.service_fee_rate = 500;
        // 1%
        contract.interest_rate = 100;
        contract.total_num_loans = 0;

        emit!(EventContractInitialized {});

        Ok(())
    }

    pub fn deposit_nft(ctx: Context<AccountsDepositNFT>) -> Result<()> {
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
pub struct AccountsAllocate<'info> {
    #[account(mut)]
    pub contract: AccountInfo<'info>,
    #[account(signer)]
    pub authority: AccountInfo<'info>, // also the funder
    pub system: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AccountsInitialize<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>, // also the funder
    #[account(init)]
    pub contract: ProgramAccount<'info, TakerContract>,

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
    #[msg("Contract address not correct")]
    ContractAddressNotCorrect,
}

#[event]
#[derive(Debug)]
pub struct EventContractInitialized {}

#[event]
#[derive(Debug)]
pub struct EventContractAllocated {
    addr: Pubkey,
}
