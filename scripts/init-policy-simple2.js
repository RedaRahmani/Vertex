const { Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID } = require('@solana/spl-token');
const anchor = require('@coral-xyz/anchor');
const fs = require('fs');

async function main() {
    console.log('Starting initialization...');
    
    // Initialize connection
    const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
    
    // Load the keypair from the default Solana config
    const payer = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(process.env.HOME + '/.config/solana/id.json', 'utf-8')))
    );
    
    // Create anchor wallet and provider
    const wallet = new anchor.Wallet(payer);
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: 'confirmed',
        preflightCommitment: 'confirmed',
    });
    anchor.setProvider(provider);

    // Program and token parameters
    const programId = new PublicKey('B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc');
    const quoteMint = new PublicKey('ErHVBZy2oSDuDyu7accvoRVecgGNvmhXcsLZgNL1zKMu');
    const cpPool = new PublicKey('3yRXGSx8PA92uoUzdzMaH88YtfUuLWT3djTRx5mogdbu');

    console.log('Program ID:', programId.toString());
    console.log('Quote Mint:', quoteMint.toString());
    console.log('CP Pool:', cpPool.toString());
    console.log('Wallet pubkey:', wallet.publicKey.toString());

    // Derive PDAs
    const [policy] = PublicKey.findProgramAddressSync(
        [Buffer.from('policy'), cpPool.toBuffer()],
        programId
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
        [Buffer.from('vault'), policy.toBuffer()],
        programId
    );

    console.log('Policy PDA:', policy.toString());
    console.log('Vault Authority PDA:', vaultAuthority.toString());

    // Get creator's token account
    const creatorAta = (await connection.getTokenAccountsByOwner(wallet.publicKey, { mint: quoteMint })).value[0]?.pubkey;
    
    if (!creatorAta) {
        throw new Error('Missing creator token account');
    }

    // Get treasury token account
    const treasuryAta = (await connection.getTokenAccountsByOwner(vaultAuthority, { mint: quoteMint })).value[0]?.pubkey;
    
    if (!treasuryAta) {
        console.log('Treasury ATA not found - will be created during initialization');
    }

    console.log('Creator ATA:', creatorAta.toString());
    if (treasuryAta) {
        console.log('Treasury ATA:', treasuryAta.toString());
    }

    // Create the instruction data buffer for init_policy
    const initData = {
        y0Total: new anchor.BN(1000000000),
        investorFeeShareBps: 5000,
        dailyCapQuote: new anchor.BN(100000000),
        minPayoutLamports: new anchor.BN(1000),
    };

    const DISCRIMINATOR = [45, 234, 110, 100, 209, 146, 191, 86]; // init_policy discriminator from IDL
    const dataBuffer = Buffer.concat([
        Buffer.from(DISCRIMINATOR),
        Buffer.from(initData.y0Total.toArrayLike(Buffer, 'le', 8)),
        Buffer.from([initData.investorFeeShareBps & 0xFF, (initData.investorFeeShareBps >> 8) & 0xFF]),
        Buffer.from(initData.dailyCapQuote.toArrayLike(Buffer, 'le', 8)),
        Buffer.from(initData.minPayoutLamports.toArrayLike(Buffer, 'le', 8)),
    ]);

    try {
        console.log('Attempting to initialize policy...');
        const instruction = new anchor.web3.TransactionInstruction({
            programId,
            keys: [
                { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
                { pubkey: policy, isSigner: false, isWritable: true },
                { pubkey: cpPool, isSigner: false, isWritable: false },
                { pubkey: quoteMint, isSigner: false, isWritable: false },
                { pubkey: creatorAta, isSigner: false, isWritable: true },
                { pubkey: treasuryAta || wallet.publicKey, isSigner: false, isWritable: true },
                { pubkey: vaultAuthority, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
            ],
            data: dataBuffer,
        });

        const tx = new anchor.web3.Transaction().add(instruction);
        const sig = await provider.sendAndConfirm(tx);

        console.log('Policy initialized successfully!');
        console.log('Transaction signature:', tx);
    } catch (err) {
        console.error('Error initializing policy:', err);
        if (err.logs) {
            console.error('Transaction logs:', err.logs);
        }
    }
}

main();