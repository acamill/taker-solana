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

// The contract account should have address find_program_address(&[seed], program_id)
#[account]
pub struct TakerContract {
    pub seed: Vec<u8>,
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

    pub fn initialize(ctx: Context<AccountsInitialize>, seed: [u8; 32], bump: u8) -> Result<()> {
        let seeds_with_bump = &[&seed[..], &[bump]];

        let accounts = &ctx.accounts;

        utils::verify_contract_address(&ctx.program_id, seeds_with_bump, &accounts.this.key)?;

        let this = TakerContract::new(&ctx, &seed[..], bump)?;

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
                &this,
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

    // The NFT associated account for this contract must be already created
    pub fn deposit_nft(ctx: Context<AccountsDepositNFT>) -> Result<()> {
        let accounts = ctx.accounts;
        let contract = &mut accounts.contract_account;
        let seeds_with_bump = &[&contract.seed[..], &[contract.bump_seed]];

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
                &[seeds_with_bump],
            ),
            contract.deposit_incentive,
        )?;

        // now
        // contract
        //     .nft_ownership
        //     .insert(*accounts.nft_mint.key, *accounts.user_authority.key);
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
pub struct AccountsInitializeNFTAccount<'info> {
    #[account(signer)]
    pub funder: AccountInfo<'info>, // the funder
    pub contract: ProgramAccount<'info, TakerContract>,

    pub nft_mint: AccountInfo<'info>,
    #[account(mut)]
    pub nft_token: AccountInfo<'info>,

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
