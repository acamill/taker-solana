#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

use anchor_lang::prelude::*;
use anchor_spl::dex;
use anchor_spl::dex::serum_dex::instruction::SelfTradeBehavior;
use anchor_spl::dex::serum_dex::matching::{OrderType, Side as SerumSide};
use anchor_spl::dex::serum_dex::state::MarketState;
use anchor_spl::token;

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

solana_program::declare_id!("4cpqXnc2LAMmwB9wwmRtCZtsx3H2pQXYTKVhhRhJHQiy");

#[program]
pub mod taker {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> ProgramResult {
        let my_account = &mut ctx.accounts.my_account;
        my_account.data = data;
        Ok(())
    }

    pub fn update(ctx: Context<Update>, data: u64) -> ProgramResult {
        let my_account = &mut ctx.accounts.my_account;
        my_account.data = data;
        Ok(())
    }
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    pub my_account: ProgramAccount<'info, TakerContractAccount>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub my_account: ProgramAccount<'info, TakerContractAccount>,
}

#[account]
pub struct TakerContractAccount {
    pub data: u64,
}
