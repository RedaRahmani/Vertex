import { AnchorProvider, Idl } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Keypair, PublicKey, TransactionInstruction } from '@solana/web3.js';
import { describe, expect, it } from 'vitest';

import { buildLaunchpadBuyInstruction } from '../index.js';

const TEST_IDL: Idl = {
  version: '0.1.0',
  name: 'keystone_launchpad',
  instructions: [
    {
      name: 'buy',
      accounts: [
        { name: 'buyer', isMut: true, isSigner: true },
        { name: 'quoteAccount', isMut: true, isSigner: false },
        { name: 'buyerReceipt', isMut: true, isSigner: false },
        { name: 'launchConfig', isMut: true, isSigner: false },
        { name: 'treasuryAuthority', isMut: false, isSigner: false },
        { name: 'treasuryVault', isMut: true, isSigner: false },
        { name: 'saleState', isMut: true, isSigner: false },
        { name: 'mint', isMut: false, isSigner: false },
        { name: 'tokenProgram', isMut: false, isSigner: false }
      ],
      args: [
        { name: 'amount', type: 'u64' },
        { name: 'proof', type: { option: { vec: { array: ['u8', 32] } } } },
        { name: 'maxQuote', type: 'u64' }
      ]
    }
  ],
  accounts: []
};

describe('launchpad builder', () => {
  it('creates instruction structure', async () => {
    const provider = new AnchorProvider({} as any, {} as any, {} as any);
    const rand = () => Keypair.generate().publicKey;
    const accounts = {
      buyer: rand(),
      quoteAccount: rand(),
      buyerReceipt: rand(),
      launchConfig: rand(),
      treasuryVault: rand(),
      saleState: rand(),
      mint: rand(),
      tokenProgram: TOKEN_PROGRAM_ID
    };

    const ix = await buildLaunchpadBuyInstruction({
      provider,
      programId: rand(),
      idl: TEST_IDL,
      amount: BigInt(1_000_000_000),
      maxQuote: BigInt(1_500_000_000),
      proof: null,
      accounts
    });

    expect(ix).toBeInstanceOf(TransactionInstruction);
    expect(ix.keys).toHaveLength(TEST_IDL.instructions[0].accounts?.length ?? 0);
  });
});
