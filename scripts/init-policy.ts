const anchor = require("@coral-xyz/anchor");
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Connection, Keypair } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");
const fs = require('fs');
const path = require('path');

async function main() {
  // Parse command line arguments
  const args = process.argv.slice(2);
  const programId = new PublicKey(args[0]); // Program ID
  const poolPubkey = new PublicKey(args[1]); // Pool
  const quoteMint = new PublicKey(args[2]); // Quote mint
  const y0Total = new anchor.BN(args[3]); // Y0 total
  const investorFeeShareBps = parseInt(args[4]); // Investor fee share bps
  const dailyCapQuote = new anchor.BN(args[5]); // Daily cap quote
  const minPayoutLamports = new anchor.BN(args[6]); // Min payout

  // Set up connection and wallet
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/keystone_fee_router.json"), 'utf8'));
  const wallet = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(path.join(process.env.HOME!, ".config/solana/id.json"), 'utf8')))
  );

  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);

  const program = new anchor.Program(idl, programId, provider);

  // Derive policy PDA
  const [policyPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("policy"), poolPubkey.toBuffer()],
    programId
  );

  // Derive vault authority PDA
  const [vaultAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), policyPda.toBuffer()],
    programId
  );

  // Get ATAs
  const creatorQuoteAta = await anchor.utils.token.associatedAddress({
    mint: quoteMint,
    owner: provider.wallet.publicKey
  });

  const treasuryQuoteAta = await anchor.utils.token.associatedAddress({
    mint: quoteMint, 
    owner: vaultAuthority
  });

  try {
    const tx = await program.methods
      .initPolicy({
        y0Total,
        investorFeeShareBps,
        dailyCapQuote,
        minPayoutLamports,
      })
      .accounts({
        authority: provider.wallet.publicKey,
        policy: policyPda,
        cpPool: poolPubkey,
        quoteMint,
        creatorQuoteAta,
        treasuryQuoteAta,
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("Policy initialized successfully!");
    console.log("Transaction signature:", tx);
    console.log("Policy address:", policyPda.toString());
    console.log("Vault authority:", vaultAuthority.toString());
    console.log("Treasury ATA:", treasuryQuoteAta.toString());
  } catch (err) {
    console.error("Error initializing policy:", err);
    process.exit(1);
  }
}

main().catch(console.error);