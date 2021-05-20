#![forbid(unsafe_code)]
#![allow(unused_imports, unused_variables, dead_code)]

pub mod error;
pub mod instruction;
mod spl_helper;
mod state;
// pub mod native_mint;
pub mod processor;
// pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("HJLnBLx5azZjTf5aVfjn2oBMZ5fbD9QZnSTbN62wQDJ2");

/// Create an associated token account for the given wallet address and token mint
///
/// Accounts expected by this instruction:
///
///   0. `[writeable,signer]` Funding account (must be a system account)
///   1. `[writeable]` authority of the Contract Account
///   4. `[]` System program
///   5. `[]` SPL Token program
///   6. `[]` Rent sysvar
///
pub fn create_contract_account(funding_address: &Pubkey, wallet_address: &Pubkey) -> Instruction {
    let associated_account_address =
        get_associated_token_address(wallet_address, spl_token_mint_address);

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, false),
            AccountMeta::new_readonly(*spl_token_mint_address, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: vec![],
    }
}
