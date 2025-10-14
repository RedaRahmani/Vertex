const { readFileSync } = require('fs');
const path = require('path');
const anchor = require('@coral-xyz/anchor');
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID } = require('@solana/spl-token');

async function main() {
  const fixturePath = path.join(__dirname, '../target/local-fixture.json');
  const fixture = JSON.parse(readFileSync(fixturePath, 'utf8'));
  const idl = JSON.parse(readFileSync(path.join(__dirname, '../target/idl/keystone_fee_router.json'), 'utf8'));
  idl.address = fixture.programId;

  const connection = new anchor.web3.Connection(fixture.rpcUrl, 'confirmed');
  const wallet = anchor.Wallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });
  anchor.setProvider(provider);

  const programId = new PublicKey(fixture.programId);
  const quoteMint = new PublicKey(fixture.quoteMint);
  const cpPool = new PublicKey(fixture.cpPool);
  const creatorAta = new PublicKey(fixture.creatorAta);
  const treasuryAta = new PublicKey(fixture.treasuryAta);

  const program = new anchor.Program(idl, programId, provider);

  const [policy] = PublicKey.findProgramAddressSync([Buffer.from('policy'), cpPool.toBuffer()], programId);
  const [vaultAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), policy.toBuffer()],
    programId,
  );

  console.log('Initializing policy using fixture values...');
  console.log({ programId: programId.toBase58(), quoteMint: quoteMint.toBase58(), cpPool: cpPool.toBase58() });

  const tx = await program.methods
    .initPolicy({
      y0Total: new anchor.BN(1_000_000),
      investorFeeShareBps: 2_000,
      dailyCapQuote: new anchor.BN(0),
      minPayoutLamports: new anchor.BN(1),
    })
    .accounts({
      authority: provider.wallet.publicKey,
      policy,
      cpPool,
      quoteMint,
      creatorQuoteAta: creatorAta,
      treasuryQuoteAta: treasuryAta,
      vaultAuthority,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .rpc();

  console.log('Policy initialized! Signature:', tx);
}

main().catch(err => {
  console.error('Failed to initialize policy', err);
  process.exit(1);
});
