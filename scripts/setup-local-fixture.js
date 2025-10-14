const { readFileSync, writeFileSync } = require('fs');
const path = require('path');
const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} = require('@solana/web3.js');
const {
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  createMint,
} = require('@solana/spl-token');

const RPC_URL = process.env.RPC_URL ?? 'http://127.0.0.1:8899';
const PROGRAM_ID = new PublicKey('B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc');
const METEORA_PROGRAM_ID = new PublicKey('cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG');
const KEYPAIR_PATH =
  process.env.SOLANA_KEYPAIR ?? path.join(process.env.HOME ?? '.', '.config/solana/id.json');
const OUTPUT_PATH = path.join(__dirname, '../target/local-fixture.json');
const IDL_PATH = path.join(__dirname, '../target/idl/keystone_fee_router.json');

function loadKeypair() {
  const raw = readFileSync(KEYPAIR_PATH, 'utf8');
  const secret = Uint8Array.from(JSON.parse(raw));
  return Keypair.fromSecretKey(secret);
}

async function main() {
  const payer = loadKeypair();
  const connection = new Connection(RPC_URL, 'confirmed');

  console.log('Payer:', payer.publicKey.toBase58());

  const balance = await connection.getBalance(payer.publicKey);
  if (balance < 2 * 1_000_000_000) {
    console.log('Requesting airdrop...');
    const sig = await connection.requestAirdrop(payer.publicKey, 5_000_000_000);
    await connection.confirmTransaction(sig, 'confirmed');
    console.log('Airdrop signature:', sig);
  }

  console.log('Creating quote mint...');
  const quoteMint = await createMint(connection, payer, payer.publicKey, null, 6);
  console.log('Quote mint:', quoteMint.toBase58());

  const cpPool = Keypair.generate();
  const rentLamports = await connection.getMinimumBalanceForRentExemption(0);
  const createPoolIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: cpPool.publicKey,
    lamports: rentLamports,
    space: 0,
    programId: METEORA_PROGRAM_ID,
  });
  console.log('Creating cp_pool account:', cpPool.publicKey.toBase58());
  await sendAndConfirmTransaction(connection, new Transaction().add(createPoolIx), [payer, cpPool]);

  const cpPosition = Keypair.generate();
  const createPositionIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: cpPosition.publicKey,
    lamports: rentLamports,
    space: 0,
    programId: METEORA_PROGRAM_ID,
  });
  console.log('Creating cp_position account:', cpPosition.publicKey.toBase58());
  await sendAndConfirmTransaction(connection, new Transaction().add(createPositionIx), [payer, cpPosition]);

  const [policy] = PublicKey.findProgramAddressSync(
    [Buffer.from('policy'), cpPool.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const [vault] = PublicKey.findProgramAddressSync([Buffer.from('vault'), policy.toBuffer()], PROGRAM_ID);

  const creatorAta = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    quoteMint,
    payer.publicKey,
    false,
    'confirmed',
  );
  console.log('Creator ATA:', creatorAta.address.toBase58());

  const treasuryAta = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    quoteMint,
    vault,
    true,
    'confirmed',
  );
  console.log('Vault ATA:', treasuryAta.address.toBase58());

  await mintTo(connection, payer, quoteMint, treasuryAta.address, payer.publicKey, 100_000_000);
  await mintTo(connection, payer, quoteMint, creatorAta.address, payer.publicKey, 100_000_000);

  const fixture = {
    rpcUrl: RPC_URL,
    programId: PROGRAM_ID.toBase58(),
    quoteMint: quoteMint.toBase58(),
    cpPool: cpPool.publicKey.toBase58(),
    cpPosition: cpPosition.publicKey.toBase58(),
    policy: policy.toBase58(),
    vaultAuthority: vault.toBase58(),
    creatorAta: creatorAta.address.toBase58(),
    treasuryAta: treasuryAta.address.toBase58(),
    cpPoolSecret: Array.from(cpPool.secretKey),
    cpPositionSecret: Array.from(cpPosition.secretKey),
  };

  writeFileSync(OUTPUT_PATH, JSON.stringify(fixture, null, 2));
  console.log('Fixture written to', OUTPUT_PATH);

  const configBody = `export type Fixture = {
  rpcUrl: string;
  programId: string;
  quoteMint: string;
  cpPool: string;
  policy: string;
  cpPosition: string;
  creatorAta: string;
  treasuryAta: string;
};

export const LOCAL_FIXTURE: Fixture = ${JSON.stringify(
    {
      rpcUrl: fixture.rpcUrl,
      programId: fixture.programId,
      quoteMint: fixture.quoteMint,
      cpPool: fixture.cpPool,
      policy: fixture.policy,
      cpPosition: fixture.cpPosition,
      creatorAta: fixture.creatorAta,
      treasuryAta: fixture.treasuryAta,
    },
    null,
    2,
  )};
`;

  const uiTempConfig = path.join(__dirname, '../apps/ui-temp/app/config.ts');
  const uiConfig = path.join(__dirname, '../apps/ui/app/config.ts');
  writeFileSync(uiTempConfig, configBody);
  writeFileSync(uiConfig, configBody);
  console.log('Wrote UI config defaults.');

  const idlSource = readFileSync(IDL_PATH, 'utf8');
  const idlBody = `export const DEFAULT_IDL = ${idlSource};\n`;
  writeFileSync(path.join(__dirname, '../apps/ui-temp/app/idl.ts'), idlBody);
  writeFileSync(path.join(__dirname, '../apps/ui/app/idl.ts'), idlBody);
  console.log('Wrote IDL defaults.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
