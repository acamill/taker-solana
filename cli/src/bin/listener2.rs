use anchor_client::Client;
use anchor_client::Cluster;
use anyhow::Result;
use cli::Keypair;
use solana_sdk::pubkey::Pubkey;
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
use taker::EventItWorks;

#[derive(Debug, StructOpt)]
#[structopt(name = "listener", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env)]
    taker_authority_keypair: Keypair,

    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let client = Client::new(Cluster::Devnet, opt.taker_authority_keypair.0);
    let program = client.program(opt.taker_program_address);

    let _handle = program.on(|_, e: EventItWorks| {
        println!("{:?}", e);
    })?;

    sleep(Duration::from_secs(1000000));

    Ok(())
}
