"use client";
import { useState } from 'react';
import { AnchorProvider, Idl } from '@coral-xyz/anchor';
import { Connection, PublicKey, Transaction } from '@solana/web3.js';
import { buildInitHonoraryPositionIx } from '../../../sdk/ts/src/feeRouter';

export default function Page() {
  const [rpcUrl, setRpcUrl] = useState('http://127.0.0.1:8899');
  const [programIdStr, setProgramIdStr] = useState('');
  const [idlJson, setIdlJson] = useState<string>('');
  const [policy, setPolicy] = useState('');
  const [cpPool, setCpPool] = useState('');
  const [quoteMint, setQuoteMint] = useState('');
  const [cpPosition, setCpPosition] = useState('');

  async function onSubmit() {
    const programId = new PublicKey(programIdStr);
    const idl = JSON.parse(idlJson) as Idl;
    const conn = new Connection(rpcUrl);
    // @ts-ignore
    const provider = new AnchorProvider(conn, (window as any).solana, {});
    const ix = await buildInitHonoraryPositionIx({
      provider,
      programId,
      idl,
      policy: new PublicKey(policy),
      cpPool: new PublicKey(cpPool),
      quoteMint: new PublicKey(quoteMint),
      cpPosition: new PublicKey(cpPosition),
      authority: provider.wallet.publicKey
    });
    const tx = new Transaction().add(ix);
    const sig = await provider.sendAndConfirm(tx);
    alert(`InitHonoraryPosition tx: ${sig}`);
  }

  return (
    <main className="space-y-3">
      <h2 className="text-lg font-medium">Honorary Position</h2>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <input className="border p-2" placeholder="RPC URL" value={rpcUrl} onChange={e=>setRpcUrl(e.target.value)} />
        <input className="border p-2" placeholder="Program ID" value={programIdStr} onChange={e=>setProgramIdStr(e.target.value)} />
        <textarea className="border p-2 col-span-2" rows={6} placeholder="Paste keystone_fee_router IDL JSON" value={idlJson} onChange={e=>setIdlJson(e.target.value)} />
        <input className="border p-2" placeholder="Policy PDA" value={policy} onChange={e=>setPolicy(e.target.value)} />
        <input className="border p-2" placeholder="Meteora cp_pool" value={cpPool} onChange={e=>setCpPool(e.target.value)} />
        <input className="border p-2" placeholder="Quote mint" value={quoteMint} onChange={e=>setQuoteMint(e.target.value)} />
        <input className="border p-2" placeholder="cp-position account" value={cpPosition} onChange={e=>setCpPosition(e.target.value)} />
      </div>
      <button onClick={onSubmit} className="px-3 py-2 bg-blue-600 text-white text-sm rounded">Initialize Position</button>
    </main>
  );
}

