use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
use taker::EventContractAllocated;

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

    let _h = program.on(|_, e: EventContractAllocated| {
        println!("{:?}", e);
    })?;

    // let _h = program.on(|_, e: EventContractInitialized| {
    //     println!("{:?}", e);
    // })?;

    sleep(Duration::from_secs(1000000));

    Ok(())
}
