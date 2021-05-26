use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::Keypair;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signer, system_program, sysvar,
    transaction::Transaction,
};
use structopt::StructOpt;
use taker::NFTPool;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    pool_owner_keypair: Keypair,

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
    println!("program_id: {}", program_id);

    let client = Client::new(Cluster::Devnet, opt.pool_owner_keypair.clone().0);
    let program = client.program(program_id);

    let pool = NFTPool::get_address(&program.id());

    let tx = program
        .request()
        .accounts(taker::accounts::AccountsInitialize {
            pool,
            pool_owner: opt.pool_owner_keypair.pubkey(),

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
            system_program: system_program::id(),
            rent: sysvar::rent::id(),
        })
        .args(taker::instruction::Initialize {})
        .signer(&*opt.pool_owner_keypair)
        .send()?;

    println!("The transaction is {}", tx);
    println!("Pool address: {}", pool);

    let rpc = program.rpc();
    let (h, _, _) = rpc
        .get_recent_blockhash_with_commitment(CommitmentConfig::finalized())?
        .value;
    rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[
            spl_token::instruction::transfer(
                &spl_token::id(),
                &spl_associated_token_account::get_associated_token_address(
                    &opt.pool_owner_keypair.pubkey(),
                    &opt.tai_mint_address,
                ),
                &spl_associated_token_account::get_associated_token_address(
                    &pool,
                    &opt.tai_mint_address,
                ),
                &opt.pool_owner_keypair.pubkey(),
                &[&opt.pool_owner_keypair.pubkey()],
                1000 * 10u64.pow(9),
            )?,
            spl_token::instruction::transfer(
                &spl_token::id(),
                &spl_associated_token_account::get_associated_token_address(
                    &opt.pool_owner_keypair.pubkey(),
                    &opt.tkr_mint_address,
                ),
                &spl_associated_token_account::get_associated_token_address(
                    &pool,
                    &opt.tkr_mint_address,
                ),
                &opt.pool_owner_keypair.pubkey(),
                &[&opt.pool_owner_keypair.pubkey()],
                1000 * 10u64.pow(9),
            )?,
        ],
        Some(&opt.pool_owner_keypair.pubkey()),
        &[&opt.pool_owner_keypair.0],
        h,
    ))?;

    Ok(())
}
