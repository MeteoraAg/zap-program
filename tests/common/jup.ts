import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { LiteSVM } from "litesvm";
import { Jupiter } from "./idl/jup_v6";
import JupIDL from "../../idls/jup_v6.json";
import { IdlTypes } from "@coral-xyz/anchor";
import { deriveZapAuthorityAddress } from "./zap";
import { DAMM_V2_PROGRAM_ID } from "./damm_v2";
import { deriveDammV2EventAuthority, deriveDammV2PoolAuthority, getDammV2Pool } from "./pda";

export type RoutePlanStep = IdlTypes<Jupiter>["routePlanStep"];

export const JUP_V6_PROGRAM_ID = new PublicKey(JupIDL.address);

export function deriveJupV6EventAuthority() {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    JUP_V6_PROGRAM_ID
  )[0];
}

// https://explorer.solana.com/tx/4r5gcvi3j2RoPedr1zYxUmLRfMt29U9FNucCfGkxoYSC5sxnv6U5nuYNzVqjJpV4RCZb9qBrMzp2A3dhN4NHH6G9
export function getJupRemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  inputTokenAccount: PublicKey,
  outputTokenAccount: PublicKey,
  outputMint: PublicKey,
  tokenAProgram = TOKEN_PROGRAM_ID,
  tokenBProgram = TOKEN_PROGRAM_ID
): Array<{
  isSigner: boolean;
  isWritable: boolean;
  pubkey: PublicKey;
}> {
  const poolState = getDammV2Pool(svm, pool);
  const accounts: Array<{
    isSigner: boolean;
    isWritable: boolean;
    pubkey: PublicKey;
  }> = [
    {
      isSigner: false,
      isWritable: false,
      pubkey: TOKEN_PROGRAM_ID,
    },
    {
      pubkey: deriveZapAuthorityAddress(),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: inputTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: outputTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: JUP_V6_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: JUP_V6_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveJupV6EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: JUP_V6_PROGRAM_ID,
    },
    // swap pool account
    {
      pubkey: DAMM_V2_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: deriveDammV2PoolAuthority(),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: pool,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: inputTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: outputTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: poolState.tokenAVault,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: poolState.tokenBVault,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: poolState.tokenAMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: poolState.tokenBMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: deriveZapAuthorityAddress(),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: tokenAProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: tokenBProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: DAMM_V2_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveDammV2EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: DAMM_V2_PROGRAM_ID,
    },
  ];
  return accounts;
}
