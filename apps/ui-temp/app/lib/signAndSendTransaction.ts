import type { AnchorProvider } from '@coral-xyz/anchor';
import {
  Commitment,
  SendTransactionError,
  Transaction,
  TransactionSignature,
} from '@solana/web3.js';

const DEFAULT_COMMITMENT: Commitment = 'confirmed';

export async function signAndSendTransaction(
  provider: AnchorProvider,
  tx: Transaction,
  commitment: Commitment = DEFAULT_COMMITMENT,
): Promise<TransactionSignature> {
  if (!provider.wallet?.publicKey) {
    throw new Error('Wallet not connected');
  }

  tx.feePayer = tx.feePayer ?? provider.wallet.publicKey;

  const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash(commitment);
  tx.recentBlockhash = blockhash;

  const signed = await provider.wallet.signTransaction(tx);

  let signature: TransactionSignature;
  try {
    signature = await provider.connection.sendRawTransaction(signed.serialize(), {
      skipPreflight: false,
      preflightCommitment: commitment,
    });
  } catch (error) {
    if (error instanceof SendTransactionError) {
      try {
        const logs = await error.getLogs(provider.connection);
        if (logs && logs.length > 0) {
          throw new Error(`${error.message}\nLogs:\n${logs.join('\n')}`);
        }
      } catch {
        // ignore failures fetching logs
      }
    }
    throw error;
  }

  await provider.connection.confirmTransaction(
    { blockhash, lastValidBlockHeight, signature },
    commitment,
  );

  return signature;
}
