const anchor = require('@project-serum/anchor');
const bs58 = require('bs58');

describe('taker-solana', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Taker;

  it('Is initialized!', async () => {
    let authority = anchor.web3.Keypair.fromSecretKey(
      bs58.decode(process.env.TAKER_OWNER_KEYPAIR)
    );

    const seed = await anchor.web3.Keypair.generate();

    const [contract_acc, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [authority.publicKey.toBuffer()],
      program.programId
    );
    const tkr_mint = new anchor.web3.PublicKey(process.env.TKR_MINT_ADDRESS);
    const nft_mint = new anchor.web3.PublicKey(process.env.NFT_MINT_ADDRESS);
    const tai_mint = new anchor.web3.PublicKey(process.env.TAI_MINT_ADDRESS);
    const dai_mint = new anchor.web3.PublicKey(process.env.DAI_MINT_ADDRESS);

    console.log(tkr_mint);
    const tx = await program.rpc.initialize(
      seed.publicKey.toBuffer(),
      {
        accounts: {
          contractAccount: contract_acc,
          authority: authority.publicKey,
          tkrMint: tkr_mint,
          //  tkr_token: CpiAccount< 'info, TokenAccount>,
          taiMint: tai_mint,
          //  tai_token: CpiAccount< 'info, TokenAccount>,
          daiMint: dai_mint,
          //  dai_token: CpiAccount< 'info, TokenAccount>,
          //  spl_program: AccountInfo< 'info>,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        }
      });

    console.log("Your transaction signature", tx);
  });
});
