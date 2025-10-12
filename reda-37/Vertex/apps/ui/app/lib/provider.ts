'use client';

import { AnchorProvider } from '@coral-xyz/anchor';
import { Connection } from '@solana/web3.js';

export async function getAnchorProvider(rpcUrl: string): Promise<AnchorProvider> {
  const wallet = (window as any).solana;
  if (!wallet) {
    throw new Error('Phantom or compatible wallet not found in window.solana');
  }
  if (!wallet.isConnected) {
    await wallet.connect();
  }
  const connection = new Connection(rpcUrl, 'confirmed');
  return new AnchorProvider(connection, wallet, {});
}
