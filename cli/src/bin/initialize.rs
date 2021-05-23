#![allow(unused_imports, unused_variables, dead_code)]

use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::Keypair;
use rand::rngs::OsRng;
use solana_sdk::{
    instruction::AccountMeta, instruction::Instruction, pubkey::Pubkey, signature::Signer,
    system_instruction, system_program, sysvar, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use structopt::StructOpt;
use taker::AccountsInitialize;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,

    #[structopt(long, env)]
    taker_authority_keypair: Keypair,

    #[structopt(long, env, short = "c")]
    taker_contract_seed: Pubkey,

    #[structopt(long, env)]
    tkr_mint_address: Pubkey,

    #[structopt(long, env)]
    tai_mint_address: Pubkey,

    #[structopt(long, env)]
    dai_mint_address: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();

    let authority = &opt.taker_authority_keypair;

    let client = Client::new(Cluster::Devnet, authority.clone().0);
    let program = client.program(opt.taker_program_address);

    let taker_owner_keypair = &opt.taker_authority_keypair.0;

    let seed = opt.taker_contract_seed;
    let (contract, bump) = Pubkey::find_program_address(&[&seed.to_bytes()[..]], &program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsInitialize {
            authority: authority.pubkey(),
            contract,

            tkr_mint: opt.tkr_mint_address,
            tkr_token: spl_associated_token_account::get_associated_token_address(
                &contract,
                &opt.tkr_mint_address,
            ),

            tai_mint: opt.tai_mint_address,
            tai_token: spl_associated_token_account::get_associated_token_address(
                &contract,
                &opt.tai_mint_address,
            ),

            dai_mint: opt.dai_mint_address,
            dai_token: spl_associated_token_account::get_associated_token_address(
                &contract,
                &opt.dai_mint_address,
            ),

            ata_program: spl_associated_token_account::id(),
            spl_program: spl_token::id(),
            rent: sysvar::rent::id(),
            system: system_program::id(),
        })
        .args(taker::instruction::Initialize {
            seed: seed.to_bytes(),
            bump,
        })
        .signer(&**authority)
        .send()?;

    println!("The transaction is {}", tx);

    Ok(())
}
