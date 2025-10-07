'use client';

import { Idl } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import React, {
  ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';

type EnvState = {
  rpcUrl: string;
  programId: string;
  idlJson: string;
};

type EnvironmentContextValue = {
  rpcUrl: string;
  programId: string;
  idlJson: string;
  programKey?: PublicKey;
  idl?: Idl;
  setEnvironment: (next: Partial<EnvState>) => void;
};

const STORAGE_KEY = 'fee-router-env';

const EnvironmentContext = createContext<EnvironmentContextValue | undefined>(undefined);

const DEFAULT_STATE: EnvState = {
  rpcUrl: 'http://127.0.0.1:8899',
  programId: '',
  idlJson: '',
};

export function EnvironmentProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<EnvState>(DEFAULT_STATE);

  useEffect(() => {
    const saved = typeof window !== 'undefined' ? window.localStorage.getItem(STORAGE_KEY) : null;
    if (saved) {
      try {
        const parsed = JSON.parse(saved) as EnvState;
        setState({
          rpcUrl: parsed.rpcUrl || DEFAULT_STATE.rpcUrl,
          programId: parsed.programId || '',
          idlJson: parsed.idlJson || '',
        });
      } catch {
        // ignore malformed cache
      }
    }
  }, []);

  const setEnvironment = useCallback((next: Partial<EnvState>) => {
    setState(current => {
      const updated = { ...current, ...next };
      if (typeof window !== 'undefined') {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
      }
      return updated;
    });
  }, []);

  const value = useMemo<EnvironmentContextValue>(() => {
    let programKey: PublicKey | undefined;
    try {
      if (state.programId) {
        programKey = new PublicKey(state.programId);
      }
    } catch {
      programKey = undefined;
    }

    let idl: Idl | undefined;
    try {
      if (state.idlJson) {
        idl = JSON.parse(state.idlJson) as Idl;
      }
    } catch {
      idl = undefined;
    }

    return {
      rpcUrl: state.rpcUrl,
      programId: state.programId,
      idlJson: state.idlJson,
      programKey,
      idl,
      setEnvironment,
    };
  }, [setEnvironment, state.idlJson, state.programId, state.rpcUrl]);

  return <EnvironmentContext.Provider value={value}>{children}</EnvironmentContext.Provider>;
}

export function useEnvironment(): EnvironmentContextValue {
  const ctx = useContext(EnvironmentContext);
  if (!ctx) {
    throw new Error('useEnvironment must be used within EnvironmentProvider');
  }
  return ctx;
}

export function EnvironmentPanel() {
  const { rpcUrl, programId, idlJson, setEnvironment } = useEnvironment();
  const [draftRpc, setDraftRpc] = useState(rpcUrl);
  const [draftProgramId, setDraftProgramId] = useState(programId);
  const [draftIdl, setDraftIdl] = useState(idlJson);

  useEffect(() => {
    setDraftRpc(rpcUrl);
    setDraftProgramId(programId);
    setDraftIdl(idlJson);
  }, [idlJson, programId, rpcUrl]);

  const apply = useCallback(() => {
    setEnvironment({
      rpcUrl: draftRpc,
      programId: draftProgramId.trim(),
      idlJson: draftIdl.trim(),
    });
  }, [draftIdl, draftProgramId, draftRpc, setEnvironment]);

  return (
    <section className="mb-6 rounded border border-gray-200 bg-gray-50 p-4 text-sm">
      <h2 className="mb-3 text-base font-medium">Environment</h2>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
        <div className="flex flex-col space-y-2">
          <label className="font-medium text-gray-700">RPC URL</label>
          <input
            className="rounded border border-gray-300 p-2"
            value={draftRpc}
            onChange={e => setDraftRpc(e.target.value)}
            placeholder="http://127.0.0.1:8899"
          />
          <label className="font-medium text-gray-700">Program ID</label>
          <input
            className="rounded border border-gray-300 p-2"
            value={draftProgramId}
            onChange={e => setDraftProgramId(e.target.value)}
            placeholder="Fee router program id"
          />
          <button
            type="button"
            className="self-start rounded bg-blue-600 px-3 py-1.5 text-white"
            onClick={apply}
          >
            Save
          </button>
        </div>
        <div className="flex flex-col space-y-2 md:col-span-1">
          <label className="font-medium text-gray-700">IDL JSON</label>
          <textarea
            className="h-36 rounded border border-gray-300 p-2 font-mono text-xs"
            value={draftIdl}
            onChange={e => setDraftIdl(e.target.value)}
            placeholder="Paste keystone_fee_router IDL JSON"
          />
          <span className="text-xs text-gray-500">
            Paste the IDL emitted by Anchor so the SDK can derive accounts. The values are cached
            locally in your browser.
          </span>
        </div>
      </div>
    </section>
  );
}
