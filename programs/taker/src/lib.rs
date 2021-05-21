#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

use anchor_lang::prelude::*;
use anchor_spl::token;
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

#[program]
pub mod taker {
    use super::*;

    #[state]
    pub struct TakerContract {
        pub authority: Pubkey,
        pub tkr_mint: Pubkey,
        pub tai_mint: Pubkey,
        pub dai_mint: Pubkey,
        pub nft_ownership: HashMap<Pubkey, Pubkey>,
        pub nft_price_at_loan: HashMap<Pubkey, u64>,
        pub deposit_incentive: u64,
        pub max_loan_duration: u64,
        pub service_fee_rate: u64,
        pub interest_rate: u64,
        // Total number of loans generated
        pub total_num_loans: u64,
    }

    impl TakerContract {
        pub fn new(ctx: Context<New>) -> Result<Self> {
            Ok(Self {
                authority: *ctx.accounts.authority.key,
                tkr_mint: *ctx.accounts.tkr_mint.key,
                tai_mint: *ctx.accounts.tai_mint.key,
                dai_mint: *ctx.accounts.dai_mint.key,

                nft_ownership: HashMap::new(),
                nft_price_at_loan: HashMap::new(),
                deposit_incentive: 100,
                max_loan_duration: 30,
                // 5%
                service_fee_rate: 500,
                // 1%
                interest_rate: 100,
                total_num_loans: 0,
            })
        }

        pub fn update_tkr_mint(&mut self, ctx: Context<UpdateTkrMint>) -> Result<()> {
            if ctx.accounts.authority.key != &self.authority {
                throw!(TakerError::NotAuhorized)
            }
            self.tkr_mint = *ctx.accounts.mint.key;
            Ok(())
        }

        pub fn deposit_nft(&mut self) {}
    }
}

#[derive(Accounts)]
pub struct New<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub tkr_mint: AccountInfo<'info>,
    pub tai_mint: AccountInfo<'info>,
    pub dai_mint: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateTkrMint<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
}

#[error]
pub enum TakerError {
    #[msg("Not Authorized")]
    NotAuhorized,
}
