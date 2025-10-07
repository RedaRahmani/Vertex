import { AnchorProvider, BN, Idl, Program } from '@coral-xyz/anchor';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';

export const METEORA_DLMM_V2 = new PublicKey('cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG');

export type FeeRouterProgram = Program<Idl>;

export function getProgram(provider: AnchorProvider, programId: PublicKey, idl: Idl): FeeRouterProgram {
  return new Program(idl, programId, provider);
}

export function findPolicyPda(programId: PublicKey, cpPool: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('policy'), cpPool.toBuffer()], programId);
}

export function findProgressPda(programId: PublicKey, cpPool: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('progress'), cpPool.toBuffer()], programId);
}

export function findVaultAuthorityPda(programId: PublicKey, policy: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('vault'), policy.toBuffer()], programId);
}

export function findPositionPda(programId: PublicKey, policy: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('position'), policy.toBuffer()], programId);
}

export interface InitPolicyParams {
  provider: AnchorProvider;
  programId: PublicKey;
  idl: Idl;
  cpPool: PublicKey;
  quoteMint: PublicKey;
  creatorQuoteAta: PublicKey;
  treasuryQuoteAta: PublicKey;
  y0Total: bigint | BN | number;
  investorFeeShareBps: number;
  dailyCapQuote: bigint | BN | number;
  minPayoutLamports: bigint | BN | number;
  authority: PublicKey;
  tokenProgram: PublicKey;
}

export async function buildInitPolicyIx(p: InitPolicyParams): Promise<TransactionInstruction> {
  const program = getProgram(p.provider, p.programId, p.idl);
  const [policy] = findPolicyPda(p.programId, p.cpPool);
  const [vaultAuthority] = findVaultAuthorityPda(p.programId, policy);
  return program.methods
    .initPolicy({
      y0Total: new BN(p.y0Total as any),
      investorFeeShareBps: p.investorFeeShareBps,
      dailyCapQuote: new BN(p.dailyCapQuote as any),
      minPayoutLamports: new BN(p.minPayoutLamports as any)
    })
    .accounts({
      authority: p.authority,
      policy,
      cpPool: p.cpPool,
      quoteMint: p.quoteMint,
      creatorQuoteAta: p.creatorQuoteAta,
      treasuryQuoteAta: p.treasuryQuoteAta,
      vaultAuthority,
      tokenProgram: p.tokenProgram
    })
    .instruction();
}

export interface InitHonoraryPositionParams {
  provider: AnchorProvider;
  programId: PublicKey;
  idl: Idl;
  policy: PublicKey;
  cpPool: PublicKey;
  quoteMint: PublicKey;
  cpPosition: PublicKey;
  authority: PublicKey;
}

export async function buildInitHonoraryPositionIx(p: InitHonoraryPositionParams): Promise<TransactionInstruction> {
  const program = getProgram(p.provider, p.programId, p.idl);
  const [ownerPda] = PublicKey.findProgramAddressSync([
    Buffer.from('vault'),
    p.policy.toBuffer(),
    Buffer.from('investor_fee_pos_owner')
  ], p.programId);
  const [honoraryPosition] = findPositionPda(p.programId, p.policy);
  return program.methods
    .initHonoraryPosition()
    .accounts({
      authority: p.authority,
      policy: p.policy,
      cpPool: p.cpPool,
      quoteMint: p.quoteMint,
      ownerPda,
      cpPosition: p.cpPosition,
      honoraryPosition
    })
    .instruction();
}

export interface CrankDistributeParams {
  provider: AnchorProvider;
  programId: PublicKey;
  idl: Idl;
  policy: PublicKey;
  cpPool: PublicKey;
  payer: PublicKey;
  treasuryQuoteAta: PublicKey;
  creatorQuoteAta: PublicKey;
  investorQuoteAta: PublicKey;
  stream: PublicKey;
  pageCursor: number | bigint | BN;
  isLastPage: boolean;
}

export async function buildCrankDistributeIx(p: CrankDistributeParams): Promise<TransactionInstruction> {
  const program = getProgram(p.provider, p.programId, p.idl);
  const [vaultAuthority] = findVaultAuthorityPda(p.programId, p.policy);
  const [progress] = findProgressPda(p.programId, p.cpPool);
  return program.methods
    .crankDistribute({ pageCursor: new BN(p.pageCursor as any), isLastPage: p.isLastPage })
    .accounts({
      cpProgram: METEORA_DLMM_V2,
      policy: p.policy,
      progress,
      payer: p.payer,
      vaultAuthority,
      treasuryQuoteAta: p.treasuryQuoteAta,
      creatorQuoteAta: p.creatorQuoteAta,
      investorQuoteAta: p.investorQuoteAta,
      stream: p.stream
    })
    .instruction();
}

