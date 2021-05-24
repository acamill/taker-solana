use anchor_client::{Client, Cluster};
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
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    taker_user: Keypair,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let listing =
        get_nft_listing_address(&program_id, &opt.nft_mint_address, &opt.taker_user.pubkey());

    let client = Client::new(Cluster::Devnet, opt.taker_user.clone().0);
    let program = client.program(program_id);

    let content: taker::NFTListing = program.account(listing)?;

    println!(
        "The listing address is {} with {} listed",
        listing, content.count
    );

    Ok(())
}
