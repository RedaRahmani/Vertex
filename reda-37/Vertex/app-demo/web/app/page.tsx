'use client';

import { type CSSProperties, useMemo, useState } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';

const containerStyle: CSSProperties = {
  minHeight: '100vh',
  backgroundColor: '#0f172a',
  color: '#f8fafc',
  fontFamily: 'system-ui, sans-serif'
};

const cardStyle: CSSProperties = {
  maxWidth: '720px',
  margin: '0 auto',
  padding: '4rem 1.5rem'
};

const inputStyle: CSSProperties = {
  width: '100%',
  padding: '0.75rem',
  borderRadius: '0.5rem',
  border: '1px solid #1e293b',
  marginTop: '0.5rem',
  backgroundColor: '#1e293b',
  color: 'inherit'
};

const buttonStyle: CSSProperties = {
  marginTop: '1.5rem',
  padding: '0.75rem 1.5rem',
  borderRadius: '0.75rem',
  border: 'none',
  backgroundColor: '#6366f1',
  color: '#ffffff',
  fontWeight: 600,
  cursor: 'pointer'
};

export default function Home() {
  const [programId, setProgramId] = useState('KeystoneLaunchpad11111111111111111111111111111');
  const [status, setStatus] = useState<string>('Idle');

  const connection = useMemo(() => new Connection(process.env.NEXT_PUBLIC_RPC_ENDPOINT ?? 'https://api.devnet.solana.com'), []);

  const handlePing = async () => {
    setStatus('Checking on-chain program account...');
    try {
      const info = await connection.getAccountInfo(new PublicKey(programId));
      setStatus(info ? `Program account found with ${info.lamports} lamports` : 'Program account not found');
    } catch (error) {
      console.error(error);
      setStatus('Failed to reach RPC endpoint');
    }
  };

  return (
    <main style={containerStyle}>
      <section style={cardStyle}>
        <h1 style={{ fontSize: '2.25rem', fontWeight: 600 }}>Keystone Vertex Demo</h1>
        <p style={{ marginTop: '1rem', color: '#cbd5f5' }}>
          Quickly validate deployed programs and cluster connectivity. Configure <code>NEXT_PUBLIC_RPC_ENDPOINT</code> for custom
          clusters.
        </p>
        <label style={{ display: 'block', marginTop: '2rem' }}>
          <span style={{ textTransform: 'uppercase', fontSize: '0.75rem', letterSpacing: '0.08em', color: '#94a3b8' }}>
            Launchpad Program ID
          </span>
          <input
            style={inputStyle}
            value={programId}
            onChange={(event) => setProgramId(event.target.value)}
          />
        </label>
        <button onClick={handlePing} style={buttonStyle} type="button">
          Ping Program
        </button>
        <p style={{ marginTop: '1rem', color: '#cbd5f5' }}>Status: {status}</p>
      </section>
    </main>
  );
}
