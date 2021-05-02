#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

//! An ERC20-like Token program for the Solana blockchain

pub mod error;
pub mod instruction;
// pub mod native_mint;
pub mod processor;
// pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("HJLnBLx5azZjTf5aVfjn2oBMZ5fbD9QZnSTbN62wQDJ2");
