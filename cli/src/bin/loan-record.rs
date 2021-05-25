use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use structopt::StructOpt;
use taker::NFTLoan;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,

    #[structopt(long, env)]
    borrower_wallet_address: Pubkey,

    #[structopt(long, env)]
    lender_wallet_address: Pubkey,

    #[structopt(long, env)]
    loan_id: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let loan_account = NFTLoan::get_address(
        &program_id,
        &opt.nft_mint_address,
        &opt.borrower_wallet_address,
        &opt.lender_wallet_address,
        &opt.loan_id,
    );

    let client = Client::new(Cluster::Devnet, Keypair::generate(&mut OsRng));
    let program = client.program(program_id);

    let content: taker::NFTLoan = program.account(loan_account)?;

    println!("Loan: {:?}", content);

    Ok(())
}
