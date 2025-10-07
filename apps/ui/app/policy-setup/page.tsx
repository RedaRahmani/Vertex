"use client";
import { useState } from 'react';
import { AnchorProvider, Idl, Program } from '@coral-xyz/anchor';
import { Connection, PublicKey, Transaction } from '@solana/web3.js';
import { METEORA_DLMM_V2, buildInitPolicyIx } from '../../../sdk/ts/src/feeRouter';

export default function Page() {
  const [rpcUrl, setRpcUrl] = useState('http://127.0.0.1:8899');
  const [programIdStr, setProgramIdStr] = useState('');
  const [idlJson, setIdlJson] = useState<string>('');
  const [cpPool, setCpPool] = useState('');
  const [quoteMint, setQuoteMint] = useState('');
  const [creatorAta, setCreatorAta] = useState('');
  const [treasuryAta, setTreasuryAta] = useState('');
  const [bps, setBps] = useState(2000);
  const [y0, setY0] = useState(100000);
  const [cap, setCap] = useState(0);
  const [minPayout, setMinPayout] = useState(1);

  async function onSubmit() {
    const programId = new PublicKey(programIdStr);
    const idl = JSON.parse(idlJson) as Idl;
    const conn = new Connection(rpcUrl);
    // @ts-ignore - simplified provider using window.solana
    const provider = new AnchorProvider(conn, (window as any).solana, {});
    const ix = await buildInitPolicyIx({
      provider,
      programId,
      idl,
      cpPool: new PublicKey(cpPool),
      quoteMint: new PublicKey(quoteMint),
      creatorQuoteAta: new PublicKey(creatorAta),
      treasuryQuoteAta: new PublicKey(treasuryAta),
      y0Total: BigInt(y0),
      investorFeeShareBps: bps,
      dailyCapQuote: BigInt(cap),
      minPayoutLamports: BigInt(minPayout),
      authority: provider.wallet.publicKey,
      tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
    });
    const tx = new Transaction().add(ix);
    const sig = await provider.sendAndConfirm(tx);
    alert(`InitPolicy tx: ${sig}`);
  }

  return (
    <main className="space-y-3">
      <h2 className="text-lg font-medium">Policy Setup</h2>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <input className="border p-2" placeholder="RPC URL" value={rpcUrl} onChange={e=>setRpcUrl(e.target.value)} />
        <input className="border p-2" placeholder="Program ID" value={programIdStr} onChange={e=>setProgramIdStr(e.target.value)} />
        <textarea className="border p-2 col-span-2" rows={6} placeholder="Paste keystone_fee_router IDL JSON" value={idlJson} onChange={e=>setIdlJson(e.target.value)} />
        <input className="border p-2" placeholder="Meteora cp_pool" value={cpPool} onChange={e=>setCpPool(e.target.value)} />
        <input className="border p-2" placeholder="Quote mint" value={quoteMint} onChange={e=>setQuoteMint(e.target.value)} />
        <input className="border p-2" placeholder="Creator quote ATA" value={creatorAta} onChange={e=>setCreatorAta(e.target.value)} />
        <input className="border p-2" placeholder="Treasury quote ATA (owner: vault PDA)" value={treasuryAta} onChange={e=>setTreasuryAta(e.target.value)} />
        <input className="border p-2" placeholder="Investor fee share bps" value={bps} onChange={e=>setBps(parseInt(e.target.value||'0'))} />
        <input className="border p-2" placeholder="Y0 total" value={y0} onChange={e=>setY0(parseInt(e.target.value||'0'))} />
        <input className="border p-2" placeholder="Daily cap (quote)" value={cap} onChange={e=>setCap(parseInt(e.target.value||'0'))} />
        <input className="border p-2" placeholder="Min payout lamports" value={minPayout} onChange={e=>setMinPayout(parseInt(e.target.value||'0'))} />
      </div>
      <button onClick={onSubmit} className="px-3 py-2 bg-blue-600 text-white text-sm rounded">Initialize Policy</button>
    </main>
  );
}

