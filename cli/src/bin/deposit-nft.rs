use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::{load_program_from_idl, Keypair};
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program, sysvar};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::{NFTDeposit, NFTPool};

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    borrower_wallet_keypair: Keypair,

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

    let client = Client::new(Cluster::Devnet, opt.borrower_wallet_keypair.clone().0);
    let program = client.program(program_id);

    let pool = NFTPool::get_address(&program.id());

    let deposit_id = solana_sdk::signature::Keypair::generate(&mut OsRng).pubkey();

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsDepositNFT {
            pool,
            borrower_wallet_account: opt.borrower_wallet_keypair.pubkey(),

            nft_mint: opt.nft_mint_address,
            tkr_mint: opt.tkr_mint_address,

            borrower_nft_account: dbg!(get_associated_token_address(
                &opt.borrower_wallet_keypair.pubkey(),
                &opt.nft_mint_address
            )),
            pool_nft_account: dbg!(get_associated_token_address(&pool, &opt.nft_mint_address)),

            pool_tkr_account: dbg!(get_associated_token_address(&pool, &opt.tkr_mint_address)),
            borrower_tkr_account: dbg!(get_associated_token_address(
                &opt.borrower_wallet_keypair.pubkey(),
                &opt.tkr_mint_address
            )),

            deposit_account: dbg!(NFTDeposit::get_address(
                &program_id,
                &opt.nft_mint_address,
                &opt.borrower_wallet_keypair.pubkey(),
                &deposit_id
            )),

            ata_program: spl_associated_token_account::id(),
            spl_program: spl_token::id(),
            rent: sysvar::rent::id(),
            system_program: system_program::id(),
        })
        .args(taker::instruction::DepositNft { deposit_id })
        .signer(&*opt.borrower_wallet_keypair)
        .send()?;

    println!("The transaction is {}", tx);
    println!("The deposit_id is {}", deposit_id);

    Ok(())
}
