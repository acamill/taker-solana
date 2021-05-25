use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::{load_program_from_idl, Keypair};
use solana_sdk::{pubkey::Pubkey, signature::Signer, sysvar};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::{NFTListing, NFTLoan, NFTPool};

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    borrower_wallet_keypair: Keypair,

    #[structopt(long, env)]
    lender_wallet_address: Pubkey,

    #[structopt(long, env)]
    dai_mint_address: Pubkey,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,

    #[structopt(long, env)]
    loan_id: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(load_program_from_idl);

    let client = Client::new(Cluster::Devnet, opt.borrower_wallet_keypair.clone().0);
    let program = client.program(program_id);

    let pool = NFTPool::get_address(&program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsRepay {
            pool,
            borrower_wallet_account: opt.borrower_wallet_keypair.pubkey(),

            borrower_dai_account: dbg!(get_associated_token_address(
                &opt.borrower_wallet_keypair.pubkey(),
                &opt.dai_mint_address
            )),
            lender_dai_account: dbg!(get_associated_token_address(
                &opt.lender_wallet_address,
                &opt.dai_mint_address
            )),
            pool_dai_account: dbg!(get_associated_token_address(&pool, &opt.dai_mint_address)),

            loan_account: dbg!(NFTLoan::get_address(
                &program_id,
                &opt.nft_mint_address,
                &opt.borrower_wallet_keypair.pubkey(),
                &opt.lender_wallet_address,
                &opt.loan_id,
            )),
            listing_account: dbg!(NFTListing::get_address(
                &program_id,
                &opt.nft_mint_address,
                &opt.borrower_wallet_keypair.pubkey(),
            )),

            spl_program: spl_token::id(),
            clock: sysvar::clock::id(),
        })
        .args(taker::instruction::Repay {})
        .signer(&opt.borrower_wallet_keypair.clone().0)
        .send()?;

    println!("The transaction is {}", tx);

    Ok(())
}
