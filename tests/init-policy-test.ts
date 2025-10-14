// @ts-nocheck
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { KeystoneFeeRouter } from "../target/types/keystone_fee_router";

describe("keystone-fee-router", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.KeystoneFeeRouter as Program<KeystoneFeeRouter>;

  it("Initialize policy", async () => {
    const quoteMint = new PublicKey("ErHVBZy2oSDuDyu7accvoRVecgGNvmhXcsLZgNL1zKMu");
    const cpPool = new PublicKey("3yRXGSx8PA92uoUzdzMaH88YtfUuLWT3djTRx5mogdbu");
    
    // Derive policy PDA
    const [policy] = PublicKey.findProgramAddressSync(
      [Buffer.from("policy"), cpPool.toBuffer()],
      program.programId
    );

    // Derive vault authority
    const [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), policy.toBuffer()],
      program.programId
    );

    // Get creator and treasury ATAs
    const creatorAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet as any,
      quoteMint,
      provider.wallet.publicKey
    );

    const treasuryAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet as any,
      quoteMint,
      vaultAuthority,
      true // allowOwnerOffCurve=true since it's a PDA
    );

    try {
      await program.methods
        .initPolicy({
          y0Total: new anchor.BN(1000000000),
          investorFeeShareBps: 5000,
          dailyCapQuote: new anchor.BN(100000000),
          minPayoutLamports: new anchor.BN(1000),
        })
        .accounts({
          authority: provider.wallet.publicKey,
          policy,
          cpPool,
          quoteMint,
          creatorQuoteAta: creatorAta.address,
          treasuryQuoteAta: treasuryAta.address,
          vaultAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      console.log("Policy initialized successfully!");
      console.log("Policy:", policy.toString());
      console.log("Vault Authority:", vaultAuthority.toString());
      console.log("Treasury ATA:", treasuryAta.address.toString());
    } catch (err) {
      console.error("Error initializing policy:", err);
      throw err;
    }
  });
});
