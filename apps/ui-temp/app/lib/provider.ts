'use client';

import { AnchorProvider } from '@coral-xyz/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import type { Transaction, VersionedTransaction } from '@solana/web3.js';

type WalletLike = {
  isConnected: boolean;
  connect: () => Promise<void>;
  publicKey?: PublicKey;
  signTransaction: (tx: Transaction | VersionedTransaction) => Promise<Transaction | VersionedTransaction>;
  signAllTransactions?: (txs: Array<Transaction | VersionedTransaction>) => Promise<Array<Transaction | VersionedTransaction>>;
};

export async function getAnchorProvider(rpcUrl: string): Promise<AnchorProvider> {
  const wallet = (window as any).solana as WalletLike | undefined;
  if (!wallet) {
    throw new Error('Phantom or compatible wallet not found in window.solana');
  }
  if (!wallet.isConnected) {
    await wallet.connect();
  }
  if (!wallet.publicKey) {
    throw new Error('Wallet did not provide a public key');
  }

  const connection = new Connection(rpcUrl, { commitment: 'confirmed' });

  const anchorWallet = {
    get publicKey() {
      return wallet.publicKey as PublicKey;
    },
    async signTransaction(tx: Transaction | VersionedTransaction) {
      return (await wallet.signTransaction(tx)) as Transaction | VersionedTransaction;
    },
    async signAllTransactions(txs: Array<Transaction | VersionedTransaction>) {
      if (!wallet.signAllTransactions) {
        throw new Error('Wallet does not support signAllTransactions');
      }
      return (await wallet.signAllTransactions(txs)) as Array<Transaction | VersionedTransaction>;
    },
  };

  return new AnchorProvider(connection, anchorWallet as any, {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });
}
