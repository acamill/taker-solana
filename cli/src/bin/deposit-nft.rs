use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::{load_program_from_idl, Keypair};
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program, sysvar};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::get_nft_listing_address;
use taker::get_pool_address;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    taker_user: Keypair,

    #[structopt(long, env)]
    tkr_mint_address: Pubkey,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(load_program_from_idl);

    let taker_user = &opt.taker_user;

    let client = Client::new(Cluster::Devnet, taker_user.clone().0);
    let program = client.program(program_id);

    let pool = get_pool_address(&program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsDepositNFT {
            pool,
            user_wallet_account: taker_user.pubkey(),

            nft_mint: opt.nft_mint_address,
            user_nft_account: dbg!(get_associated_token_address(
                &taker_user.pubkey(),
                &opt.nft_mint_address
            )),
            pool_nft_account: dbg!(get_associated_token_address(&pool, &opt.nft_mint_address)),

            tkr_mint: opt.tkr_mint_address,
            pool_tkr_account: dbg!(get_associated_token_address(&pool, &opt.tkr_mint_address)),
            user_tkr_account: dbg!(get_associated_token_address(
                &taker_user.pubkey(),
                &opt.tkr_mint_address
            )),

            listing_account: dbg!(get_nft_listing_address(
                &program_id,
                &opt.nft_mint_address,
                &taker_user.pubkey(),
            )),

            ata_program: spl_associated_token_account::id(),
            spl_program: spl_token::id(),
            rent: sysvar::rent::id(),
            system: system_program::id(),
        })
        .args(taker::instruction::DepositNft {})
        .signer(&**taker_user)
        .send()?;

    println!("The transaction is {}", tx);

    Ok(())
}
