use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::Keypair;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program, sysvar};
use structopt::StructOpt;
use taker::get_pool_address;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    taker_authority_keypair: Keypair,

    #[structopt(long, env)]
    tkr_mint_address: Pubkey,

    #[structopt(long, env)]
    tai_mint_address: Pubkey,

    #[structopt(long, env)]
    dai_mint_address: Pubkey,
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");

    let opt = Opt::from_args();
    let program_id = opt
        .taker_program_address
        .unwrap_or_else(cli::load_program_from_idl);

    let authority = &opt.taker_authority_keypair;

    let client = Client::new(Cluster::Devnet, authority.clone().0);
    let program = client.program(program_id);

    let pool = get_pool_address(&program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsInitialize {
            pool,
            pool_owner: authority.pubkey(),

            tkr_mint: opt.tkr_mint_address,
            pool_tkr_account: spl_associated_token_account::get_associated_token_address(
                &pool,
                &opt.tkr_mint_address,
            ),

            tai_mint: opt.tai_mint_address,
            pool_tai_account: spl_associated_token_account::get_associated_token_address(
                &pool,
                &opt.tai_mint_address,
            ),

            dai_mint: opt.dai_mint_address,
            pool_dai_account: spl_associated_token_account::get_associated_token_address(
                &pool,
                &opt.dai_mint_address,
            ),

            ata_program: spl_associated_token_account::id(),
            spl_program: spl_token::id(),
            rent: sysvar::rent::id(),
            system: system_program::id(),
        })
        .args(taker::instruction::Initialize {})
        .signer(&**authority)
        .send()?;

    println!("The transaction is {}", tx);
    println!("Pool address: {}", pool);

    Ok(())
}