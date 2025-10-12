import anchor from '@coral-xyz/anchor';
import type { Idl } from '@coral-xyz/anchor';
const { AnchorProvider, BN, Program } = anchor;
import { expect } from 'chai';
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo
} from '@solana/spl-token';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

describe('keystone_fee_router happy path', () => {
  const provider = AnchorProvider.local();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  const payer = wallet.payer as Keypair;

  const DLMM_PROGRAM_ID = new PublicKey('cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG');
  const POLICY_SEED = Buffer.from('policy');
  const PROGRESS_SEED = Buffer.from('progress');
  const VAULT_SEED = Buffer.from('vault');
  const POSITION_SEED = Buffer.from('position');
  const FEE_POS_OWNER_SEED = Buffer.from('investor_fee_pos_owner');

  let program: Program<Idl>;
  let policyPda: PublicKey;
  let progressPda: PublicKey;
  let vaultAuthority: PublicKey;
  let ownerPda: PublicKey;
  let positionPda: PublicKey;
  let quoteMint: PublicKey;
  let treasuryAta: PublicKey;
  let creatorAta: PublicKey;
  let investorAta: PublicKey;
  let cpPool: Keypair;
  let cpProgram: Keypair;
  let cpPosition: Keypair;
  let streamAccount: Keypair;
  let investor: Keypair;

  const mintDecimals = 6;
  const initialTreasuryAmount = 500_000n; // 500 quote tokens with 6 decimals

  const idlPath = path.resolve(__dirname, '../../..', 'target/idl/keystone_fee_router.json');

  before(async () => {
    const idl = JSON.parse(fs.readFileSync(idlPath, 'utf8')) as Idl & { metadata?: { address?: string } };
    if (!idl.accounts?.[0]?.type) {
      idl.accounts = [
        {
          name: 'honoraryPosition',
          discriminator: [238, 164, 37, 108, 84, 131, 245, 25],
          type: {
            kind: 'struct',
            fields: [
              { name: 'owner_pda', type: 'pubkey' },
              { name: 'position', type: 'pubkey' },
              { name: 'cp_pool', type: 'pubkey' },
              { name: 'quote_mint', type: 'pubkey' },
              { name: 'bump', type: 'u8' }
            ]
          }
        },
        {
          name: 'policy',
          discriminator: [222, 135, 7, 163, 235, 177, 33, 68],
          type: {
            kind: 'struct',
            fields: [
              { name: 'authority', type: 'pubkey' },
              { name: 'cp_pool', type: 'pubkey' },
              { name: 'quote_mint', type: 'pubkey' },
              { name: 'creator_quote_ata', type: 'pubkey' },
              { name: 'treasury_quote_ata', type: 'pubkey' },
              { name: 'investor_fee_share_bps', type: 'u16' },
              { name: 'y0_total', type: 'u64' },
              { name: 'daily_cap_quote', type: 'u64' },
              { name: 'min_payout_lamports', type: 'u64' },
              { name: 'bump', type: 'u8' },
              { name: 'initialized', type: 'bool' }
            ]
          }
        },
        {
          name: 'progress',
          discriminator: [125, 4, 195, 102, 134, 179, 253, 6],
          type: {
            kind: 'struct',
            fields: [
              { name: 'current_day', type: 'i64' },
              { name: 'last_distribution_ts', type: 'i64' },
              { name: 'claimed_quote_today', type: 'u64' },
              { name: 'distributed_quote_today', type: 'u64' },
              { name: 'carry_quote_today', type: 'u64' },
              { name: 'page_cursor', type: 'u64' },
              { name: 'day_closed', type: 'bool' },
              { name: 'bump', type: 'u8' }
            ]
          }
        }
      ];
      const types = idl.types ?? [];
      const upsertType = (name: string, type: any) => {
        const existing = types.find((t: any) => t.name === name);
        if (existing) {
          existing.type = type;
        } else {
          types.push({ name, type });
        }
      };
      upsertType('honoraryPosition', idl.accounts[0].type);
      upsertType('policy', idl.accounts[1].type);
      upsertType('progress', idl.accounts[2].type);
      idl.types = types;
    }
    const camelIdl = anchor.utils.idl.convertIdlToCamelCase(idl as anchor.Idl);
    const programId = new PublicKey(camelIdl.metadata?.address ?? (camelIdl as any).address ?? (idl as any).address);
    const coder = new anchor.BorshCoder(camelIdl as anchor.Idl);
    program = new Program(camelIdl, provider, coder);

    quoteMint = await createMint(connection, payer, payer.publicKey, null, mintDecimals);
    creatorAta = (await getOrCreateAssociatedTokenAccount(connection, payer, quoteMint, payer.publicKey)).address;

    cpPool = Keypair.generate();
    cpProgram = Keypair.generate();
    cpPosition = Keypair.generate();
    streamAccount = Keypair.generate();
    investor = Keypair.generate();

    await createOwnedAccount(cpPool, 0, DLMM_PROGRAM_ID);
    await createOwnedAccount(cpProgram, 0, SystemProgram.programId);
    await createOwnedAccount(cpPosition, 0, cpProgram.publicKey);
    await createOwnedAccount(streamAccount, 8, program.programId);

    policyPda = PublicKey.findProgramAddressSync([POLICY_SEED, cpPool.publicKey.toBuffer()], program.programId)[0];
    positionPda = PublicKey.findProgramAddressSync([POSITION_SEED, policyPda.toBuffer()], program.programId)[0];
    vaultAuthority = PublicKey.findProgramAddressSync([VAULT_SEED, policyPda.toBuffer()], program.programId)[0];
    ownerPda = PublicKey.findProgramAddressSync(
      [VAULT_SEED, policyPda.toBuffer(), FEE_POS_OWNER_SEED],
      program.programId
    )[0];
    progressPda = PublicKey.findProgramAddressSync([
      PROGRESS_SEED,
      cpPool.publicKey.toBuffer()
    ], program.programId)[0];

    treasuryAta = (
      await getOrCreateAssociatedTokenAccount(connection, payer, quoteMint, vaultAuthority, true)
    ).address;
    investorAta = (
      await getOrCreateAssociatedTokenAccount(connection, payer, quoteMint, investor.publicKey)
    ).address;
  });

  async function createOwnedAccount(account: Keypair, space: number, owner: PublicKey) {
    const rent = await connection.getMinimumBalanceForRentExemption(space);
    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: account.publicKey,
        space,
        lamports: rent,
        programId: owner
      })
    );
    await provider.sendAndConfirm(tx, [payer, account]);
  }

  it('initializes policy and honorary position, then distributes fees', async () => {
    const y0Total = new BN(1_000_000);
    const cap = new BN(0);
    const minPayout = new BN(0);

    await program.methods
      .initPolicy({
        y0Total,
        investorFeeShareBps: 2_000,
        dailyCapQuote: cap,
        minPayoutLamports: minPayout
      })
      .accounts({
        authority: payer.publicKey,
        policy: policyPda,
        cpPool: cpPool.publicKey,
        quoteMint,
        creatorQuoteAta: creatorAta,
        treasuryQuoteAta: treasuryAta,
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      })
      .rpc();

    // Reassign pool ownership to the mock cp_program so collect_quote_fees check passes.
    const assignIx = SystemProgram.assign({
      accountPubkey: cpPool.publicKey,
      programId: cpProgram.publicKey
    });
    await provider.sendAndConfirm(new Transaction().add(assignIx), [payer, cpPool]);

    await program.methods
      .initHonoraryPosition()
      .accounts({
        authority: payer.publicKey,
        policy: policyPda,
        cpPool: cpPool.publicKey,
        quoteMint,
        ownerPda,
        cpPosition: cpPosition.publicKey,
        honoraryPosition: positionPda,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      })
      .rpc();

    await mintTo(connection, payer, quoteMint, treasuryAta, payer.publicKey, Number(initialTreasuryAmount));

    const investorEventLog: unknown[] = [];
    const creatorEventLog: unknown[] = [];
    const investorListener = await program.addEventListener('InvestorPayoutPage', (event) => {
      investorEventLog.push(event);
    });
    const creatorListener = await program.addEventListener('CreatorPayoutDayClosed', (event) => {
      creatorEventLog.push(event);
    });

    await program.methods
      .crankDistribute({ pageCursor: new BN(1), isLastPage: false })
      .accounts({
        cpProgram: cpProgram.publicKey,
        cpPool: cpPool.publicKey,
        policy: policyPda,
        progress: progressPda,
        payer: payer.publicKey,
        vaultAuthority,
        treasuryQuoteAta: treasuryAta,
        creatorQuoteAta: creatorAta,
        investorQuoteAta: investorAta,
        stream: streamAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([cpProgram])
      .rpc();

    const progressAfterFirst = await program.account.progress.fetch(progressPda);
    expect(progressAfterFirst.carryQuoteToday.toNumber()).to.equal(Number(initialTreasuryAmount));
    expect(progressAfterFirst.distributedQuoteToday.toNumber()).to.equal(0);
    expect(progressAfterFirst.dayClosed).to.be.false;

    const creatorBefore = await connection.getTokenAccountBalance(creatorAta);

    await program.methods
      .crankDistribute({ pageCursor: new BN(2), isLastPage: true })
      .accounts({
        cpProgram: cpProgram.publicKey,
        cpPool: cpPool.publicKey,
        policy: policyPda,
        progress: progressPda,
        payer: payer.publicKey,
        vaultAuthority,
        treasuryQuoteAta: treasuryAta,
        creatorQuoteAta: creatorAta,
        investorQuoteAta: investorAta,
        stream: streamAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([cpProgram])
      .rpc();

    const progressAfterSecond = await program.account.progress.fetch(progressPda);
    expect(progressAfterSecond.carryQuoteToday.toNumber()).to.equal(0);
    expect(progressAfterSecond.dayClosed).to.be.true;

    const creatorAfter = await connection.getTokenAccountBalance(creatorAta);
    const delta = BigInt(creatorAfter.value.amount) - BigInt(creatorBefore.value.amount);
    expect(delta).to.equal(initialTreasuryAmount);
    expect(investorEventLog.length).to.be.greaterThan(0);
    expect(creatorEventLog.length).to.be.greaterThan(0);

    await program.removeEventListener(investorListener);
    await program.removeEventListener(creatorListener);
  });

  it('rejects pools without Meteora ownership', async () => {
    const badPool = Keypair.generate();
    await createOwnedAccount(badPool, 0, SystemProgram.programId);
    const badPolicy = PublicKey.findProgramAddressSync([POLICY_SEED, badPool.publicKey.toBuffer()], program.programId)[0];
    const badVault = PublicKey.findProgramAddressSync([VAULT_SEED, badPolicy.toBuffer()], program.programId)[0];
    const badTreasury = (
      await getOrCreateAssociatedTokenAccount(connection, payer, quoteMint, badVault, true)
    ).address;
    const badCreatorAta = (
      await getOrCreateAssociatedTokenAccount(connection, payer, quoteMint, payer.publicKey)
    ).address;

    try {
      await program.methods
        .initPolicy({
          y0Total: new BN(1_000_000),
          investorFeeShareBps: 2_000,
          dailyCapQuote: new BN(0),
          minPayoutLamports: new BN(0)
        })
        .accounts({
          authority: payer.publicKey,
          policy: badPolicy,
          cpPool: badPool.publicKey,
          quoteMint,
          creatorQuoteAta: badCreatorAta,
          treasuryQuoteAta: badTreasury,
          vaultAuthority: badVault,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        })
        .rpc();
      expect.fail('expected QuoteOnlyViolation');
    } catch (err: any) {
      expect(err.error.errorCode.code).to.equal('QuoteOnlyViolation');
    }
  });

  it('rolls state forward to new day after close', async () => {
    const currentProgress = await program.account.progress.fetch(progressPda);
    const currentDay = currentProgress.currentDay.toNumber();

    await mintTo(connection, payer, quoteMint, treasuryAta, payer.publicKey, 200_000);

    const slot = await connection.getSlot();
    await connection.rpcRequest('warpSlot', [slot + 200_000]);

    await program.methods
      .crankDistribute({ pageCursor: new BN(3), isLastPage: true })
      .accounts({
        cpProgram: cpProgram.publicKey,
        cpPool: cpPool.publicKey,
        policy: policyPda,
        progress: progressPda,
        payer: payer.publicKey,
        vaultAuthority,
        treasuryQuoteAta: treasuryAta,
        creatorQuoteAta: creatorAta,
        investorQuoteAta: investorAta,
        stream: streamAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([cpProgram])
      .rpc();

    const rolled = await program.account.progress.fetch(progressPda);
    expect(rolled.currentDay.toNumber()).to.be.greaterThan(currentDay);
    expect(rolled.dayClosed).to.be.true;
  });
});
