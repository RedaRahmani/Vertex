import { AnchorProvider, BN, Idl, Program } from '@coral-xyz/anchor';
import {
  AccountMeta,
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
  TransactionInstruction
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

export const METEORA_DLMM_V2 = new PublicKey('cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG');

export type FeeRouterProgram = Program<Idl>;

const POLICY_SEED = Buffer.from('policy');
const PROGRESS_SEED = Buffer.from('progress');
const VAULT_SEED = Buffer.from('vault');
const POSITION_SEED = Buffer.from('position');
const FEE_POS_OWNER_SEED = Buffer.from('investor_fee_pos_owner');

type BNLike = BN | bigint | number;

const toBN = (value: BNLike): BN => (value instanceof BN ? value : new BN(value));

export function getProgram(provider: AnchorProvider, programId: PublicKey, idl: Idl): FeeRouterProgram {
  const patchedIdl: Idl = { ...idl, address: programId.toBase58() };
  return new Program(patchedIdl, provider);
}

export function findPolicyPda(cpPool: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([POLICY_SEED, cpPool.toBuffer()], programId);
}

export function findProgressPda(cpPool: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([PROGRESS_SEED, cpPool.toBuffer()], programId);
}

export function findVaultAuthorityPda(policy: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([VAULT_SEED, policy.toBuffer()], programId);
}

export function findPositionPda(policy: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([POSITION_SEED, policy.toBuffer()], programId);
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
  tokenProgram?: PublicKey;
  systemProgram?: PublicKey;
  rentSysvar?: PublicKey;
}

export async function buildInitPolicyIx(p: InitPolicyParams): Promise<TransactionInstruction> {
  const program = getProgram(p.provider, p.programId, p.idl);
  const [policy] = findPolicyPda(p.cpPool, p.programId);
  const [vaultAuthority] = findVaultAuthorityPda(policy, p.programId);
  const tokenProgram = p.tokenProgram ?? TOKEN_PROGRAM_ID;
  const systemProgram = p.systemProgram ?? SystemProgram.programId;
  const rent = p.rentSysvar ?? SYSVAR_RENT_PUBKEY;

  return program.methods
    .initPolicy({
      y0Total: toBN(p.y0Total),
      investorFeeShareBps: p.investorFeeShareBps,
      dailyCapQuote: toBN(p.dailyCapQuote),
      minPayoutLamports: toBN(p.minPayoutLamports)
    })
    .accounts({
      authority: p.authority,
      policy,
      cpPool: p.cpPool,
      quoteMint: p.quoteMint,
      creatorQuoteAta: p.creatorQuoteAta,
      treasuryQuoteAta: p.treasuryQuoteAta,
      vaultAuthority,
      tokenProgram,
      systemProgram,
      rent
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
  systemProgram?: PublicKey;
  rentSysvar?: PublicKey;
}

export async function buildInitHonoraryPositionIx(p: InitHonoraryPositionParams): Promise<TransactionInstruction> {
  const program = getProgram(p.provider, p.programId, p.idl);
  const [ownerPda] = PublicKey.findProgramAddressSync(
    [VAULT_SEED, p.policy.toBuffer(), FEE_POS_OWNER_SEED],
    p.programId
  );
  const [honoraryPosition] = findPositionPda(p.policy, p.programId);
  const systemProgram = p.systemProgram ?? SystemProgram.programId;
  const rent = p.rentSysvar ?? SYSVAR_RENT_PUBKEY;

  return program.methods
    .initHonoraryPosition()
    .accounts({
      authority: p.authority,
      policy: p.policy,
      cpPool: p.cpPool,
      quoteMint: p.quoteMint,
      ownerPda,
      cpPosition: p.cpPosition,
      honoraryPosition,
      systemProgram,
      rent
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
  pageCursor: BNLike;
  isLastPage: boolean;
  cpProgram?: PublicKey;
  tokenProgram?: PublicKey;
  systemProgram?: PublicKey;
  additionalInvestors?: Array<{ investorQuoteAta: PublicKey; stream: PublicKey }>;
}

export async function buildCrankDistributeIx(p: CrankDistributeParams): Promise<TransactionInstruction> {
  const program = getProgram(p.provider, p.programId, p.idl);
  const [vaultAuthority] = findVaultAuthorityPda(p.policy, p.programId);
  const [progress] = findProgressPda(p.cpPool, p.programId);
  const cpProgram = p.cpProgram ?? METEORA_DLMM_V2;
  const tokenProgram = p.tokenProgram ?? TOKEN_PROGRAM_ID;
  const systemProgram = p.systemProgram ?? SystemProgram.programId;

  let builder = program.methods
    .crankDistribute({ pageCursor: toBN(p.pageCursor), isLastPage: p.isLastPage })
    .accounts({
      cpProgram,
      cpPool: p.cpPool,
      policy: p.policy,
      progress,
      payer: p.payer,
      vaultAuthority,
      treasuryQuoteAta: p.treasuryQuoteAta,
      creatorQuoteAta: p.creatorQuoteAta,
      investorQuoteAta: p.investorQuoteAta,
      stream: p.stream,
      tokenProgram,
      systemProgram
    });

  const remaining: AccountMeta[] = [];
  for (const entry of p.additionalInvestors ?? []) {
    remaining.push({ pubkey: entry.investorQuoteAta, isSigner: false, isWritable: true });
    remaining.push({ pubkey: entry.stream, isSigner: false, isWritable: false });
  }

  if (remaining.length > 0) {
    builder = builder.remainingAccounts(remaining);
  }

  return builder.instruction();
}
