use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use structopt::StructOpt;
use taker::NFTPool;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let pool_account = NFTPool::get_address(&program_id);

    let client = Client::new(Cluster::Devnet, Keypair::generate(&mut OsRng));
    let program = client.program(program_id);

    let content: NFTPool = program.account(pool_account)?;

    println!(
        "The pool address is {} with content {:?}",
        pool_account, content
    );

    Ok(())
}
