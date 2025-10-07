"use client";

import { useMemo, useState } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import { getProgram, findPolicyPda, findProgressPda } from '@keystone/sdk';
import { useEnvironment } from './environment';
import { getAnchorProvider } from './lib/provider';

type PolicyInfo = {
  authority: string;
  cpPool: string;
  quoteMint: string;
  y0Total: string;
  investorFeeShareBps: number;
  dailyCapQuote: string;
  minPayout: string;
  creatorQuoteAta: string;
  treasuryQuoteAta: string;
};

type ProgressInfo = {
  currentDay: string;
  claimed: string;
  distributed: string;
  carry: string;
  pageCursor: string;
  dayClosed: boolean;
};

type EventRow = {
  signature: string;
  slot: number;
  blockTime?: number | null;
};

export default function Page() {
  const { rpcUrl, programKey, idl } = useEnvironment();
  const [cpPool, setCpPool] = useState('');
  const [policyInfo, setPolicyInfo] = useState<PolicyInfo | null>(null);
  const [progressInfo, setProgressInfo] = useState<ProgressInfo | null>(null);
  const [events, setEvents] = useState<EventRow[]>([]);
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const pdas = useMemo(() => {
    if (!programKey || !cpPool) {
      return { policy: '', progress: '' };
    }
    try {
      const poolKey = new PublicKey(cpPool);
      const [policyPda] = findPolicyPda(poolKey, programKey);
      const [progressPda] = findProgressPda(poolKey, programKey);
      return { policy: policyPda.toBase58(), progress: progressPda.toBase58() };
    } catch {
      return { policy: '', progress: '' };
    }
  }, [cpPool, programKey]);

  async function fetchState() {
    if (!programKey || !idl) {
      setStatus('Configure program ID and IDL in the Environment panel.');
      return;
    }
    let poolKey: PublicKey;
    try {
      poolKey = new PublicKey(cpPool);
    } catch {
      setStatus('Invalid cp_pool public key.');
      return;
    }

    setLoading(true);
    setStatus('Fetching on-chain state...');

    try {
      const provider = await getAnchorProvider(rpcUrl);
      const program = getProgram(provider, programKey, idl);
      const accountClient = program.account as any;
      const [policyPda] = findPolicyPda(poolKey, programKey);
      const [progressPda] = findProgressPda(poolKey, programKey);

      const policyAccount = await accountClient.policy.fetch(policyPda);
      const progressAccount = await accountClient.progress.fetch(progressPda);

      setPolicyInfo({
        authority: policyAccount.authority.toBase58(),
        cpPool: policyAccount.cpPool.toBase58(),
        quoteMint: policyAccount.quoteMint.toBase58(),
        creatorQuoteAta: policyAccount.creatorQuoteAta.toBase58(),
        treasuryQuoteAta: policyAccount.treasuryQuoteAta.toBase58(),
        y0Total: policyAccount.y0Total.toString(),
        investorFeeShareBps: Number(policyAccount.investorFeeShareBps),
        dailyCapQuote: policyAccount.dailyCapQuote.toString(),
        minPayout: policyAccount.minPayoutLamports.toString(),
      });

      setProgressInfo({
        currentDay: progressAccount.currentDay.toString(),
        claimed: progressAccount.claimedQuoteToday.toString(),
        distributed: progressAccount.distributedQuoteToday.toString(),
        carry: progressAccount.carryQuoteToday.toString(),
        pageCursor: progressAccount.pageCursor.toString(),
        dayClosed: !!progressAccount.dayClosed,
      });

      const connection: Connection = provider.connection;
      const signatures = await connection.getSignaturesForAddress(policyPda, { limit: 8 });
      setEvents(
        signatures.map(sig => ({
          signature: sig.signature,
          slot: sig.slot,
          blockTime: sig.blockTime,
        })),
      );

      setStatus('Fetched latest state.');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setStatus(`Error: ${message}`);
      setPolicyInfo(null);
      setProgressInfo(null);
      setEvents([]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="space-y-6">
      <section>
        <h2 className="mb-3 text-lg font-medium">Dashboard</h2>
        <p className="text-sm text-gray-600">
          Inspect policy configuration and progress for a given Meteora cp_pool. Use the forms below to
          initialize policy and honorary position, then crank the distribution.
        </p>
      </section>

      <section className="space-y-3 text-sm">
        <div className="flex flex-col gap-2 md:flex-row md:items-end">
          <div className="flex-1">
            <label className="block text-xs font-medium uppercase tracking-wide text-gray-600">
              Meteora cp_pool
            </label>
            <input
              className="mt-1 w-full rounded border p-2"
              placeholder="cp_pool public key"
              value={cpPool}
              onChange={e => setCpPool(e.target.value)}
            />
          </div>
          <button
            onClick={fetchState}
            className="rounded bg-blue-600 px-3 py-2 text-white"
            disabled={loading}
          >
            {loading ? 'Loading...' : 'Fetch State'}
          </button>
        </div>
        {pdas.policy && (
          <div className="rounded border border-gray-200 bg-gray-50 p-3 text-xs">
            <div className="flex flex-col gap-1">
              <span><strong>Policy PDA:</strong> {pdas.policy}</span>
              <span><strong>Progress PDA:</strong> {pdas.progress}</span>
            </div>
          </div>
        )}
        {status && <p className="text-xs text-gray-600">{status}</p>}
      </section>

      {policyInfo && (
        <section className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <div className="rounded border border-gray-200 p-4 text-sm">
            <h3 className="mb-2 text-sm font-semibold">Policy</h3>
            <ul className="space-y-1 text-xs">
              <li><strong>Authority:</strong> {policyInfo.authority}</li>
              <li><strong>Pool:</strong> {policyInfo.cpPool}</li>
              <li><strong>Quote Mint:</strong> {policyInfo.quoteMint}</li>
              <li><strong>Y0 Total:</strong> {policyInfo.y0Total}</li>
              <li><strong>Investor Share:</strong> {policyInfo.investorFeeShareBps} bps</li>
              <li><strong>Daily Cap:</strong> {policyInfo.dailyCapQuote}</li>
              <li><strong>Min Payout:</strong> {policyInfo.minPayout}</li>
              <li><strong>Creator ATA:</strong> {policyInfo.creatorQuoteAta}</li>
              <li><strong>Treasury ATA:</strong> {policyInfo.treasuryQuoteAta}</li>
            </ul>
          </div>
          {progressInfo && (
            <div className="rounded border border-gray-200 p-4 text-sm">
              <h3 className="mb-2 text-sm font-semibold">Progress (UTC day)</h3>
              <ul className="space-y-1 text-xs">
                <li><strong>Current Day:</strong> {progressInfo.currentDay}</li>
                <li><strong>Claimed Today:</strong> {progressInfo.claimed}</li>
                <li><strong>Distributed Today:</strong> {progressInfo.distributed}</li>
                <li><strong>Carry:</strong> {progressInfo.carry}</li>
                <li><strong>Page Cursor:</strong> {progressInfo.pageCursor}</li>
                <li><strong>Day Closed:</strong> {progressInfo.dayClosed ? 'Yes' : 'No'}</li>
              </ul>
            </div>
          )}
        </section>
      )}

      {events.length > 0 && (
        <section className="text-sm">
          <h3 className="mb-2 text-sm font-semibold">Recent Events (signatures)</h3>
          <ul className="space-y-2 text-xs">
            {events.map(event => (
              <li key={event.signature} className="rounded border border-gray-200 p-2">
                <div className="flex flex-col md:flex-row md:items-center md:justify-between">
                  <span className="font-mono text-[11px]">{event.signature}</span>
                  <span className="text-gray-500">slot {event.slot}</span>
                </div>
                {event.blockTime && (
                  <span className="text-gray-500">
                    {new Date(event.blockTime * 1000).toLocaleString()}
                  </span>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}
    </main>
  );
}
