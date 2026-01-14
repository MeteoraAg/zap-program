import { BN } from "@coral-xyz/anchor";
import { LiteSVM } from "litesvm";
import {
  AccountMeta,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import { DAMM_V2_PROGRAM_ID } from "../damm_v2";
import {
  deriveDammV2EventAuthority,
  deriveDammV2PoolAuthority,
  deriveDlmmEventAuthority,
  deriveLedgerAccount,
  getDammV2Pool,
} from "../pda";
import { createZapProgram } from "./zapOut";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import {
  DLMM_PROGRAM_ID_LOCAL,
  getLbPairState,
  MEMO_PROGRAM_ID,
} from "../dlmm";

export async function zapInDammv2(params: {
  svm: LiteSVM;
  user: PublicKey;
  pool: PublicKey;
  position: PublicKey;
  positionNftAccount: PublicKey;
  preSqrtPrice: BN;
  maxSqrtPriceChangeBps: number;
}): Promise<Transaction> {
  const zapProgram = createZapProgram();

  const {
    svm,
    user,
    pool,
    position,
    positionNftAccount,
    preSqrtPrice,
    maxSqrtPriceChangeBps,
  } = params;

  const poolState = getDammV2Pool(svm, pool);
  const { tokenAVault, tokenBVault, tokenAMint, tokenBMint } = poolState;

  const tokenAProgram = svm.getAccount(poolState.tokenAMint).owner;

  const tokenBProgram = svm.getAccount(poolState.tokenBMint).owner;

  const tokenAAccount = getAssociatedTokenAddressSync(
    tokenAMint,
    user,
    true,
    tokenAProgram
  );

  const tokenBAccount = getAssociatedTokenAddressSync(
    tokenBMint,
    user,
    true,
    tokenBProgram
  );

  return await zapProgram.methods
    .zapInDammV2(preSqrtPrice, maxSqrtPriceChangeBps)
    .accountsPartial({
      ledger: deriveLedgerAccount(user),
      pool,
      poolAuthority: deriveDammV2PoolAuthority(),
      position,
      tokenAAccount,
      tokenBAccount,
      tokenAVault,
      tokenBVault,
      tokenAMint,
      tokenBMint,
      positionNftAccount,
      owner: user,
      tokenAProgram,
      tokenBProgram,
      dammProgram: DAMM_V2_PROGRAM_ID,
      dammEventAuthority: deriveDammV2EventAuthority(),
    })
    .remainingAccounts([
      {
        isSigner: false,
        isWritable: false,
        pubkey: SYSVAR_INSTRUCTIONS_PUBKEY,
      },
    ])
    .transaction();
}

export async function zapInDlmmforInitializedPosition(params: {
  svm: LiteSVM;
  owner: PublicKey;
  lbPair: PublicKey;
  position: PublicKey;
  activeId: number;
  minDeltaId: number;
  maxDeltaId: number;
  maxActiveBinSlippage: number;
  favorXInActiveId: boolean;
  strategy: any;
  remainingAccountInfo: any;
  binArrays: AccountMeta[];
  binArrayBitmapExtension: PublicKey;
}): Promise<Transaction> {
  const program = createZapProgram();

  const {
    svm,
    owner,
    lbPair,
    position,
    activeId,
    minDeltaId,
    maxActiveBinSlippage,
    maxDeltaId,
    favorXInActiveId,
    strategy,
    remainingAccountInfo,
    binArrayBitmapExtension,
    binArrays,
  } = params;

  const lbPairState = getLbPairState(svm, lbPair);
  const { tokenXMint, tokenYMint, reserveX, reserveY } = lbPairState;

  const tokenXProgram = svm.getAccount(tokenXMint).owner;
  const tokenYProgram = svm.getAccount(tokenYMint).owner;

  const userTokenX = getAssociatedTokenAddressSync(
    tokenXMint,
    owner,
    true,
    tokenXProgram
  );

  const userTokenY = getAssociatedTokenAddressSync(
    tokenYMint,
    owner,
    true,
    tokenYProgram
  );

  let binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);

  return await program.methods
    .zapInDlmmForInitializedPosition(
      activeId,
      minDeltaId,
      maxDeltaId,
      maxActiveBinSlippage,
      favorXInActiveId,
      strategy,
      remainingAccountInfo
    )
    .accountsPartial({
      ledger: deriveLedgerAccount(owner),
      lbPair,
      position,
      binArrayBitmapExtension: binArrayBitmapExtensionState
        ? binArrayBitmapExtension
        : null,
      userTokenX,
      userTokenY,
      reserveX,
      reserveY,
      tokenXMint,
      tokenYMint,
      tokenXProgram,
      tokenYProgram,
      dlmmProgram: DLMM_PROGRAM_ID_LOCAL,
      owner,
      rentPayer: owner,
      memoProgram: MEMO_PROGRAM_ID,
      dlmmEventAuthority: deriveDlmmEventAuthority(),
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts(binArrays)
    .transaction();
}

export async function zapInDlmmforUnInitializedPosition(params: {
  svm: LiteSVM;
  owner: PublicKey;
  lbPair: PublicKey;
  position: PublicKey;
  activeId: number;
  minDeltaId: number;
  maxDeltaId: number;
  maxActiveBinSlippage: number;
  favorXInActiveId: boolean;
  strategy: any;
  remainingAccountInfo: any;
  binArrays: AccountMeta[];
  binArrayBitmapExtension: PublicKey;
}): Promise<Transaction> {
  const program = createZapProgram();

  const {
    svm,
    owner,
    lbPair,
    position,
    activeId,
    minDeltaId,
    maxDeltaId,
    maxActiveBinSlippage,
    favorXInActiveId,
    strategy,
    remainingAccountInfo,
    binArrayBitmapExtension,
    binArrays,
  } = params;

  const lbPairState = getLbPairState(svm, lbPair);
  const { tokenXMint, tokenYMint, reserveX, reserveY } = lbPairState;

  const tokenXProgram = svm.getAccount(tokenXMint).owner;
  const tokenYProgram = svm.getAccount(tokenYMint).owner;

  const userTokenX = getAssociatedTokenAddressSync(
    tokenXMint,
    owner,
    true,
    tokenXProgram
  );

  const userTokenY = getAssociatedTokenAddressSync(
    tokenYMint,
    owner,
    true,
    tokenYProgram
  );

  let binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);

  return await program.methods
    .zapInDlmmForUninitializedPosition(
      minDeltaId,
      maxDeltaId,
      activeId,
      maxActiveBinSlippage,
      favorXInActiveId,
      strategy,
      remainingAccountInfo
    )
    .accountsPartial({
      ledger: deriveLedgerAccount(owner),
      lbPair,
      position,
      binArrayBitmapExtension: binArrayBitmapExtensionState
        ? binArrayBitmapExtension
        : null,
      userTokenX,
      userTokenY,
      reserveX,
      reserveY,
      tokenXMint,
      tokenYMint,
      tokenXProgram,
      tokenYProgram,
      dlmmProgram: DLMM_PROGRAM_ID_LOCAL,
      owner,
      rentPayer: owner,
      memoProgram: MEMO_PROGRAM_ID,
      dlmmEventAuthority: deriveDlmmEventAuthority(),
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts(binArrays)
    .transaction();
}
