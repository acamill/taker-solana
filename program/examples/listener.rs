use anyhow::Result;
use solana_client::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
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

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let (_pc, rx) = PubsubClient::logs_subscribe(
        &opt.solana_sub_url,
        RpcTransactionLogsFilter::Mentions(vec![format!("{}", opt.tkr_program_id)]),
        RpcTransactionLogsConfig { commitment: None },
    )?;

    loop {
        let msg = rx.recv()?;
        for l in msg.value.logs {
            println!("{}", l);
        }
    }
}
