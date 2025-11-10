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
  zapOutDlmm,
  TOKEN_DECIMALS,
} from "../common";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { expect } from "chai";

import ZapIDL from "../../target/idl/zap.json";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import {
  binIdToBinArrayIndex,
  createBinArrays,
  createDlmmPool,
  createPresetParameter2,
  DLMM_PROGRAM_ID_LOCAL,
  dlmmCreatePositionAndAddLiquidityRadius,
  DEFAULT_BIN_PER_POSITION,
  removeAllLiquidity,
} from "../common/dlmm";
import { BN } from "@coral-xyz/anchor";

describe("Zap out dlmm", () => {
  let zapProgram: ZapProgram;
  let svm: LiteSVM;
  let user: Keypair;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
  let lbPair: PublicKey;

  const binStep = new BN(10);
  const activeId = new BN(5660);
  const lowerBinId =
    activeId.toNumber() - DEFAULT_BIN_PER_POSITION.toNumber() / 2;
  // 5 = Create 5 lower bin arrays, and 5 upper bin arrays surrounding the active bin arrays. Total bins = 600 * 11
  const binArrayDelta = 5;
  const upperBinId = DEFAULT_BIN_PER_POSITION.toNumber() + lowerBinId - 1;

  const admin = Keypair.fromSecretKey(
    new Uint8Array([
      230, 207, 238, 109, 95, 154, 47, 93, 183, 250, 147, 189, 87, 15, 117, 184,
      44, 91, 94, 231, 126, 140, 238, 134, 29, 58, 8, 182, 88, 22, 113, 234, 8,
      234, 192, 109, 87, 125, 190, 55, 129, 173, 227, 8, 104, 201, 104, 13, 31,
      178, 74, 80, 54, 14, 77, 78, 226, 57, 47, 122, 166, 165, 57, 144,
    ])
  );

  beforeEach(async () => {
    zapProgram = createZapProgram();
    svm = new LiteSVM();
    svm.addProgramFromFile(
      new PublicKey(ZapIDL.address),
      "./target/deploy/zap.so"
    );
    svm.addProgramFromFile(
      new PublicKey(DLMM_PROGRAM_ID_LOCAL),
      "./tests/fixtures/dlmm.so"
    );

    user = Keypair.generate();
    // admin = Keypair.generate();
    svm.airdrop(user.publicKey, BigInt(LAMPORTS_PER_SOL));
    svm.airdrop(admin.publicKey, BigInt(LAMPORTS_PER_SOL));

    tokenAMint = createToken(svm, admin, admin.publicKey, null);
    tokenBMint = createToken(svm, admin, admin.publicKey, null);
    mintToken(svm, admin, tokenAMint, admin, admin.publicKey);
    mintToken(svm, admin, tokenBMint, admin, admin.publicKey);

    mintToken(svm, admin, tokenAMint, admin, user.publicKey);
    mintToken(svm, admin, tokenBMint, admin, user.publicKey);

    console.log("create presetParameter2");
    let presetParameter2 = await createPresetParameter2(
      svm,
      admin,
      new BN(0),
      binStep.toNumber(),
      10000,
      0,
      0,
      0,
      0,
      0,
      500,
      0
    );

    console.log("create lb pair");
    lbPair = await createDlmmPool(
      svm,
      admin,
      tokenAMint,
      tokenBMint,
      activeId,
      TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      presetParameter2
    );

    console.log("Create bin array");
    const binArrayIndex = binIdToBinArrayIndex(activeId);
    const binArrayIndexes = [];

    // Lower bin arrays
    for (let i = binArrayDelta; i > 0; i--) {
      const idx = binArrayIndex.sub(new BN(i));
      binArrayIndexes.push(idx);
    }

    binArrayIndexes.push(binArrayIndex);

    // Upper bin arrays
    for (let i = 1; i <= binArrayDelta; i++) {
      const idx = binArrayIndex.add(new BN(i));
      binArrayIndexes.push(idx);
    }
    await createBinArrays(svm, admin, lbPair, binArrayIndexes);

    const amount = new BN(100_000_000).mul(new BN(10 ** TOKEN_DECIMALS));
    console.log("create position and add liquidity");

    await dlmmCreatePositionAndAddLiquidityRadius(
      svm,
      admin,
      lbPair,
      lowerBinId,
      activeId,
      tokenAMint,
      tokenBMint,
      amount,
      amount,
      new BN(lowerBinId),
      new BN(upperBinId)
    );
  });

  it("fullflow zap out", async () => {
    const inputTokenMint = tokenAMint;

    const amount = new BN(1000).mul(new BN(10 ** TOKEN_DECIMALS));
    const userPosition = await dlmmCreatePositionAndAddLiquidityRadius(
      svm,
      user,
      lbPair,
      lowerBinId,
      activeId,
      tokenAMint,
      tokenBMint,
      amount,
      amount,
      new BN(lowerBinId),
      new BN(upperBinId)
    );

    const tokenXAccount = getAssociatedTokenAddressSync(
      tokenAMint,
      user.publicKey,
      true,
      TOKEN_PROGRAM_ID
    );
    const tokenYAccount = getAssociatedTokenAddressSync(
      tokenBMint,
      user.publicKey,
      true,
      TOKEN_PROGRAM_ID
    );

    const removeLiquidityTx = await removeAllLiquidity(
      svm,
      lbPair,
      user,
      userPosition,
      tokenAMint,
      tokenBMint,
      tokenXAccount,
      tokenYAccount,
      lowerBinId,
      upperBinId
    );

    const zapOutTx = await zapOutDlmm(
      svm,
      lbPair,
      user.publicKey,
      inputTokenMint
    );

    const finalTransaction = new Transaction()
      .add(removeLiquidityTx)
      .add(zapOutTx);

    finalTransaction.recentBlockhash = svm.latestBlockhash();
    finalTransaction.sign(user);

    const result = svm.sendTransaction(finalTransaction);
    if (result instanceof FailedTransactionMetadata) {
      console.log(result.meta().logs());
    }
    expect(result).instanceOf(TransactionMetadata);
  });
});
