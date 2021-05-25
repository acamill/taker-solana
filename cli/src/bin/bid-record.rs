use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::Keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use structopt::StructOpt;
use taker::NFTBid;

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

    let bid_account =
        NFTBid::get_address(&program_id, &opt.nft_mint_address, &opt.taker_user.pubkey());

    let client = Client::new(Cluster::Devnet, opt.taker_user.clone().0);
    let program = client.program(program_id);

    let content: taker::NFTBid = program.account(bid_account)?;

    println!(
        "The bid address is {} with bid {} @ {}",
        bid_account, content.qty, content.price
    );

    Ok(())
}
