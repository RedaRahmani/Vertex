import { AnchorProvider, Idl, Program } from '@coral-xyz/anchor';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';

export interface LaunchpadProgramAccounts {
  buyer: PublicKey;
  quoteAccount: PublicKey;
  buyerReceipt: PublicKey;
  launchConfig: PublicKey;
  treasuryVault: PublicKey;
  saleState: PublicKey;
  mint: PublicKey;
  tokenProgram: PublicKey;
}

export interface LaunchpadBuyParams {
  provider: AnchorProvider;
  programId: PublicKey;
  idl: Idl;
  amount: bigint;
  maxQuote: bigint;
  proof?: Array<Uint8Array> | null;
  accounts: LaunchpadProgramAccounts;
}

/** Returns an Anchor program wrapper using the supplied IDL. */
export function getProgram(provider: AnchorProvider, programId: PublicKey, idl: Idl): Program<Idl> {
  return new Program(idl, programId, provider);
}

/** Builds a Launchpad buy instruction using IDL metadata. */
export async function buildLaunchpadBuyInstruction(params: LaunchpadBuyParams): Promise<TransactionInstruction> {
  const program = getProgram(params.provider, params.programId, params.idl);
  const treasuryAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from('treasury'), params.accounts.launchConfig.toBuffer()],
    params.programId
  )[0];

  return program.methods
    .buy(params.amount, params.proof ?? null, params.maxQuote)
    .accounts({
      buyer: params.accounts.buyer,
      quoteAccount: params.accounts.quoteAccount,
      buyerReceipt: params.accounts.buyerReceipt,
      launchConfig: params.accounts.launchConfig,
      treasuryAuthority,
      treasuryVault: params.accounts.treasuryVault,
      saleState: params.accounts.saleState,
      mint: params.accounts.mint,
      tokenProgram: params.accounts.tokenProgram
    })
    .instruction();
}

export * as FeeRouter from './feeRouter';
