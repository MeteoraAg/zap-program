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
  AccountMeta,
} from "@solana/web3.js";
import {
  createToken,
  mintToken,
  TOKEN_DECIMALS,
  initializeLedgerAccount,
  setLedgerBalance,
  getTokenBalance,
  updateLedgerBalanceAfterSwap,
  zapInDlmmforInitializedPosition,
  closeLedgerAccount,
  U64_MAX,
} from "../common";
import babar from "babar";
import { expect } from "chai";

import ZapIDL from "../../target/idl/zap.json";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import {
  binIdToBinArrayIndex,
  createBinArrays,
  createDlmmPosition,
  DLMM_PROGRAM_ID_LOCAL,
  dlmmCreatePositionAndAddLiquidityRadius,
  dlmmSwap,
  getLbPairState,
  getPositionTotalLiquidityAllBin,
  DEFAULT_BIN_PER_POSITION,
  RemainingAccountsInfo,
  StrategyType,
  createDlmmPermissionlessPool,
  SET_COMPUTE_UNIT_LIMIT_IX,
  initializeBinArrayBitmapExtension,
  getBinArrayAccountMetaByBinRange,
} from "../common/dlmm";
import { BN } from "@coral-xyz/anchor";
import { deriveBinArrayBitmapExtension } from "../common/pda";

describe("Zapin DLMM with initialize position", () => {
  let svm: LiteSVM;
  let user: Keypair;
  let tokenXMint: PublicKey;
  let tokenYMint: PublicKey;
  let lbPair: PublicKey;

  const binStep = new BN(10);
  const activeId = new BN(5660);
  const lowerBinId =
    activeId.toNumber() - DEFAULT_BIN_PER_POSITION.toNumber() / 2;
  // 5 = Create 5 lower bin arrays, and 5 upper bin arrays surrounding the active bin arrays. Total bins = 600 * 11
  const binArrayDelta = 5;
  const upperBinId = DEFAULT_BIN_PER_POSITION.toNumber() + lowerBinId - 1;

  const minDeltaId = -DEFAULT_BIN_PER_POSITION.toNumber() / 2;
  const maxDeltaId = DEFAULT_BIN_PER_POSITION.toNumber() / 2;

  const admin = Keypair.fromSecretKey(
    new Uint8Array([
      230, 207, 238, 109, 95, 154, 47, 93, 183, 250, 147, 189, 87, 15, 117, 184,
      44, 91, 94, 231, 126, 140, 238, 134, 29, 58, 8, 182, 88, 22, 113, 234, 8,
      234, 192, 109, 87, 125, 190, 55, 129, 173, 227, 8, 104, 201, 104, 13, 31,
      178, 74, 80, 54, 14, 77, 78, 226, 57, 47, 122, 166, 165, 57, 144,
    ])
  );

  beforeEach(async () => {
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

    tokenXMint = createToken(svm, admin, admin.publicKey, null);
    tokenYMint = createToken(svm, admin, admin.publicKey, null);
    mintToken(svm, admin, tokenXMint, admin, admin.publicKey);
    mintToken(svm, admin, tokenYMint, admin, admin.publicKey);

    mintToken(svm, admin, tokenXMint, admin, user.publicKey);
    mintToken(svm, admin, tokenYMint, admin, user.publicKey);

    console.log("create lb pair");
    lbPair = await createDlmmPermissionlessPool({
      svm,
      creator: admin,
      tokenX: tokenXMint,
      tokenY: tokenYMint,
      activeId,
      baseFactor: 10000,
      binStep: binStep.toNumber(),
    });

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

    const amount = new BN(1000).mul(new BN(10 ** TOKEN_DECIMALS));
    await dlmmCreatePositionAndAddLiquidityRadius(
      svm,
      admin,
      lbPair,
      lowerBinId,
      activeId,
      tokenXMint,
      tokenYMint,
      amount,
      amount,
      new BN(lowerBinId),
      new BN(upperBinId)
    );
  });

  it("Zapin dlmm with Spot strategy", async () => {
    await initializeBinArrayBitmapExtension(svm, lbPair, admin);
    const position = await createDlmmPosition(svm, user, lbPair, lowerBinId);
    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);

    const binArrays = getBinArrayAccountMetaByBinRange(
      lbPair,
      new BN(lowerBinId),
      new BN(upperBinId)
    );

    await zapInDlmmFullFlow({
      svm,
      user,
      lbPair,
      position,
      inputTokenMint: tokenXMint,
      outputTokenMint: tokenYMint,
      totalAmount: amountTokenA,
      amountSwap,
      strategy: StrategyType.Spot,
      minDeltaId,
      maxDeltaId,
      binArrays,
      remainingAccountInfo: { slices: [] },
    });
  });

  it("Zapin dlmm with Bidask strategy", async () => {
    await initializeBinArrayBitmapExtension(svm, lbPair, admin);
    const position = await createDlmmPosition(svm, user, lbPair, lowerBinId);

    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);

    const binArrays = getBinArrayAccountMetaByBinRange(
      lbPair,
      new BN(lowerBinId),
      new BN(upperBinId)
    );

    await zapInDlmmFullFlow({
      svm,
      user,
      lbPair,
      position,
      inputTokenMint: tokenXMint,
      outputTokenMint: tokenYMint,
      totalAmount: amountTokenA,
      amountSwap,
      minDeltaId,
      maxDeltaId,
      strategy: StrategyType.BidAsk,
      binArrays,
      remainingAccountInfo: { slices: [] },
    });
  });

  it("Zapin dlmm with Curve strategy", async () => {
    await initializeBinArrayBitmapExtension(svm, lbPair, admin);
    const position = await createDlmmPosition(svm, user, lbPair, lowerBinId);

    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);

    const binArrays = getBinArrayAccountMetaByBinRange(
      lbPair,
      new BN(lowerBinId),
      new BN(upperBinId)
    );

    await zapInDlmmFullFlow({
      svm,
      user,
      lbPair,
      position,
      inputTokenMint: tokenXMint,
      outputTokenMint: tokenYMint,
      totalAmount: amountTokenA,
      amountSwap,
      minDeltaId,
      maxDeltaId,
      strategy: StrategyType.Curve,
      binArrays,
      remainingAccountInfo: { slices: [] },
    });
  });

  it("Zapin dlmm without bin array bitmap extension", async () => {
    const position = await createDlmmPosition(svm, user, lbPair, lowerBinId);
    const amountTokenA = new BN(LAMPORTS_PER_SOL);
    const amountSwap = amountTokenA.divn(2);

    const binArrays = getBinArrayAccountMetaByBinRange(
      lbPair,
      new BN(lowerBinId),
      new BN(upperBinId)
    );

    await zapInDlmmFullFlow({
      svm,
      user,
      lbPair,
      position,
      inputTokenMint: tokenXMint,
      outputTokenMint: tokenYMint,
      totalAmount: amountTokenA,
      amountSwap,
      strategy: StrategyType.Spot,
      minDeltaId,
      maxDeltaId,
      binArrays,
      remainingAccountInfo: { slices: [] },
    });
  });
});

async function zapInDlmmFullFlow(params: {
  svm: LiteSVM;
  user: Keypair;
  lbPair: PublicKey;
  position: PublicKey;
  inputTokenMint: PublicKey;
  outputTokenMint: PublicKey;
  totalAmount: BN;
  amountSwap: BN;
  strategy;
  minDeltaId: number;
  maxDeltaId: number;
  binArrays: AccountMeta[];
  remainingAccountInfo: RemainingAccountsInfo;
}) {
  const {
    svm,
    user,
    lbPair,
    position,
    inputTokenMint,
    outputTokenMint,
    amountSwap,
    totalAmount,
    minDeltaId,
    maxDeltaId,
    strategy,
    binArrays,
    remainingAccountInfo,
  } = params;

  let lbPairState = getLbPairState(svm, lbPair);
  const swapForY = inputTokenMint.equals(lbPairState.tokenXMint);
  const swapTx = await dlmmSwap(
    svm,
    user.publicKey,
    lbPair,
    amountSwap,
    swapForY
  );

  const initializeLedgerTx = await initializeLedgerAccount(user.publicKey);

  const setLedgerBalanceTx = await setLedgerBalance(
    user.publicKey,
    totalAmount.sub(amountSwap),
    inputTokenMint.equals(lbPairState.tokenXMint)
  );

  // update balance after swapxww
  const tokenBAccount = getAssociatedTokenAddressSync(
    outputTokenMint,
    user.publicKey
  );

  const preBalance = getTokenBalance(svm, tokenBAccount);

  const updateLedgerBalanceAfterSwapTx = await updateLedgerBalanceAfterSwap(
    user.publicKey,
    tokenBAccount,
    preBalance,
    U64_MAX,
    outputTokenMint.equals(lbPairState.tokenXMint)
  );

  // zapin
  const [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);
  const zapInTx = await zapInDlmmforInitializedPosition({
    svm,
    owner: user.publicKey,
    lbPair,
    position,
    activeId: lbPairState.activeId,
    minDeltaId,
    maxDeltaId,
    maxActiveBinSlippage: 10,
    favorXInActiveId: true,
    strategy,
    remainingAccountInfo,
    binArrays,
    binArrayBitmapExtension,
  });

  // close ledge
  const closeLedgerTx = await closeLedgerAccount(user.publicKey);

  const finalTx = new Transaction()
    .add(SET_COMPUTE_UNIT_LIMIT_IX)
    .add(swapTx)
    .add(initializeLedgerTx)
    .add(setLedgerBalanceTx)
    .add(updateLedgerBalanceAfterSwapTx)
    .add(zapInTx)
    .add(closeLedgerTx);

  finalTx.recentBlockhash = svm.latestBlockhash();
  finalTx.sign(user);

  const result = svm.sendTransaction(finalTx);
  if (result instanceof FailedTransactionMetadata) {
    console.log(result.err());
    console.log(result.meta().logs());
  } else {
    // console.log(result.logs());
  }
  expect(result).instanceOf(TransactionMetadata);

  let liquidities = getPositionTotalLiquidityAllBin(svm, position);
  console.log(babar(liquidities));
}
