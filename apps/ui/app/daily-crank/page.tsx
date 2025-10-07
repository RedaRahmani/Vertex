"use client";
import { useState } from 'react';
import { AnchorProvider, Idl } from '@coral-xyz/anchor';
import { Connection, PublicKey, Transaction } from '@solana/web3.js';
import { buildCrankDistributeIx } from '../../../sdk/ts/src/feeRouter';

export default function Page() {
  const [rpcUrl, setRpcUrl] = useState('http://127.0.0.1:8899');
  const [programIdStr, setProgramIdStr] = useState('');
  const [idlJson, setIdlJson] = useState<string>('');
  const [policy, setPolicy] = useState('');
  const [cpPool, setCpPool] = useState('');
  const [treasuryAta, setTreasuryAta] = useState('');
  const [creatorAta, setCreatorAta] = useState('');
  const [investorAta, setInvestorAta] = useState('');
  const [stream, setStream] = useState('');
  const [cursor, setCursor] = useState(0);
  const [isLast, setIsLast] = useState(false);

  async function onSubmit() {
    const programId = new PublicKey(programIdStr);
    const idl = JSON.parse(idlJson) as Idl;
    const conn = new Connection(rpcUrl);
    // @ts-ignore
    const provider = new AnchorProvider(conn, (window as any).solana, {});
    const ix = await buildCrankDistributeIx({
      provider,
      programId,
      idl,
      policy: new PublicKey(policy),
      cpPool: new PublicKey(cpPool),
      payer: provider.wallet.publicKey,
      treasuryQuoteAta: new PublicKey(treasuryAta),
      creatorQuoteAta: new PublicKey(creatorAta),
      investorQuoteAta: new PublicKey(investorAta),
      stream: new PublicKey(stream),
      pageCursor: BigInt(cursor),
      isLastPage: isLast
    });
    const tx = new Transaction().add(ix);
    (tx as any).feePayer = provider.wallet.publicKey;
    const sig = await provider.sendAndConfirm(tx);
    alert(`Crank tx: ${sig}`);
  }

  return (
    <main className="space-y-3">
      <h2 className="text-lg font-medium">Daily Crank</h2>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <input className="border p-2" placeholder="RPC URL" value={rpcUrl} onChange={e=>setRpcUrl(e.target.value)} />
        <input className="border p-2" placeholder="Program ID" value={programIdStr} onChange={e=>setProgramIdStr(e.target.value)} />
        <textarea className="border p-2 col-span-2" rows={6} placeholder="Paste keystone_fee_router IDL JSON" value={idlJson} onChange={e=>setIdlJson(e.target.value)} />
        <input className="border p-2" placeholder="Policy PDA" value={policy} onChange={e=>setPolicy(e.target.value)} />
        <input className="border p-2" placeholder="Meteora cp_pool" value={cpPool} onChange={e=>setCpPool(e.target.value)} />
        <input className="border p-2" placeholder="Treasury quote ATA" value={treasuryAta} onChange={e=>setTreasuryAta(e.target.value)} />
        <input className="border p-2" placeholder="Creator quote ATA" value={creatorAta} onChange={e=>setCreatorAta(e.target.value)} />
        <input className="border p-2" placeholder="Investor quote ATA" value={investorAta} onChange={e=>setInvestorAta(e.target.value)} />
        <input className="border p-2" placeholder="Stream account (mock)" value={stream} onChange={e=>setStream(e.target.value)} />
        <input className="border p-2" placeholder="Page cursor" value={cursor} onChange={e=>setCursor(parseInt(e.target.value||'0'))} />
        <label className="flex items-center space-x-2"><input type="checkbox" checked={isLast} onChange={e=>setIsLast(e.target.checked)} /><span>Is last page?</span></label>
      </div>
      <button onClick={onSubmit} className="px-3 py-2 bg-blue-600 text-white text-sm rounded">Run Crank</button>
    </main>
  );
}

