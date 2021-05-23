#![allow(unused_imports, unused_variables, dead_code)]

use anchor_client::Client;
use anchor_client::Cluster;
use anyhow::Result;
use cli::Keypair;
use rand::rngs::OsRng;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::AccountMeta, signature::Signer};
use solana_sdk::{instruction::Instruction, sysvar};
use solana_sdk::{pubkey::Pubkey, system_program};
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
    let opt = Opt::from_args();

    let authority = &opt.taker_authority_keypair;
    let client = Client::new(Cluster::Devnet, opt.taker_authority_keypair.clone().0);
    let program = client.program(opt.taker_program_address);

    let taker_owner_keypair = &opt.taker_authority_keypair.0;
    // let nft_holder_keypair = &opt.nft_holder_keypair.0;

    let seed = solana_sdk::signature::Keypair::generate(&mut OsRng);

    let (contract_acc, _) =
        Pubkey::find_program_address(&[&authority.pubkey().to_bytes()[..]], &program.id());

    // let tx = program
    //     .request()
    //     .accounts(taker::accounts::AccountsInitialize {
    //         contract_account: contract_acc,
    //         authority: authority.pubkey(),
    //         tkr_mint: opt.tkr_mint_address,
    //         tkr_token: spl_associated_token_account::get_associated_token_address(
    //             &contract_acc,
    //             &opt.tkr_mint_address,
    //         ),
    //         tai_mint: opt.tai_mint_address,
    //         tai_token: spl_associated_token_account::get_associated_token_address(
    //             &contract_acc,
    //             &opt.tai_mint_address,
    //         ),
    //         dai_mint: opt.dai_mint_address,
    //         dai_token: spl_associated_token_account::get_associated_token_address(
    //             &contract_acc,
    //             &opt.dai_mint_address,
    //         ),
    //         spl_program: spl_token::id(),
    //         rent: sysvar::id(),
    //         system: system_program::id(),
    //     })
    //     .args(taker::instruction::Initialize {
    //         seed: seed.pubkey().to_bytes(),
    //     })
    //     .signer(&**authority)
    //     .send()?;

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsItWorks {
            contract_account: contract_acc,
            rent: sysvar::id(),
        })
        .send()?;
    println!("The transaction is {}", tx);
    Ok(())
}
