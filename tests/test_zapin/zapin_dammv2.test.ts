import {
  FailedTransactionMetadata,
  LiteSVM,
  TransactionMetadata,
} from "litesvm";
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
  warpSlotBy,
  TOKEN_DECIMALS,
  U64_MAX,
} from "../common";

import ZapIDL from "../../target/idl/zap.json";
import DAMMV2IDL from "../../idls/damm_v2.json";
import {
  convertToRateLimiterSecondFactor,
  createDammV2Pool,
  createDammV2Position,
  swap,
} from "../common/damm_v2";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { BN } from "@coral-xyz/anchor";
import { getDammV2Pool, getDammV2Position } from "../common/pda";
import { expect } from "chai";
import fs from "fs";

describe("Zap In damm V2", () => {
  let zapProgram: ZapProgram;
  let svm: LiteSVM;
  let tokenMint: PublicKey;
  let user: Keypair;
  let admin: Keypair;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
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
  });

  it("happy path zap in", async () => {
    const pool = await createDammV2Pool(svm, admin, tokenAMint, tokenBMint);

    const { position, positionNftAccount } = await createDammV2Position(
      svm,
      user,
      pool
    );

    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);
    await zapInFullFlow({
      svm,
      user,
      pool,
      position,
      positionNftAccount,
      inputTokenMint: tokenAMint,
      outputTokenMint: tokenBMint,
      totalAmount: amountTokenA,
      amountSwap,
    });
  });

  it("zap in with pool has fee scheduler", async () => {
    const baseFee = {
      cliffFeeNumerator: new BN(500_000_000), // 50 %
      firstFactor: 100, // 100 periods
      secondFactor: Array.from(new BN(1).toArrayLike(Buffer, "le", 8)),
      thirdFactor: new BN(4875000),
      baseFeeMode: 0, // fee scheduler Linear mode
    };

    warpSlotBy(svm, new BN(10));

    const pool = await createDammV2Pool(
      svm,
      admin,
      tokenAMint,
      tokenBMint,
      undefined,
      undefined,
      baseFee
    );

    const { position, positionNftAccount } = await createDammV2Position(
      svm,
      user,
      pool
    );

    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);
    await zapInFullFlow({
      svm,
      user,
      pool,
      position,
      positionNftAccount,
      inputTokenMint: tokenAMint,
      outputTokenMint: tokenBMint,
      totalAmount: amountTokenA,
      amountSwap,
    });
  });

  it("zap in with pool has rate limiter", async () => {
    let maxRateLimiterDuration = new BN(10);
    let maxFeeBps = new BN(5000);
    const baseFee = {
      cliffFeeNumerator: new BN(10_000_000), // 100bps
      firstFactor: 10, // 10 bps
      secondFactor: convertToRateLimiterSecondFactor(
        maxRateLimiterDuration,
        maxFeeBps
      ),
      thirdFactor: new BN(LAMPORTS_PER_SOL), // 1 SOL,
      baseFeeMode: 2, // rate limiter mode
    };

    const pool = await createDammV2Pool(
      svm,
      admin,
      tokenAMint,
      tokenBMint,
      undefined,
      undefined,
      baseFee
    );

    const { position, positionNftAccount } = await createDammV2Position(
      svm,
      user,
      pool
    );

    const amountTokenA = new BN(5 * LAMPORTS_PER_SOL); // 5 SOL
    const amountSwap = amountTokenA.divn(2);
    await zapInFullFlow({
      svm,
      user,
      pool,
      position,
      positionNftAccount,
      inputTokenMint: tokenAMint,
      outputTokenMint: tokenBMint,
      totalAmount: amountTokenA,
      amountSwap,
    });
  });

  it("zap in with pool low liquidity", async () => {
    const pool = await createDammV2Pool(
      svm,
      admin,
      tokenAMint,
      tokenBMint,
      new BN(LAMPORTS_PER_SOL),
      new BN(LAMPORTS_PER_SOL)
    );

    const { position, positionNftAccount } = await createDammV2Position(
      svm,
      user,
      pool
    );

    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);
    await zapInFullFlow({
      svm,
      user,
      pool,
      position,
      positionNftAccount,
      inputTokenMint: tokenAMint,
      outputTokenMint: tokenBMint,
      totalAmount: amountTokenA,
      amountSwap,
    });
  });

  it("zap in without external swap", async () => {
    const pool = await createDammV2Pool(
      svm,
      admin,
      tokenAMint,
      tokenBMint,
      new BN(LAMPORTS_PER_SOL),
      new BN(LAMPORTS_PER_SOL)
    );

    const { position, positionNftAccount } = await createDammV2Position(
      svm,
      user,
      pool
    );

    let poolState = getDammV2Pool(svm, pool);

    const totalAmountA = new BN(LAMPORTS_PER_SOL / 2); // 0.5 SOL
    const initializeLedgerTx = await initializeLedgerAccount(user.publicKey);

    const setLedgerBalanceTx = await setLedgerBalance(
      user.publicKey,
      totalAmountA,
      true
    );

    const tokenBAccount = getAssociatedTokenAddressSync(
      tokenBMint,
      user.publicKey
    );

    const preTokenBBalance = getTokenBalance(svm, tokenBAccount);

    const updateLedgerBalanceAfterSwapTx = await updateLedgerBalanceAfterSwap(
      user.publicKey,
      tokenBAccount,
      preTokenBBalance,
      U64_MAX,
      false
    );

    // zapin

    const zapInTx = await zapInDammv2({
      svm,
      user: user.publicKey,
      pool,
      position,
      positionNftAccount,
      preSqrtPrice: poolState.sqrtPrice,
      maxSqrtPriceChangeBps: 5000,
    });

    // close ledge
    const closeLedgerTx = await closeLedgerAccount(user.publicKey);

    const finalTx = new Transaction()
      .add(initializeLedgerTx)
      .add(setLedgerBalanceTx)
      .add(updateLedgerBalanceAfterSwapTx)
      .add(zapInTx)
      .add(closeLedgerTx);

    finalTx.recentBlockhash = svm.latestBlockhash();
    finalTx.sign(user);

    const result = svm.sendTransaction(finalTx);
    if (result instanceof FailedTransactionMetadata) {
      console.log(result.meta().logs());
    }
    expect(result).instanceOf(TransactionMetadata);
  });
});

async function zapInFullFlow(params: {
  svm: LiteSVM;
  user: Keypair;
  pool: PublicKey;
  position: PublicKey;
  positionNftAccount: PublicKey;
  inputTokenMint: PublicKey;
  outputTokenMint: PublicKey;
  totalAmount: BN;
  amountSwap: BN;
}) {
  const {
    svm,
    user,
    pool,
    position,
    positionNftAccount,
    inputTokenMint,
    outputTokenMint,
    amountSwap,
    totalAmount,
  } = params;

  let poolState = getDammV2Pool(svm, pool);
  const swapTx = await swap({
    svm,
    user: user.publicKey,
    pool,
    amountIn: amountSwap,
    inputTokenMint,
    outputTokenMint,
  });

  const initializeLedgerTx = await initializeLedgerAccount(user.publicKey);

  const setLedgerBalanceTx = await setLedgerBalance(
    user.publicKey,
    totalAmount.sub(amountSwap),
    inputTokenMint.equals(poolState.tokenAMint)
  );

  const tokenAAccount = getAssociatedTokenAddressSync(
    inputTokenMint,
    user.publicKey
  );

  const tokenBAccount = getAssociatedTokenAddressSync(
    outputTokenMint,
    user.publicKey
  );

  const preTokenABalance = getTokenBalance(svm, tokenAAccount);
  const preTokenBBalance = getTokenBalance(svm, tokenBAccount);

  const updateLedgerBalanceAfterSwapTx = await updateLedgerBalanceAfterSwap(
    user.publicKey,
    tokenBAccount,
    preTokenBBalance,
    U64_MAX,
    outputTokenMint.equals(poolState.tokenAMint)
  );

  // zapin

  const zapInTx = await zapInDammv2({
    svm,
    user: user.publicKey,
    pool,
    position,
    positionNftAccount,
    preSqrtPrice: poolState.sqrtPrice,
    maxSqrtPriceChangeBps: 5000,
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
  // if (result instanceof TransactionMetadata) {
  //   console.log(result.logs());
  // }
  expect(result).instanceOf(TransactionMetadata);

  // check position after add liquidity
  const positionState = getDammV2Position(svm, position);
  poolState = getDammV2Pool(svm, pool);

  expect(positionState.unlockedLiquidity.gt(new BN(0))).to.be.true;

  const postTokenABalance = getTokenBalance(svm, tokenAAccount);
  const postTokenBBalance = getTokenBalance(svm, tokenBAccount);

  const remainingTokenA = totalAmount
    .add(postTokenABalance)
    .sub(preTokenABalance);

  const remainingTokenB = postTokenBBalance.sub(preTokenBBalance);

  const remainAmountAPercent =
    remainingTokenA.toNumber() / totalAmount.toNumber();

  expect(remainAmountAPercent < 0.001); // 0.1%
  expect(remainingTokenB.toNumber() < 0.0001 * 10 ** TOKEN_DECIMALS); // 0.0001 token
}
