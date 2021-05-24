use anyhow::Result;
use cli::Keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use structopt::StructOpt;
use taker::get_nft_listing_address;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,

    #[structopt(long, env)]
    taker_user: Keypair,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();

    let listing = get_nft_listing_address(
        &opt.taker_program_address,
        &opt.nft_mint_address,
        &opt.taker_user.pubkey(),
    );

    println!("The listing address is {}", listing);

    Ok(())
}
