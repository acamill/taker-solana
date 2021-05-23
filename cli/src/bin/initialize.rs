use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::Keypair;
use rand::rngs::OsRng;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program, sysvar};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Pubkey,

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

    let authority = &opt.taker_authority_keypair;

    let client = Client::new(Cluster::Devnet, authority.clone().0);
    let program = client.program(opt.taker_program_address);

    let seed = solana_sdk::signature::Keypair::generate(&mut OsRng);
    let (contract, bump) =
        Pubkey::find_program_address(&[&seed.pubkey().to_bytes()[..]], &program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsInitialize {
            authority: authority.pubkey(),
            this: contract,

            tkr_mint: opt.tkr_mint_address,
            tkr_token: spl_associated_token_account::get_associated_token_address(
                &contract,
                &opt.tkr_mint_address,
            ),

            tai_mint: opt.tai_mint_address,
            tai_token: spl_associated_token_account::get_associated_token_address(
                &contract,
                &opt.tai_mint_address,
            ),

            dai_mint: opt.dai_mint_address,
            dai_token: spl_associated_token_account::get_associated_token_address(
                &contract,
                &opt.dai_mint_address,
            ),

            ata_program: spl_associated_token_account::id(),
            spl_program: spl_token::id(),
            rent: sysvar::rent::id(),
            system: system_program::id(),
        })
        .args(taker::instruction::Initialize {
            seed: seed.pubkey().to_bytes(),
            bump,
        })
        .signer(&**authority)
        .send()?;

    println!("The transaction is {}", tx);
    println!("Contract address: {}, seed: {}", contract, seed.pubkey());

    Ok(())
}
