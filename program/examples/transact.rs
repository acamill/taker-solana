use anyhow::Result;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::AccountMeta, signature::Signer};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(long, env)]
    solana_rpc_url: String,

    #[structopt(long, env)]
    solana_sub_url: String,

    #[structopt(long, env)]
    solana_pubkey: Pubkey,

    #[structopt(long, env)]
    solana_keypair: String,

    #[structopt(long, env)]
    tkr_program_id: Pubkey,
}

#[derive(Serialize)]
pub struct Abc {
    abc: usize,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let keypair = Keypair::from_base58_string(&opt.solana_keypair);

    let c = RpcClient::new(opt.solana_rpc_url.clone());
    let b = c.get_balance(&keypair.pubkey())?;
    println!("balance {}", b);


    
    let ins = vec![Instruction::new_with_bincode(
        opt.tkr_program_id.clone(),
        &Abc { abc: 33 },
        vec![AccountMeta::new(opt.solana_pubkey.clone(), true)],
    )];

    let (h, _) = c.get_recent_blockhash()?;
    let t =
        Transaction::new_signed_with_payer(&ins, Some(&opt.solana_pubkey.clone()), &[&keypair], h);
    let resp = c.send_transaction(&t)?;
    println!("{:?}", resp);
    Ok(())
}
