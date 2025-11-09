import { LiteSVM, TransactionMetadata } from "litesvm";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  Transaction,
} from "@solana/web3.js";
import {
  createZapProgram,
  createToken,
  mintToken,
  ZapProgram,
  initializeLedgerAccount,
  setLedgerBalance,
  updateLedgerBalanceAfterSwap,
  getTokenBalance,
  zapInDammv2,
  closeLedgerAccount,
} from "../common";
import {
  getAmountAFromLiquidityDelta,
  getAmountBFromLiquidityDelta,
  Rounding,
} from "@meteora-ag/cp-amm-sdk";

import ZapIDL from "../../target/idl/zap.json";
import DAMMV2IDL from "../../idls/damm_v2.json";
import {
  createDammV2Pool,
  createDammV2Position,
  swap,
} from "../common/damm_v2";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { BN } from "@coral-xyz/anchor";
import { getDammV2Pool, getDammV2Position } from "../common/pda";
import { expect } from "chai";

describe("Zap In damm V2", () => {
  let zapProgram: ZapProgram;
  let svm: LiteSVM;
  let tokenMint: PublicKey;
  let user: Keypair;
  let admin: Keypair;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
  let pool: PublicKey;
  let position: PublicKey;
  let positionNftAccount: PublicKey;

  beforeEach(async () => {
    zapProgram = createZapProgram();
    svm = new LiteSVM();
    svm.addProgramFromFile(
      new PublicKey(ZapIDL.address),
      "./target/deploy/zap.so"
    );
    svm.addProgramFromFile(
      new PublicKey(DAMMV2IDL.address),
      "./tests/fixtures/damm_v2.so"
    );

    user = Keypair.generate();
    admin = Keypair.generate();
    svm.airdrop(user.publicKey, BigInt(LAMPORTS_PER_SOL));
    svm.airdrop(admin.publicKey, BigInt(LAMPORTS_PER_SOL));

    tokenAMint = createToken(svm, admin, admin.publicKey, null);
    tokenBMint = createToken(svm, admin, admin.publicKey, null);
    mintToken(svm, admin, tokenAMint, admin, admin.publicKey);
    mintToken(svm, admin, tokenBMint, admin, admin.publicKey);

    mintToken(svm, admin, tokenAMint, admin, user.publicKey);
    mintToken(svm, admin, tokenBMint, admin, user.publicKey);

    pool = await createDammV2Pool(svm, admin, tokenAMint, tokenBMint);

    const result = await createDammV2Position(svm, user, pool);
    position = result.position;
    positionNftAccount = result.positionNftAccount;
  });

  it("happy path zap in", async () => {
    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);
    const swapTx = await swap({
      svm,
      user: user.publicKey,
      pool,
      amountIn: amountSwap,
      inputTokenMint: tokenAMint,
      outputTokenMint: tokenBMint,
    });

    const initializeLedgerTx = await initializeLedgerAccount(user.publicKey);

    const setLedgerBalanceTx = await setLedgerBalance(
      user.publicKey,
      amountTokenA.sub(amountSwap),
      true // is token a
    );

    // update balance after swapxww

    const tokenBAccount = getAssociatedTokenAddressSync(
      tokenBMint,
      user.publicKey
    );

    const preBalance = getTokenBalance(svm, tokenBAccount);

    const updateLedgerBalanceAfterSwapTx = await updateLedgerBalanceAfterSwap(
      user.publicKey,
      tokenBAccount,
      preBalance,
      amountSwap,
      false
    );

    // zapin
    let poolState = getDammV2Pool(svm, pool);

    const zapInTx = await zapInDammv2({
      svm,
      user: user.publicKey,
      pool,
      position,
      positionNftAccount,
      preSqrtPrice: poolState.sqrtPrice,
      maxSqrtPriceChangeBps: 10,
    });

    // close ledge
    const closeLedgerTx = await closeLedgerAccount(user.publicKey);

    const finalTx = new Transaction()
      .add(swapTx)
      .add(initializeLedgerTx)
      .add(setLedgerBalanceTx)
      .add(updateLedgerBalanceAfterSwapTx)
      .add(zapInTx)
      .add(closeLedgerTx);

    finalTx.recentBlockhash = svm.latestBlockhash();
    finalTx.sign(user);

    const result = svm.sendTransaction(finalTx);

    if (result instanceof TransactionMetadata) {
      // console.log(result.logs());
      console.log(
        "computeUnitsConsumed: ",
        result.computeUnitsConsumed().toString()
      );
      // fs.writeFileSync("./logs.json", JSON.stringify(result.logs()));
    } else {
      console.log(result.meta().logs());
    }

    // check position after add liquidity
    const positionState = getDammV2Position(svm, position);
    poolState = getDammV2Pool(svm, pool);

    expect(positionState.unlockedLiquidity.gt(new BN(0))).to.be.true;

    const tokenAAmount = getAmountAFromLiquidityDelta(
      poolState.sqrtPrice,
      poolState.sqrtMaxPrice,
      positionState.unlockedLiquidity,
      Rounding.Down
    );

    const tokenBAmount = getAmountBFromLiquidityDelta(
      poolState.sqrtMinPrice,
      poolState.sqrtPrice,
      positionState.unlockedLiquidity,
      Rounding.Down
    );

    console.log({
      tokenAAmount: tokenAAmount.toString(),
      tokenBAmount: tokenBAmount.toString(),
    });
  });
});
