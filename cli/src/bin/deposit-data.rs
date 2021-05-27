use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use structopt::StructOpt;
use taker::NFTDeposit;

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
    deposit_id: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let deposit_account = NFTDeposit::get_address(
        &program_id,
        &opt.nft_mint_address,
        &opt.borrower_wallet_address,
        &opt.deposit_id,
    );

    let client = Client::new(Cluster::Devnet, Keypair::generate(&mut OsRng));
    let program = client.program(program_id);

    let content: NFTDeposit = program.account(deposit_account)?;

    println!("Account: {:?}, Deposit: {:?}", deposit_account, content);

    Ok(())
}
