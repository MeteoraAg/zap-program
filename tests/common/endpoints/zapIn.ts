import { BN } from "@coral-xyz/anchor";
import { LiteSVM } from "litesvm";
import { PublicKey, Transaction } from "@solana/web3.js";
import { DAMM_V2_PROGRAM_ID } from "../damm_v2";
import {
  deriveDammV2EventAuthority,
  deriveDammV2PoolAuthority,
  deriveLedgerAccount,
  getDammV2Pool,
} from "../pda";
import { createZapProgram } from "./zapOut";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

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
    .transaction();
}
