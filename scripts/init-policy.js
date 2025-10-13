const anchor = require('@coral-xyz/anchor');
const { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID, createMint } = require('@solana/spl-token');

const FEE_ROUTER_PROGRAM_ID = 'B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc';
const METEORA_POOL = 'QnCi53pPuxT1ud9wysgFBRZk3yewF4vKhmDjfuvyqL3';

async function main() {
    // Setup provider
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    // Get the IDL
    const idl = require('../target/idl/keystone_fee_router.json');
    
    // Create program
    const program = new anchor.Program(idl, FEE_ROUTER_PROGRAM_ID, provider);

    // Create test token
    const payer = Keypair.generate();
    
    // Request airdrop
    const airdropSig = await provider.connection.requestAirdrop(
        payer.publicKey,
        2_000_000_000 // 2 SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
    
    const quoteMint = await createMint(
        provider.connection,
        payer,
        payer.publicKey,
        payer.publicKey,
        6
    );

    console.log('Quote mint created:', quoteMint.toString());

    // Get PDAs
    const [policy] = PublicKey.findProgramAddressSync(
        [Buffer.from('policy'), new PublicKey(METEORA_POOL).toBuffer()],
        program.programId
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
        [Buffer.from('vault'), policy.toBuffer()],
        program.programId
    );

    try {
        const tx = await program.methods
            .initPolicy({
                y0Total: new anchor.BN(1_000_000),
                investorFeeShareBps: 5000,
                dailyCapQuote: new anchor.BN(1_000_000),
                minPayoutLamports: new anchor.BN(1_000),
            })
            .accounts({
                authority: provider.wallet.publicKey,
                policy,
                cpPool: new PublicKey(METEORA_POOL),
                quoteMint,
                creatorQuoteAta: provider.wallet.publicKey,
                treasuryQuoteAta: provider.wallet.publicKey,
                vaultAuthority,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
            })
            .signers([payer])
            .rpc();

        console.log('Policy initialized!');
        console.log('Transaction:', tx);
        console.log('Policy address:', policy.toString());
        console.log('Vault authority:', vaultAuthority.toString());
    } catch (err) {
        console.error('Error:', err);
    }
}

main();