use anchor_client::ClientError as ClientError0;
use anchor_client::{Client, Cluster};
use anyhow::Result;
use cli::{load_program_from_idl, Keypair};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::{RpcError, RpcResponseErrorData},
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};
use solana_sdk::{pubkey::Pubkey, signature::Signer, sysvar};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::TakerError;
use taker::{NFTBid, NFTDeposit, NFTPool};

#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    borrower_wallet_keypair: Keypair,

    #[structopt(long, env)]
    lender_wallet_address: Pubkey,

    #[structopt(long, env)]
    tai_mint_address: Pubkey,

    #[structopt(long, env)]
    dai_mint_address: Pubkey,

    #[structopt(long, env)]
    nft_mint_address: Pubkey,

    #[structopt(long, env)]
    deposit_id: Pubkey,
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

    let resp = program
        .request()
        .accounts(taker::accounts::AccountsBorrow {
            pool,
            borrower_wallet_account: opt.borrower_wallet_keypair.pubkey(),
            lender_wallet_account: opt.lender_wallet_address,

            nft_mint: opt.nft_mint_address,
            borrower_dai_account: dbg!(get_associated_token_address(
                &opt.borrower_wallet_keypair.pubkey(),
                &opt.dai_mint_address
            )),
            lender_dai_account: dbg!(get_associated_token_address(
                &opt.lender_wallet_address,
                &opt.dai_mint_address
            )),
            pool_dai_account: dbg!(get_associated_token_address(&pool, &opt.dai_mint_address)),

            lender_tai_account: dbg!(get_associated_token_address(
                &opt.lender_wallet_address,
                &opt.tai_mint_address
            )),
            pool_tai_account: dbg!(get_associated_token_address(&pool, &opt.tai_mint_address)),

            deposit_account: dbg!(NFTDeposit::get_address(
                &program_id,
                &opt.nft_mint_address,
                &opt.borrower_wallet_keypair.pubkey(),
                &opt.deposit_id,
            )),
            bid_account: dbg!(NFTBid::get_address(
                &program_id,
                &opt.nft_mint_address,
                &opt.lender_wallet_address,
            )),

            spl_program: spl_token::id(),
            clock: sysvar::clock::id(),
        })
        .args(taker::instruction::Borrow {
            amount: 998 * 10u64.pow(9),
        })
        .signer(&opt.borrower_wallet_keypair.clone().0)
        .send();

    match resp {
        Ok(tx) => println!("The transaction is {}", tx),
        Err(ClientError0::SolanaClientError(ClientError {
            kind:
                ClientErrorKind::RpcError(RpcError::RpcResponseError {
                    data:
                        RpcResponseErrorData::SendTransactionPreflightFailure(
                            RpcSimulateTransactionResult {
                                err:
                                    Some(TransactionError::InstructionError(
                                        _,
                                        InstructionError::Custom(code),
                                    )),
                                ..
                            },
                        ),
                    ..
                }),
            ..
        })) => {
            println!("Error: {}", TakerError::from_code(code));
        }
        Err(e) => println!("{:?}", e),
    };

    Ok(())
}
