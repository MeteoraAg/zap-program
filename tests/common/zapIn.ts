import { PublicKey, Transaction } from "@solana/web3.js";
import { LiteSVM } from "litesvm";
import { createZapProgram } from "./zapOut";
import { deriveDammV2EventAuthority, getDammV2Pool } from "./pda";
import { DAMM_V2_PROGRAM_ID } from "./damm_v2";
import {
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";
import { getTokenBalance } from "./utils";

export async function zapInDammV2(
  svm: LiteSVM,
  user: PublicKey,
  pool: PublicKey,
  position: PublicKey,
  positionNftAccount: PublicKey,
  thesholdAmountA: BN,
  thresholdAmountB: BN,
  maxDepositAmountA: BN,
  maxDepositAmountB: BN
): Promise<Transaction> {
  const zapProgram = createZapProgram();

  const poolState = getDammV2Pool(svm, pool);
  const tokenAProgram =
    poolState.tokenAFlag == 0 ? TOKEN_PROGRAM_ID : TOKEN_2022_PROGRAM_ID;
  const tokenBProgram =
    poolState.tokenAFlag == 0 ? TOKEN_PROGRAM_ID : TOKEN_2022_PROGRAM_ID;

  const userTokenAAccount = getAssociatedTokenAddressSync(
    poolState.tokenAMint,
    user,
    true,
    tokenAProgram
  );
  const userTokenBAccount = getAssociatedTokenAddressSync(
    poolState.tokenBMint,
    user,
    true,
    tokenBProgram
  );

  const preTokenABalance = getTokenBalance(svm, userTokenAAccount);
  const preTokenBBalance = getTokenBalance(svm, userTokenBAccount);
  return await zapProgram.methods
    .zapIn({
      preTokenABalance,
      preTokenBBalance,
      thesholdAmountA,
      thresholdAmountB,
      maxDepositAmountA,
      maxDepositAmountB,
    })
    .accountsPartial({
      pool,
      position,
      positionNftAccount,
      tokenAMint: poolState.tokenAMint,
      tokenBMint: poolState.tokenBMint,
      tokenAVault: poolState.tokenAVault,
      tokenBVault: poolState.tokenBVault,
      userTokenAAccount,
      userTokenBAccount,
      owner: user,
      ammProgram: DAMM_V2_PROGRAM_ID,
      dammEventAuthority: deriveDammV2EventAuthority(),
      tokenAProgram,
      tokenBProgram,
    })
    .transaction();
}
