// client.js is used to introduce the reader to generating clients from IDLs.
// It is not expected users directly test with this example. For a more
// ergonomic example, see `tests/basic-0.js` in this workspace.

const anchor = require('@project-serum/anchor');
const solanaWeb3 = require('@solana/web3.js');
const splToken = require('@solana/spl-token');
const bs58 = require('bs58');


// Configure the client to use the local cluster.
anchor.setProvider(anchor.Provider.env());

// const idl = JSON.parse(require('fs').readFileSync('./target/idl/taker.json', 'utf8'));
// const programId = new anchor.web3.PublicKey('91aE2UGTmGfy9FVCPB9PFoNbEokDoPBKh8nitW4QPwxp');
// const program = new anchor.Program(idl, programId);

const program = anchor.workspace.Taker;

const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new anchor.web3.PublicKey(
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);
async function findAssociatedTokenAddress(
    walletAddress,
    tokenMintAddress
) {
    return (await solanaWeb3.PublicKey.findProgramAddress(
        [
            walletAddress.toBuffer(),
            splToken.TOKEN_PROGRAM_ID.toBuffer(),
            tokenMintAddress.toBuffer(),
        ],
        SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    ))[0];
}

async function main() {
    program.addEventListener("CalledInitialize", (event, slot) => { console.log("Event:", event) });

    let authority = anchor.web3.Keypair.fromSecretKey(
        bs58.decode(process.env.TAKER_OWNER_KEYPAIR)
    );

    const seed = await anchor.web3.Keypair.generate();

    const [contract_acc, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
        [authority.publicKey.toBuffer()],
        program.programId
    );
    const tkr_mint = new anchor.web3.PublicKey(process.env.TKR_MINT_ADDRESS);
    const tai_mint = new anchor.web3.PublicKey(process.env.TAI_MINT_ADDRESS);
    const dai_mint = new anchor.web3.PublicKey(process.env.DAI_MINT_ADDRESS);


    const tx = await program.rpc.initialize(
        seed.publicKey.toBuffer(),
        {
            accounts: {
                contractAccount: contract_acc,
                authority: authority.publicKey,
                tkrMint: tkr_mint,
                tkrToken: await findAssociatedTokenAddress(contract_acc, tkr_mint),
                taiMint: tai_mint,
                taiToken: await findAssociatedTokenAddress(contract_acc, tai_mint),
                daiMint: dai_mint,
                daiToken: await findAssociatedTokenAddress(contract_acc, dai_mint),
                splProgram: splToken.TOKEN_PROGRAM_ID,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
            signers: [authority],
        });

    console.log("Your transaction signature", tx);
}

console.log('Running client.');
main().then(() => console.log('Success'));

