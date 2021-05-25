use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::{load_program_from_idl, Keypair};
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program, sysvar};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::{NFTBid, NFTPool};

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    lender_wallet_keypair: Keypair,

    #[structopt(long, env)]
    dai_mint_address: Pubkey,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,

    #[structopt(long)]
    price: u64,

    #[structopt(long)]
    qty: u64,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(load_program_from_idl);

    let lender_wallet_keypair = &opt.lender_wallet_keypair;

    let client = Client::new(Cluster::Devnet, lender_wallet_keypair.clone().0);
    let program = client.program(program_id);

    let pool = NFTPool::get_address(&program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsPlaceBid {
            pool,
            lender_wallet_account: lender_wallet_keypair.pubkey(),

            nft_mint: opt.nft_mint_address,
            lender_dai_account: dbg!(get_associated_token_address(
                &lender_wallet_keypair.pubkey(),
                &opt.dai_mint_address
            )),

            bid_account: dbg!(NFTBid::get_address(
                &program_id,
                &opt.nft_mint_address,
                &lender_wallet_keypair.pubkey(),
            )),

            spl_program: spl_token::id(),
            system_program: system_program::id(),
            rent: sysvar::rent::id(),
        })
        .args(taker::instruction::PlaceBid {
            price: opt.price,
            qty: opt.qty,
        })
        .signer(&**lender_wallet_keypair)
        .send()?;

    println!("The transaction is {}", tx);

    Ok(())
}
