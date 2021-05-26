use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
use taker::EventInitialized;

#[derive(Debug, StructOpt)]
#[structopt(name = "listener", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let client = Client::new(Cluster::Devnet, Keypair::generate(&mut OsRng));
    let program = client.program(program_id);

    let _h = program.on(|_, e: EventInitialized| {
        println!("{:?}", e);
    })?;

    // let _h = program.on(|_, e: EventContractInitialized| {
    //     println!("{:?}", e);
    // })?;

    sleep(Duration::from_secs(1000000));

    Ok(())
}
