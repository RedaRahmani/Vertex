import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, web3, BN } from '@coral-xyz/anchor';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';
import { MeteoraDlmmPoolClient } from '../sdk/ts/src/types/meteora';

async function main() {
    // Connect to local validator
    const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
    const wallet = web3.Keypair.generate();
    
    // Request airdrop
    const airdropSig = await connection.requestAirdrop(
        wallet.publicKey,
        web3.LAMPORTS_PER_SOL * 2
    );
    await connection.confirmTransaction(airdropSig);

    const provider = new AnchorProvider(
        connection,
        { publicKey: wallet.publicKey, signTransaction: wallet.sign },
        {}
    );

    // Create Meteora DLMM Pool
    const dlmmPool = new MeteoraDlmmPoolClient({
        connection,
        wallet: provider.wallet,
        programId: new PublicKey('QnCi53pPuxT1ud9wysgFBRZk3yewF4vKhmDjfuvyqL3'),
    });

    // Create two test tokens
    const token0Mint = await createMint(connection, wallet);
    const token1Mint = await createMint(connection, wallet);

    // Create pool
    const poolKeypair = web3.Keypair.generate();
    const { tx } = await dlmmPool.createPool({
        poolKeypair,
        token0Mint,
        token1Mint,
        admin: wallet.publicKey,
        poolType: 'standard',
    });

    await provider.sendAndConfirm(tx);
    
    console.log('Pool created:', poolKeypair.publicKey.toString());
    console.log('Token0 Mint:', token0Mint.toString());
    console.log('Token1 Mint:', token1Mint.toString());
}

async function createMint(
    connection: Connection,
    payer: Keypair
): Promise<PublicKey> {
    const mint = web3.Keypair.generate();
    const rentLamports = await connection.getMinimumBalanceForRentExemption(82);

    const createAccountIx = web3.SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mint.publicKey,
        space: 82,
        lamports: rentLamports,
        programId: TOKEN_PROGRAM_ID,
    });

    const initMintIx = await createInitializeMintInstruction(
        mint.publicKey,
        6,
        payer.publicKey,
        payer.publicKey
    );

    const tx = new web3.Transaction().add(createAccountIx, initMintIx);
    await web3.sendAndConfirmTransaction(connection, tx, [payer, mint]);

    return mint.publicKey;
}

main();