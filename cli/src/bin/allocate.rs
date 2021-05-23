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
}

fn main() -> Result<()> {
    solana_logger::setup_with("solana=debug");
    let opt = Opt::from_args();

    let authority = &opt.taker_authority_keypair;
    let client = Client::new(Cluster::Devnet, opt.taker_authority_keypair.clone().0);
    let program = client.program(opt.taker_program_address);

    let seed = solana_sdk::signature::Keypair::generate(&mut OsRng);

    let (contract, _) =
        Pubkey::find_program_address(&[&seed.pubkey().to_bytes()[..]], &program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsAllocate {
            contract,
            authority: authority.pubkey(),
            rent: sysvar::rent::id(),
            system: system_program::id(),
        })
        .args(taker::instruction::Allocate {
            seed: seed.pubkey().to_bytes(),
        })
        .signer(&**authority)
        .send()?;

    println!("The transaction is {}", tx);
    println!("Contract address: {}", contract);

    Ok(())
}
