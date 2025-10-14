const { readFileSync } = require('fs');
const path = require('path');
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const anchor = require('@coral-xyz/anchor');
const { TOKEN_PROGRAM_ID } = require('@solana/spl-token');

async function main() {
  const fixturePath = path.join(__dirname, '../target/local-fixture.json');
  const fixture = JSON.parse(readFileSync(fixturePath, 'utf8'));

  const connection = new Connection(fixture.rpcUrl, 'confirmed');
  const wallet = anchor.Wallet.local();
  const payer = wallet.payer;

  const programId = new PublicKey(fixture.programId);
  const cpPool = new PublicKey(fixture.cpPool);
  const quoteMint = new PublicKey(fixture.quoteMint);
  const creatorAta = new PublicKey(fixture.creatorAta);
  const treasuryAta = new PublicKey(fixture.treasuryAta);

  const [policy] = PublicKey.findProgramAddressSync([Buffer.from('policy'), cpPool.toBuffer()], programId);
  const [vaultAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), policy.toBuffer()],
    programId,
  );

  const discriminator = Buffer.from([45, 234, 110, 100, 209, 146, 191, 86]);
  const data = Buffer.alloc(8 + 8 + 2 + 8 + 8);
  discriminator.copy(data, 0);
  let offset = 8;

  const writeU64 = (value) => {
    const bn = BigInt(value);
    data.writeBigUInt64LE(bn, offset);
    offset += 8;
  };

  writeU64(1_000_000); // y0_total
  data.writeUInt16LE(2_000, offset); // investor_fee_share_bps
  offset += 2;
  writeU64(0); // daily_cap_quote
  writeU64(1); // min_payout_lamports

  const keys = [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: policy, isSigner: false, isWritable: true },
    { pubkey: cpPool, isSigner: false, isWritable: false },
    { pubkey: quoteMint, isSigner: false, isWritable: false },
    { pubkey: creatorAta, isSigner: false, isWritable: true },
    { pubkey: treasuryAta, isSigner: false, isWritable: true },
    { pubkey: vaultAuthority, isSigner: false, isWritable: false },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];

  const ix = new anchor.web3.TransactionInstruction({
    keys,
    programId,
    data,
  });

  const tx = new Transaction().add(ix);
  const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
  console.log('Raw init_policy signature:', sig);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
