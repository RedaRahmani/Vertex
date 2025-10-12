"use client";

import { useMemo, useState } from 'react';
import { PublicKey, Transaction } from '@solana/web3.js';
import { buildInitPolicyIx, findPolicyPda, findVaultAuthorityPda } from '@keystone/sdk';
import { useEnvironment } from '../environment';
import { getAnchorProvider } from '../lib/provider';

export default function Page() {
  const { rpcUrl, programKey, idl } = useEnvironment();
  const [cpPool, setCpPool] = useState('');
  const [quoteMint, setQuoteMint] = useState('');
  const [creatorAta, setCreatorAta] = useState('');
  const [treasuryAta, setTreasuryAta] = useState('');
  const [bps, setBps] = useState(2000);
  const [y0, setY0] = useState(100000);
  const [cap, setCap] = useState(0);
  const [minPayout, setMinPayout] = useState(1);
  const [status, setStatus] = useState<string | null>(null);

  const derived = useMemo(() => {
    if (!programKey || !cpPool) {
      return { policy: '', vault: '' };
    }
    try {
      const poolKey = new PublicKey(cpPool);
      const [policy] = findPolicyPda(poolKey, programKey);
      const [vault] = findVaultAuthorityPda(policy, programKey);
      return { policy: policy.toBase58(), vault: vault.toBase58() };
    } catch {
      return { policy: '', vault: '' };
    }
  }, [cpPool, programKey]);

  async function onSubmit() {
    if (!programKey || !idl) {
      setStatus('Configure program ID and IDL in the Environment panel.');
      return;
    }
    let poolKey: PublicKey;
    let quoteKey: PublicKey;
    let creatorKey: PublicKey;
    let treasuryKey: PublicKey;
    try {
      poolKey = new PublicKey(cpPool);
      quoteKey = new PublicKey(quoteMint);
      creatorKey = new PublicKey(creatorAta);
      treasuryKey = new PublicKey(treasuryAta);
    } catch {
      setStatus('Invalid public key in inputs.');
      return;
    }

    try {
      setStatus('Submitting transaction...');
      const provider = await getAnchorProvider(rpcUrl);
      const ix = await buildInitPolicyIx({
        provider,
        programId: programKey,
        idl,
        cpPool: poolKey,
        quoteMint: quoteKey,
        creatorQuoteAta: creatorKey,
        treasuryQuoteAta: treasuryKey,
        y0Total: BigInt(y0),
        investorFeeShareBps: Number.isFinite(bps) ? bps : 0,
        dailyCapQuote: BigInt(cap),
        minPayoutLamports: BigInt(minPayout),
        authority: provider.wallet.publicKey,
      });
      const tx = new Transaction().add(ix);
      (tx as any).feePayer = provider.wallet.publicKey;
      const sig = await provider.sendAndConfirm(tx);
      setStatus(`Init Policy tx: ${sig}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setStatus(`Error: ${message}`);
    }
  }

  return (
    <main className="space-y-4">
      <h2 className="text-lg font-medium">Policy Setup</h2>
      <div className="grid grid-cols-1 gap-3 text-sm md:grid-cols-2">
        <input
          className="rounded border p-2"
          placeholder="Meteora cp_pool"
          value={cpPool}
          onChange={e => setCpPool(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Quote mint"
          value={quoteMint}
          onChange={e => setQuoteMint(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Creator quote ATA"
          value={creatorAta}
          onChange={e => setCreatorAta(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Treasury quote ATA (vault-owned)"
          value={treasuryAta}
          onChange={e => setTreasuryAta(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="Investor fee share bps"
          value={bps}
          onChange={e => setBps(Number(e.target.value) || 0)}
          type="number"
        />
        <input
          className="rounded border p-2"
          placeholder="Y0 total"
          value={y0}
          onChange={e => setY0(Number(e.target.value) || 0)}
          type="number"
        />
        <input
          className="rounded border p-2"
          placeholder="Daily cap (quote)"
          value={cap}
          onChange={e => setCap(Number(e.target.value) || 0)}
          type="number"
        />
        <input
          className="rounded border p-2"
          placeholder="Min payout (lamports)"
          value={minPayout}
          onChange={e => setMinPayout(Number(e.target.value) || 0)}
          type="number"
        />
      </div>
      {derived.policy && (
        <div className="rounded border border-gray-200 bg-gray-50 p-3 text-xs">
          <div className="flex flex-col gap-1">
            <span><strong>Policy PDA:</strong> {derived.policy}</span>
            <span><strong>Vault PDA:</strong> {derived.vault}</span>
          </div>
        </div>
      )}
      <button
        onClick={onSubmit}
        className="rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white"
      >
        Initialize Policy
      </button>
      {status && <p className="text-sm text-gray-700">{status}</p>}
    </main>
  );
}
