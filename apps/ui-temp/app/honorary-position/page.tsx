"use client";

import { useMemo, useState } from 'react';
import { PublicKey, Transaction } from '@solana/web3.js';
import { buildInitHonoraryPositionIx, findPositionPda } from '@keystone/sdk';
import { useEnvironment } from '../environment';
import { getAnchorProvider } from '../lib/provider';
import { signAndSendTransaction } from '../lib/signAndSendTransaction';
import { LOCAL_FIXTURE } from '../config';

const VAULT_SEED = Buffer.from('vault');
const FEE_POS_OWNER_SEED = Buffer.from('investor_fee_pos_owner');

export default function Page() {
  const { rpcUrl, programKey, idl } = useEnvironment();
  const [policy, setPolicy] = useState<string>(LOCAL_FIXTURE.policy);
  const [cpPool, setCpPool] = useState<string>(LOCAL_FIXTURE.cpPool);
  const [quoteMint, setQuoteMint] = useState<string>(LOCAL_FIXTURE.quoteMint);
  const [cpPosition, setCpPosition] = useState<string>(LOCAL_FIXTURE.cpPosition);
  const [status, setStatus] = useState<string | null>(null);

  const derived = useMemo(() => {
    if (!programKey || !policy) {
      return { honorary: '', owner: '' };
    }
    try {
      const policyKey = new PublicKey(policy);
      const [honorary] = findPositionPda(policyKey, programKey);
      const [owner] = PublicKey.findProgramAddressSync(
        [VAULT_SEED, policyKey.toBuffer(), FEE_POS_OWNER_SEED],
        programKey,
      );
      return { honorary: honorary.toBase58(), owner: owner.toBase58() };
    } catch {
      return { honorary: '', owner: '' };
    }
  }, [policy, programKey]);

  async function onSubmit() {
    if (!programKey || !idl) {
      setStatus('Configure program ID and IDL in the Environment panel.');
      return;
    }
    let policyKey: PublicKey;
    let poolKey: PublicKey;
    let quoteKey: PublicKey;
    let positionKey: PublicKey;
    try {
      policyKey = new PublicKey(policy);
      poolKey = new PublicKey(cpPool);
      quoteKey = new PublicKey(quoteMint);
      positionKey = new PublicKey(cpPosition);
    } catch {
      setStatus('Invalid public key in inputs.');
      return;
    }

    try {
      setStatus('Submitting transaction...');
      const provider = await getAnchorProvider(rpcUrl);
      const ix = await buildInitHonoraryPositionIx({
        provider,
        programId: programKey,
        idl,
        policy: policyKey,
        cpPool: poolKey,
        quoteMint: quoteKey,
        cpPosition: positionKey,
        authority: provider.wallet.publicKey,
      });
      const tx = new Transaction().add(ix);
      const sig = await signAndSendTransaction(provider, tx);
      setStatus(`Init Honorary Position tx: ${sig}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setStatus(`Error: ${message}`);
    }
  }

  return (
    <main className="space-y-4">
      <h2 className="text-lg font-medium">Honorary Position</h2>
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
          placeholder="Quote mint"
          value={quoteMint}
          onChange={e => setQuoteMint(e.target.value)}
        />
        <input
          className="rounded border p-2"
          placeholder="cp-position account"
          value={cpPosition}
          onChange={e => setCpPosition(e.target.value)}
        />
      </div>
      {derived.honorary && (
        <div className="rounded border border-gray-200 bg-gray-50 p-3 text-xs">
          <div className="flex flex-col gap-1">
            <span><strong>Honorary Position PDA:</strong> {derived.honorary}</span>
            <span><strong>Owner PDA:</strong> {derived.owner}</span>
          </div>
        </div>
      )}
      <button
        onClick={onSubmit}
        className="rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white"
      >
        Initialize Honorary Position
      </button>
      {status && <p className="text-sm text-gray-700">{status}</p>}
    </main>
  );
}
