use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::AccountMeta, signature::Signer};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::instruction::TakerInstruction;

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, default_value = "https://devnet.solana.com")]
    solana_rpc_url: String,

    #[structopt(long, env, default_value = "wss://devnet.solana.com")]
    solana_sub_url: String,

    #[structopt(long, env)]
    taker_owner_keypair: String,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,

    #[structopt(long, env)]
    nft_holder_keypair: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let taker_owner_keypair = Keypair::from_base58_string(&opt.taker_owner_keypair);
    let nft_holder_keypair = Keypair::from_base58_string(&opt.nft_holder_keypair);

    let client = RpcClient::new(opt.solana_rpc_url.clone());
    let balance = client.get_balance(&taker_owner_keypair.pubkey())?;
    println!("Taker Owner Balance: {}", balance);

    // Call Taker Program
    let taker_program_id = taker::id();
    println!("Calling {}", taker_program_id);

    let ins = Instruction::new_with_bytes(
        taker_program_id.clone(),
        &TakerInstruction::DepositNFT {
            token_id: opt.nft_mint_address,
        }
        .pack(),
        vec![
            AccountMeta::new(nft_holder_keypair.pubkey(), true),
            AccountMeta::new(
                get_associated_token_address(&nft_holder_keypair.pubkey(), &opt.nft_mint_address),
                false,
            ),
            AccountMeta::new(
                get_associated_token_address(&taker_owner_keypair.pubkey(), &opt.nft_mint_address),
                false,
            ),
            AccountMeta::new(spl_token::id(), false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &vec![ins],
        Some(&nft_holder_keypair.pubkey()),
        &[&nft_holder_keypair],
        client.get_recent_blockhash()?.0,
    );

    let resp = client.send_transaction(&tx)?;
    println!("Call Signature {:?}", resp);
    Ok(())
}
