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
    nft_mint_address: Pubkey,

    #[structopt(long, env)]
    lender_wallet_keypair: Keypair,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let bid_account = NFTBid::get_address(
        &program_id,
        &opt.nft_mint_address,
        &opt.lender_wallet_keypair.pubkey(),
    );

    let client = Client::new(Cluster::Devnet, opt.lender_wallet_keypair.clone().0);
    let program = client.program(program_id);

    let content: NFTBid = program.account(bid_account)?;

    println!(
        "The bid address is {} with content {:?}",
        bid_account, content
    );

    Ok(())
}
