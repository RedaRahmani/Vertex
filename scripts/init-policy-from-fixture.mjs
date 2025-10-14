import { readFileSync } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { Connection, PublicKey, Transaction } from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';
import { buildInitPolicyIx } from '@keystone/sdk';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  const fixturePath = path.join(__dirname, '../target/local-fixture.json');
  const fixture = JSON.parse(readFileSync(fixturePath, 'utf8'));
  const idl = JSON.parse(readFileSync(path.join(__dirname, '../target/idl/keystone_fee_router.json'), 'utf8'));

  const connection = new Connection(fixture.rpcUrl, 'confirmed');
  const wallet = anchor.Wallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });
  anchor.setProvider(provider);

  const programId = new PublicKey(fixture.programId);
  const cpPool = new PublicKey(fixture.cpPool);
  const quoteMint = new PublicKey(fixture.quoteMint);
  const creatorAta = new PublicKey(fixture.creatorAta);
  const treasuryAta = new PublicKey(fixture.treasuryAta);

  const ix = await buildInitPolicyIx({
    provider,
    programId,
    idl,
    cpPool,
    quoteMint,
    creatorQuoteAta: creatorAta,
    treasuryQuoteAta: treasuryAta,
    y0Total: BigInt(1_000_000),
    investorFeeShareBps: 2_000,
    dailyCapQuote: BigInt(0),
    minPayoutLamports: BigInt(1),
    authority: provider.wallet.publicKey,
  });

  const tx = new Transaction().add(ix);
  const sig = await provider.sendAndConfirm(tx);
  console.log('Init policy transaction:', sig);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
