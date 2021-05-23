use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
use taker::{EventCalledInitialize, EventContractCreated};

#[derive(Debug, StructOpt)]
#[structopt(name = "listener", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let acc = Keypair::generate(&mut OsRng);

    let client = Client::new(Cluster::Devnet, acc);
    let program = client.program(opt.taker_program_address);

    let _handle = program.on(|_, e: EventContractCreated| {
        println!("{:?}", e);
    })?;

    sleep(Duration::from_secs(1000000));

    Ok(())
}
