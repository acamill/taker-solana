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
    #[structopt(long, env)]
    taker_authority_keypair: Keypair,

    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,

    #[structopt(long, env)]
    nft_holder_keypair: Keypair,

    #[structopt(long, env)]
    tkr_mint_address: Pubkey,
    #[structopt(long, env)]
    tai_mint_address: Pubkey,
    #[structopt(long, env)]
    dai_mint_address: Pubkey,
}

fn main() -> Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    let authority = &opt.taker_authority_keypair;
    let client = Client::new(Cluster::Devnet, opt.taker_authority_keypair.clone().0);
    let program = client.program(opt.taker_program_address);

    let taker_owner_keypair = &opt.taker_authority_keypair.0;
    // let nft_holder_keypair = &opt.nft_holder_keypair.0;

    let seed = solana_sdk::signature::Keypair::generate(&mut OsRng);

    let (contract, _) =
        Pubkey::find_program_address(&[&seed.pubkey().to_bytes()[..]], &program.id());
    println!("authkey: {}, contract: {}", authority.pubkey(), contract);

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsAllocate {
            contract,
            authority: authority.pubkey(),
            rent: sysvar::rent::id(),
            system: system_program::id(),
        })
        .args(taker::instruction::Allocate {
            seed: seed.pubkey().to_bytes(),
        })
        .signer(&**authority)
        .send()?;

    // let tx = program
    //     .request()
    //     .accounts(taker::accounts::AccountsInitialize {
    //         contract_account: contract_acc,
    //         authority: authority.pubkey(),
    //         // tkr_mint: opt.tkr_mint_address,
    //         // tkr_token: spl_associated_token_account::get_associated_token_address(
    //         //     &contract_acc,
    //         //     &opt.tkr_mint_address,
    //         // ),
    //         // tai_mint: opt.tai_mint_address,
    //         // tai_token: spl_associated_token_account::get_associated_token_address(
    //         //     &contract_acc,
    //         //     &opt.tai_mint_address,
    //         // ),
    //         // dai_mint: opt.dai_mint_address,
    //         // dai_token: spl_associated_token_account::get_associated_token_address(
    //         //     &contract_acc,
    //         //     &opt.dai_mint_address,
    //         // ),
    //         // spl_program: spl_token::id(),
    //         rent: sysvar::rent::id(),
    //         // system: system_program::id(),
    //     })
    //     .args(taker::instruction::Initialize {
    //         seed: seed.pubkey().to_bytes(),
    //     })
    //     .instruction(system_instruction::create_account(
    //         &authority.pubkey(),
    //         &contract_acc,
    //         required_lamports,
    //         acc_size as u64,
    //         &opt.taker_program_address,
    //     ))
    //     .signer(&**authority)
    //     .send()?;

    println!("The transaction is {}", tx);
    Ok(())
}
