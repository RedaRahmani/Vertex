const anchor = require('@coral-xyz/anchor');
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID } = require('@solana/spl-token');

async function main() {
    // Initialize anchor provider
    const connection = new anchor.web3.Connection('http://127.0.0.1:8899');
    const wallet = anchor.Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {});
    anchor.setProvider(provider);

    // Program and pool parameters
    const programId = new PublicKey('B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc');
    const quoteMint = new PublicKey('ErHVBZy2oSDuDyu7accvoRVecgGNvmhXcsLZgNL1zKMu');
    const cpPool = new PublicKey('3yRXGSx8PA92uoUzdzMaH88YtfUuLWT3djTRx5mogdbu');

    // Load the IDL
    const idl = require('../target/idl/keystone_fee_router.json');
    const program = new anchor.Program(idl, programId, provider);

    // Derive PDAs
    const [policy] = await PublicKey.findProgramAddress(
        [Buffer.from('policy'), cpPool.toBuffer()],
        programId
    );

    const [vaultAuthority] = await PublicKey.findProgramAddress(
        [Buffer.from('vault'), policy.toBuffer()],
        programId
    );

    // Get or create associated token accounts
    const creatorAta = await provider.connection.getTokenAccountsByOwner(
        provider.wallet.publicKey,
        { mint: quoteMint }
    );
    
    const treasuryAta = await provider.connection.getTokenAccountsByOwner(
        vaultAuthority,
        { mint: quoteMint }
    );

    console.log('Using accounts:');
    console.log('Program ID:', programId.toString());
    console.log('Policy:', policy.toString());
    console.log('CP Pool:', cpPool.toString());
    console.log('Quote Mint:', quoteMint.toString());
    console.log('Creator ATA:', creatorAta.value[0]?.pubkey.toString());
    console.log('Treasury ATA:', treasuryAta.value[0]?.pubkey.toString());
    console.log('Vault Authority:', vaultAuthority.toString());

    try {
        const tx = await program.methods
            .initPolicy({
                y0Total: new anchor.BN(1000000000),
                investorFeeShareBps: 5000,
                dailyCapQuote: new anchor.BN(100000000),
                minPayoutLamports: new anchor.BN(1000),
            })
            .accounts({
                authority: provider.wallet.publicKey,
                policy,
                cpPool,
                quoteMint,
                creatorQuoteAta: creatorAta.value[0].pubkey,
                treasuryQuoteAta: treasuryAta.value[0].pubkey,
                vaultAuthority,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
            })
            .rpc();

        console.log('Policy initialized successfully!');
        console.log('Transaction signature:', tx);
    } catch (err) {
        console.error('Error initializing policy:', err);
    }
}

main();