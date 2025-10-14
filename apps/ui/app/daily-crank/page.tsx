"use client";

import { useState } from 'react';
import { ComputeBudgetProgram, PublicKey, Transaction } from '@solana/web3.js';
import { buildCrankDistributeIx } from '@keystone/sdk';
import { useEnvironment } from '../environment';
import { getAnchorProvider } from '../lib/provider';
import { signAndSendTransaction } from '../lib/signAndSendTransaction';

export default function Page() {
  const { rpcUrl, programKey, idl } = useEnvironment();
  const [policy, setPolicy] = useState('');
  const [cpPool, setCpPool] = useState('');
  const [treasuryAta, setTreasuryAta] = useState('');
  const [creatorAta, setCreatorAta] = useState('');
  const [investorAta, setInvestorAta] = useState('');
  const [stream, setStream] = useState('');
  const [cursor, setCursor] = useState(0);
  const [isLast, setIsLast] = useState(false);
  const [computePrice, setComputePrice] = useState(0);
  const [status, setStatus] = useState<string | null>(null);

  async function onSubmit() {
    if (!programKey || !idl) {
      setStatus('Configure program ID and IDL in the Environment panel.');
      return;
    }
    let policyKey: PublicKey;
    let poolKey: PublicKey;
    let treasuryKey: PublicKey;
    let creatorKey: PublicKey;
    let investorKey: PublicKey;
    let streamKey: PublicKey;
    try {
      policyKey = new PublicKey(policy);
      poolKey = new PublicKey(cpPool);
      treasuryKey = new PublicKey(treasuryAta);
      creatorKey = new PublicKey(creatorAta);
      investorKey = new PublicKey(investorAta);
      streamKey = new PublicKey(stream);
    } catch {
      setStatus('Invalid public key in inputs.');
      return;
    }

    try {
      setStatus('Submitting transaction...');
      const provider = await getAnchorProvider(rpcUrl);
      const ix = await buildCrankDistributeIx({
        provider,
        programId: programKey,
        idl,
        policy: policyKey,
        cpPool: poolKey,
        payer: provider.wallet.publicKey,
        treasuryQuoteAta: treasuryKey,
        creatorQuoteAta: creatorKey,
        investorQuoteAta: investorKey,
        stream: streamKey,
        pageCursor: BigInt(cursor),
        isLastPage: isLast,
      });
      const tx = new Transaction();
      if (computePrice > 0) {
        tx.add(ComputeBudgetProgram.setComputeUnitPrice({ microLamports: computePrice }));
      }
      tx.add(ix);
      const sig = await signAndSendTransaction(provider, tx);
      setStatus(`Crank tx: ${sig}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setStatus(`Error: ${message}`);
    }
  }

  return (
    <main className="space-y-4">
      <h2 className="text-lg font-medium">Daily Crank</h2>
      <div className="grid grid-cols-1 gap-3 text-sm md:grid-cols-2">
        <input
          className="rounded border p-2"
          placeholder="Policy PDA"
          value={policy}
          onChange={e => setPolicy(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Meteora cp_pool"
          value={cpPool}
          onChange={e => setCpPool(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Treasury quote ATA"
          value={treasuryAta}
          onChange={e => setTreasuryAta(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Creator quote ATA"
          value={creatorAta}
          onChange={e => setCreatorAta(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Investor quote ATA"
          value={investorAta}
          onChange={e => setInvestorAta(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Stream account"
          value={stream}
          onChange={e => setStream(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Page cursor"
          type="number"
          value={cursor}
          onChange={e => setCursor(Number(e.target.value) || 0)}
        />
        <div className="flex items-center space-x-2">
          <input
            type="checkbox"
            checked={isLast}
            onChange={e => setIsLast(e.target.checked)}
          />
          <span>Is last page?</span>
        </div>
        <input
          className="rounded border p-2"
          placeholder="CU price (micro-lamports)"
          type="number"
          value={computePrice}
          onChange={e => setComputePrice(Number(e.target.value) || 0)}
        />
      </div>
      <button
        onClick={onSubmit}
        className="rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white"
      >
        Run Crank
      </button>
      {status && <p className="text-sm text-gray-700">{status}</p>}
    </main>
  );
}
