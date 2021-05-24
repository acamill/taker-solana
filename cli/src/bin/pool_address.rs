use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use structopt::StructOpt;
use taker::get_pool_address;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();

    let pool = get_pool_address(&opt.taker_program_address);

    println!("The pool address is {}", pool);

    Ok(())
}
