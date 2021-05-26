use anchor_client::{Client, ClientError as ClientError0, Cluster};
use anyhow::Result;
use cli::{load_program_from_idl, Keypair};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::{RpcError, RpcResponseErrorData},
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};
use solana_sdk::{pubkey::Pubkey, signature::Signer};
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;
use taker::{NFTDeposit, NFTPool, TakerError};
#[derive(Debug, StructOpt)]
#[structopt(name = "transact", about = "Making transactions to the Taker Protocol")]
struct Opt {
    #[structopt(long, env, short = "p")]
    taker_program_address: Option<Pubkey>,

    #[structopt(long, env)]
    borrower_wallet_keypair: Keypair,

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

    let borrower_wallet_keypair = &opt.borrower_wallet_keypair;

    let client = Client::new(Cluster::Devnet, borrower_wallet_keypair.clone().0);
    let program = client.program(program_id);

    let pool = NFTPool::get_address(&program.id());

    let resp = program
        .request()
        .accounts(taker::accounts::AccountsWithdrawNFT {
            pool,
            borrower_wallet_account: borrower_wallet_keypair.pubkey(),

            nft_mint: opt.nft_mint_address,
            borrower_nft_account: dbg!(get_associated_token_address(
                &borrower_wallet_keypair.pubkey(),
                &opt.nft_mint_address
            )),
            pool_nft_account: dbg!(get_associated_token_address(&pool, &opt.nft_mint_address)),

            deposit_account: dbg!(NFTDeposit::get_address(
                &program_id,
                &opt.nft_mint_address,
                &borrower_wallet_keypair.pubkey(),
                &opt.deposit_id
            )),

            spl_program: spl_token::id(),
        })
        .args(taker::instruction::WithdrawNft {
            deposit_id: opt.deposit_id,
        })
        .signer(&**borrower_wallet_keypair)
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
