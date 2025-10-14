import { Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, createMint } from '@solana/spl-token';

const FEE_ROUTER_PROGRAM_ID = 'B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc';
const METEORA_POOL = 'QnCi53pPuxT1ud9wysgFBRZk3yewF4vKhmDjfuvyqL3';

async function main() {
    // Connect to local validator
    const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
    const payer = Keypair.generate();
    
    // Request airdrop
    const airdropSig = await connection.requestAirdrop(
        payer.publicKey,
        2_000_000_000 // 2 SOL
    );
    await connection.confirmTransaction(airdropSig);

    // Create test token
    const quoteMint = await createMint(
        connection,
        payer,
        payer.publicKey,
        payer.publicKey,
        6
    );

    console.log('Quote mint created:', quoteMint.toString());
    
    // Setup Anchor
    const provider = new AnchorProvider(
        connection,
        new Wallet(payer),
        AnchorProvider.defaultOptions()
    );

    // Create program instance
    const program = new Program(
        require('../target/idl/keystone_fee_router.json'),
        new PublicKey(FEE_ROUTER_PROGRAM_ID),
        provider
    );

    // Get PDAs
    const [policy] = PublicKey.findProgramAddressSync(
        [Buffer.from('policy'), new PublicKey(METEORA_POOL).toBuffer()],
        new PublicKey(FEE_ROUTER_PROGRAM_ID)
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
        [Buffer.from('vault'), policy.toBuffer()],
        new PublicKey(FEE_ROUTER_PROGRAM_ID)
    );

    try {
        const tx = await program.methods
            .initPolicy({
                y0Total: new BN(1_000_000),
                investorFeeShareBps: 5000,
                dailyCapQuote: new BN(1_000_000),
                minPayoutLamports: new BN(1_000),
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