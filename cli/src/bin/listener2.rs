use anchor_client::Client;
use anchor_client::Cluster;
use anyhow::Result;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::AccountMeta, signature::Signer};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use structopt::StructOpt;
use taker::instruction::TakerInstruction;

pub struct Keypair(pub solana_sdk::signature::Keypair);

impl FromStr for Keypair {
    fn from_str(s: &str) -> Self {
        solana_sdk::signature::Keypair::from_base58_string(s);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "listener", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env)]
    taker_owner_keypair: Keypair,

    #[structopt(long, env)]
    taker_program_address: Pubkey,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let taker_owner_keypair = &opt.taker_owner_keypair.0;
    let nft_holder_keypair = &opt.nft_holder_keypair.0;

    let client = Client::new(Cluster::Devnet, *taker_owner_keypair);
    let program = client.program(opt.taker_program_address);

    program.on(|| {});

    Ok(())
}
