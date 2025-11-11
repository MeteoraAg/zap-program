import {
  AnchorProvider,
  BN,
  IdlAccounts,
  IdlTypes,
  Program,
  Wallet,
} from "@coral-xyz/anchor";
import * as borsh from "borsh";
import { LbClmm } from "./idl/dlmm";
import DlmmIDL from "./idl/dlmm.json";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  ComputeBudgetProgram,
  AccountMeta,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  FailedTransactionMetadata,
  LiteSVM,
  TransactionMetadata,
} from "litesvm";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import {
  deriveBinArray,
  deriveBinArrayBitmapExtension,
  deriveDlmmEventAuthority,
  deriveLbCustomizablePermissionless2,
  deriveLbPermissionless2,
  deriveOracle,
  derivePresetParameter2,
  deriveReserve,
  deriveTokenBadge,
} from "./pda";
import { getExtraAccountMetasForTransferHook } from "./transferHook/transferHookUtils";
import { getOrCreateAtA } from "./utils";
import { pow } from "./math";

export type PresetParameter = Omit<IdlAccounts<LbClmm>["presetParameter"], "">;
export type BinLiquidityDistribution =
  IdlTypes<LbClmm>["binLiquidityDistribution"];

export type BinArrayBitmapExtension =
  IdlTypes<LbClmm>["binArrayBitmapExtension"];
export type RemainingAccountsInfo = IdlTypes<LbClmm>["remainingAccountsInfo"];

export type LbPairState = IdlAccounts<LbClmm>["lbPair"];

export const StrategyType = {
  Spot: { spot: {} },
  Curve: { curve: {} },
  BidAsk: { bidAsk: {} },
};
export const AccountsType = {
  TransferHookX: {
    transferHookX: {},
  },
  TransferHookY: {
    transferHookY: {},
  },
  TransferHookReward: {
    transferHookReward: {},
  },
};
export const DLMM_PROGRAM_ID_LOCAL = new PublicKey(
  "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
);

export const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
);

export const DLMM_SWAP_DISC = [65, 75, 63, 76, 235, 91, 91, 136];

const CONSTANTS = Object.entries(DlmmIDL.constants);
export const BIN_ARRAY_BITMAP_SIZE = new BN(
  CONSTANTS.find(([k, v]) => v.name == "BIN_ARRAY_BITMAP_SIZE")[1].value
);
export const DEFAULT_BIN_PER_POSITION = new BN(
  CONSTANTS.find(([k, v]) => v.name == "DEFAULT_BIN_PER_POSITION")[1].value
);

export const DEFAULT_BITMAP_RANGE = [
  BIN_ARRAY_BITMAP_SIZE.neg(),
  BIN_ARRAY_BITMAP_SIZE.sub(new BN(1)),
];

export const MAX_BIN_PER_ARRAY = new BN(
  CONSTANTS.find(([k, v]) => v.name == "MAX_BIN_PER_ARRAY")[1].value
);

export const BASIS_POINT_MAX = new BN(
  CONSTANTS.find(([k, v]) => v.name == "BASIS_POINT_MAX")[1].value
);

export const SET_COMPUTE_UNIT_LIMIT_INIT_BIN_ARRAY_IX =
  ComputeBudgetProgram.setComputeUnitLimit({
    units: 300_000,
  });
export const SET_COMPUTE_UNIT_LIMIT_IX =
  ComputeBudgetProgram.setComputeUnitLimit({
    units: 1_400_000,
  });

export const EXTENSION_BINARRAY_BITMAP_SIZE = new BN(
  CONSTANTS.find(
    ([k, v]) => v.name == "EXTENSION_BINARRAY_BITMAP_SIZE"
  )[1].value
);
export const BIN_ARRAY_INDEX_BOUND = [
  BIN_ARRAY_BITMAP_SIZE.mul(
    EXTENSION_BINARRAY_BITMAP_SIZE.add(new BN(1))
  ).neg(),
  BIN_ARRAY_BITMAP_SIZE.mul(EXTENSION_BINARRAY_BITMAP_SIZE.add(new BN(1))).sub(
    new BN(1)
  ),
];
export const PRESET_PARAMETER_TEST: PresetParameter[] = [
  {
    binStep: 1,
    baseFactor: 20000,
    filterPeriod: 10,
    decayPeriod: 120,
    reductionFactor: 5000,
    variableFeeControl: 2000000,
    protocolShare: 0,
    maxVolatilityAccumulator: 100000,
    maxBinId: 436704,
    minBinId: -436704,
  },
  {
    binStep: 2,
    baseFactor: 15000,
    filterPeriod: 10,
    decayPeriod: 120,
    reductionFactor: 5000,
    variableFeeControl: 500000,
    protocolShare: 0,
    maxBinId: 218363,
    minBinId: -218363,
    maxVolatilityAccumulator: 250000,
  },
  {
    binStep: 5,
    baseFactor: 8000,
    filterPeriod: 30,
    decayPeriod: 600,
    reductionFactor: 5000,
    variableFeeControl: 120000,
    protocolShare: 0,
    maxBinId: 87358,
    minBinId: -87358,
    maxVolatilityAccumulator: 300000,
  },
  {
    binStep: 10,
    baseFactor: 10000,
    filterPeriod: 30,
    decayPeriod: 600,
    reductionFactor: 5000,
    variableFeeControl: 40000,
    protocolShare: 0,
    maxBinId: 43690,
    minBinId: -43690,
    maxVolatilityAccumulator: 350000,
  },
  {
    binStep: 15,
    baseFactor: 10000,
    filterPeriod: 30,
    decayPeriod: 600,
    reductionFactor: 5000,
    variableFeeControl: 30000,
    protocolShare: 0,
    maxBinId: 29134,
    minBinId: -29134,
    maxVolatilityAccumulator: 350000,
  },
  {
    binStep: 20,
    baseFactor: 10000,
    filterPeriod: 30,
    decayPeriod: 600,
    reductionFactor: 5000,
    variableFeeControl: 20000,
    protocolShare: 0,
    maxBinId: 21855,
    minBinId: -21855,
    maxVolatilityAccumulator: 350000,
  },
  {
    binStep: 25,
    baseFactor: 10000,
    filterPeriod: 30,
    decayPeriod: 600,
    reductionFactor: 5000,
    variableFeeControl: 15000,
    // protocolShare: 2000,
    protocolShare: 0,
    maxBinId: 17481,
    minBinId: -17481,
    maxVolatilityAccumulator: 350000,
  },
  {
    binStep: 50,
    baseFactor: 8000,
    filterPeriod: 120,
    decayPeriod: 1200,
    reductionFactor: 5000,
    variableFeeControl: 10000,
    protocolShare: 0,
    maxBinId: 8754,
    minBinId: -8754,
    maxVolatilityAccumulator: 250000,
  },
  {
    binStep: 100,
    baseFactor: 8000,
    filterPeriod: 300,
    decayPeriod: 1200,
    reductionFactor: 5000,
    variableFeeControl: 7500,
    protocolShare: 0,
    maxBinId: 4386,
    minBinId: -4386,
    maxVolatilityAccumulator: 150000,
  },
];

export function createDlmmProgram() {
  const wallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(
    new Connection(clusterApiUrl("devnet")),
    wallet,
    {}
  );
  const program = new Program<LbClmm>(DlmmIDL as LbClmm, provider);
  return program;
}

export function getLbPairState(svm: LiteSVM, lbPair: PublicKey): LbPairState {
  const program = createDlmmProgram();
  const account = svm.getAccount(lbPair);
  return program.coder.accounts.decode("lbPair", Buffer.from(account.data));
}

export function getBinArrayBitmapExtension(
  svm: LiteSVM,
  binArray: PublicKey
): BinArrayBitmapExtension | null {
  const program = createDlmmProgram();
  const account = svm.getAccount(binArray);
  if (!account) {
    return null;
  }
  return program.coder.accounts.decode(
    "binArrayBitmapExtension",
    Buffer.from(account.data)
  );
}

export function binIdToBinArrayIndex(binId: BN) {
  if (binId.isNeg()) {
    const idx = binId.add(new BN(1)).div(MAX_BIN_PER_ARRAY);
    return idx.sub(new BN(1));
  }
  const idx = binId.div(MAX_BIN_PER_ARRAY);
  return idx;
}

export function getSpotPatternDistribution(
  delta: number,
  activeId: number
): BinLiquidityDistribution[] {
  const positiveDelta = Math.abs(delta);
  const negativeDelta = -positiveDelta;

  const binLiquidityDist = [];

  const distPerNonActiveBin = Math.floor(
    BASIS_POINT_MAX.toNumber() / (0.5 + delta)
  );

  for (let i = negativeDelta; i <= positiveDelta; i++) {
    let deltaId = i;
    let distributionX = 0;
    let distributionY = 0;

    if (i < 0) {
      distributionY = distPerNonActiveBin;
    } else if (i == 0) {
      distributionX = BASIS_POINT_MAX.toNumber() - distPerNonActiveBin * delta;
      distributionY = BASIS_POINT_MAX.toNumber() - distPerNonActiveBin * delta;
    } else {
      distributionX = distPerNonActiveBin;
    }

    let dist: BinLiquidityDistribution = {
      binId: activeId + deltaId,
      distributionX,
      distributionY,
    };
    binLiquidityDist.push(dist);
  }

  return binLiquidityDist;
}

export function getBinArrayAccountMetaByBinRange(
  lbPair: PublicKey,
  lowerBinId: BN,
  upperBinId: BN
): AccountMeta[] {
  const binArray = getBinArrayByBinRange(lbPair, lowerBinId, upperBinId);

  return binArray.map((key) => {
    return {
      pubkey: key,
      isSigner: false,
      isWritable: true,
    };
  });
}

export function getBinArrayByBinRange(
  lbPair: PublicKey,
  lowerBinId: BN,
  upperBinId: BN
): PublicKey[] {
  let binArrayIndex = binIdToBinArrayIndex(lowerBinId);
  const binArrays = [];
  while (true) {
    const [binArray] = deriveBinArray(lbPair, binArrayIndex);
    binArrays.push(binArray);

    const [binArrayLowerBinId, binArrayUpperBinId] =
      getBinArrayLowerUpperBinId(binArrayIndex);

    if (
      upperBinId.gte(binArrayLowerBinId) &&
      upperBinId.lte(binArrayUpperBinId)
    ) {
      break;
    } else {
      binArrayIndex = binArrayIndex.add(new BN(1));
    }
  }

  return binArrays;
}

export function getBinArrayLowerUpperBinId(binArrayIndex: BN) {
  const lowerBinId = binArrayIndex.mul(MAX_BIN_PER_ARRAY);
  const upperBinId = lowerBinId.add(MAX_BIN_PER_ARRAY).sub(new BN(1));

  return [lowerBinId, upperBinId];
}

export async function createBinArrays(
  svm: LiteSVM,
  payer: Keypair,
  lbPair: PublicKey,
  indexes: BN[]
) {
  const program = createDlmmProgram();
  for (const idx of indexes) {
    const [binArray] = deriveBinArray(lbPair, idx);

    const binArrayAccount = svm.getAccount(binArray);

    if (binArrayAccount == null) {
      const tx = await program.methods
        .initializeBinArray(idx)
        .accountsPartial({
          binArray,
          funder: payer.publicKey,
          lbPair,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([SET_COMPUTE_UNIT_LIMIT_INIT_BIN_ARRAY_IX])
        .transaction();

      tx.recentBlockhash = svm.latestBlockhash();
      tx.sign(payer);
      const result = svm.sendTransaction(tx);

      if (result instanceof FailedTransactionMetadata) {
        console.log(result.meta().toString());
      }
      expect(result).instanceOf(TransactionMetadata);
    }
  }
}

export async function initializeTokenBadge(
  svm: LiteSVM,
  admin: Keypair,
  tokenMint: PublicKey
) {
  const program = createDlmmProgram();
  const tx = await program.methods
    .initializeTokenBadge()
    .accountsPartial({
      tokenMint,
      tokenBadge: deriveTokenBadge(tokenMint),
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  tx.recentBlockhash = svm.latestBlockhash();
  tx.sign(admin);

  const result = svm.sendTransaction(tx);
  expect(result).instanceOf(TransactionMetadata);
}

export async function createPresetParameter2(
  svm: LiteSVM,
  payer: Keypair,
  index: BN,
  binStep: number,
  baseFactor: number,
  filterPeriod: number,
  decayPeriod: number,
  reductionFactor: number,
  variableFeeControl: number,
  maxVolatilityAccumulator: number,
  protocolShare: number,
  baseFeePowerFactor: number
): Promise<PublicKey> {
  const program = createDlmmProgram();
  const [presetParameter] = derivePresetParameter2(index);

  const presetParamAccount = svm.getAccount(presetParameter);
  if (!presetParamAccount) {
    const tx = await program.methods
      .initializePresetParameter2({
        index: index.toNumber(),
        binStep,
        baseFactor,
        filterPeriod,
        decayPeriod,
        reductionFactor,
        variableFeeControl,
        maxVolatilityAccumulator,
        protocolShare,
        baseFeePowerFactor,
      })
      .accountsPartial({
        admin: payer.publicKey,
        presetParameter,
        systemProgram: SystemProgram.programId,
      })
      .transaction();
    svm.expireBlockhash();
    tx.recentBlockhash = svm.latestBlockhash();
    tx.sign(payer);

    const result = svm.sendTransaction(tx);
    if (result instanceof FailedTransactionMetadata) {
      console.log(result.err());
      console.log(result.meta().logs());
    }
    expect(result).instanceOf(TransactionMetadata);
  }
  return presetParameter;
}

export async function createDlmmPool(
  svm: LiteSVM,
  creator: Keypair,
  tokenX: PublicKey,
  tokenY: PublicKey,
  activeId: BN,
  tokenProgramX: PublicKey,
  tokenProgramY: PublicKey,
  presetParameter2: PublicKey
) {
  const program = createDlmmProgram();
  const [lbPair] = deriveLbPermissionless2(presetParameter2, tokenX, tokenY);

  const [reserveX] = deriveReserve(tokenX, lbPair);
  const [reserveY] = deriveReserve(tokenY, lbPair);
  const [oracle] = deriveOracle(lbPair);

  const binArrayIndex = binIdToBinArrayIndex(activeId);

  const [minBinArrayIndex, maxBinArrayIndex] = DEFAULT_BITMAP_RANGE;

  const binArrayBitmapExtension =
    binArrayIndex.gt(maxBinArrayIndex) || binArrayIndex.lt(minBinArrayIndex)
      ? deriveBinArrayBitmapExtension(lbPair)[0]
      : null;

  const tokenBadgeX = deriveTokenBadge(tokenX);
  const tokenBadgeXState = svm.getAccount(tokenBadgeX);
  const tokenBadgeY = deriveTokenBadge(tokenX);
  const tokenBadgeYState = svm.getAccount(tokenBadgeY);

  const tx = await program.methods
    .initializeLbPair2({
      activeId: activeId.toNumber(),
      padding: new Array(96).fill(0),
    })
    .accountsPartial({
      funder: creator.publicKey,
      lbPair,
      reserveX,
      reserveY,
      oracle,
      binArrayBitmapExtension,
      tokenMintX: tokenX,
      tokenMintY: tokenY,
      tokenProgramX,
      tokenProgramY,
      tokenBadgeX: tokenBadgeXState ? tokenBadgeX : null,
      tokenBadgeY: tokenBadgeYState ? tokenBadgeY : null,
      systemProgram: SystemProgram.programId,
      presetParameter: presetParameter2,
    })
    .transaction();
  tx.recentBlockhash = svm.latestBlockhash();
  tx.sign(creator);

  const result = svm.sendTransaction(tx);

  if (result instanceof FailedTransactionMetadata) {
    console.log(result.meta().toString());
  }
  expect(result).instanceOf(TransactionMetadata);

  return lbPair;
}

export async function initializeBinArrayBitmapExtension(
  svm: LiteSVM,
  lbPair: PublicKey,
  payer: Keypair
) {
  const [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);
  const program = createDlmmProgram();
  const binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);
  if (!binArrayBitmapExtensionState) {
    const tx = await program.methods
      .initializeBinArrayBitmapExtension()
      .accountsPartial({
        lbPair,
        binArrayBitmapExtension,
        systemProgram: SystemProgram.programId,
        funder: payer.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .transaction();

    tx.recentBlockhash = svm.latestBlockhash();
    tx.sign(payer);

    const result = svm.sendTransaction(tx);
    expect(result).instanceOf(TransactionMetadata);
  }
}

export async function createDlmmPermissionlessPool(params: {
  svm: LiteSVM;
  creator: Keypair;
  tokenX: PublicKey;
  tokenY: PublicKey;
  activeId: BN;
  baseFactor: number;
  binStep: number;
}) {
  const { svm, creator, tokenX, tokenY, activeId, baseFactor, binStep } =
    params;
  const program = createDlmmProgram();
  const lbPair = deriveLbCustomizablePermissionless2(tokenX, tokenY);

  const [reserveX] = deriveReserve(tokenX, lbPair);
  const [reserveY] = deriveReserve(tokenY, lbPair);
  const [oracle] = deriveOracle(lbPair);

  const binArrayIndex = binIdToBinArrayIndex(activeId);

  const [minBinArrayIndex, maxBinArrayIndex] = DEFAULT_BITMAP_RANGE;

  const binArrayBitmapExtension =
    binArrayIndex.gt(maxBinArrayIndex) || binArrayIndex.lt(minBinArrayIndex)
      ? deriveBinArrayBitmapExtension(lbPair)[0]
      : null;

  const tokenBadgeX = deriveTokenBadge(tokenX);
  const tokenBadgeXState = svm.getAccount(tokenBadgeX);
  const tokenBadgeY = deriveTokenBadge(tokenX);
  const tokenBadgeYState = svm.getAccount(tokenBadgeY);

  const tokenProgramX = svm.getAccount(tokenX).owner;
  const tokenProgramY = svm.getAccount(tokenY).owner;
  const userTokenX = getAssociatedTokenAddressSync(
    tokenX,
    creator.publicKey,
    true,
    tokenProgramX
  );

  const userTokenY = getAssociatedTokenAddressSync(
    tokenX,
    creator.publicKey,
    true,
    tokenProgramX
  );

  const tx = await program.methods
    .initializeCustomizablePermissionlessLbPair2({
      activeId: activeId.toNumber(),
      binStep,
      baseFactor,
      activationPoint: null,
      activationType: 0,
      hasAlphaVault: false,
      creatorPoolOnOffControl: true,
      baseFeePowerFactor: 0,
      padding: new Array(63).fill(0),
    })
    .accountsPartial({
      funder: creator.publicKey,
      lbPair,
      reserveX,
      reserveY,
      oracle,
      binArrayBitmapExtension,
      tokenMintX: tokenX,
      tokenMintY: tokenY,
      tokenProgramX: svm.getAccount(tokenX).owner,
      tokenProgramY: svm.getAccount(tokenY).owner,
      tokenBadgeX: tokenBadgeXState ? tokenBadgeX : null,
      tokenBadgeY: tokenBadgeYState ? tokenBadgeY : null,
      systemProgram: SystemProgram.programId,
      userTokenX,
      userTokenY,
    })
    .transaction();
  tx.recentBlockhash = svm.latestBlockhash();
  tx.sign(creator);

  const result = svm.sendTransaction(tx);

  if (result instanceof FailedTransactionMetadata) {
    console.log(result.err());
    console.log(result.meta().logs());
  }
  expect(result).instanceOf(TransactionMetadata);

  return lbPair;
}

export async function createDlmmPosition(
  svm: LiteSVM,
  creator: Keypair,
  lbPair: PublicKey,
  lowerBinId: number,
  width?: number
) {
  const program = createDlmmProgram();
  const positionWidth = width ? width : DEFAULT_BIN_PER_POSITION.toNumber();
  const position = Keypair.generate();

  let [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);
  let binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);
  if (!binArrayBitmapExtensionState) {
    binArrayBitmapExtension = null;
  }

  const createPositionTx = await program.methods
    .initializePosition(lowerBinId, positionWidth)
    .accountsPartial({
      lbPair,
      owner: creator.publicKey,
      payer: creator.publicKey,
      position: position.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  const finalTx = new Transaction().add(createPositionTx);
  finalTx.recentBlockhash = svm.latestBlockhash();
  finalTx.sign(creator, position);

  const result = svm.sendTransaction(finalTx);
  expect(result).instanceOf(TransactionMetadata);

  return position.publicKey;
}

export async function dlmmCreatePositionAndAddLiquidityRadius(
  svm: LiteSVM,
  creator: Keypair,
  lbPair: PublicKey,
  lowerBinId: number,
  activeId: BN,
  tokenXMint: PublicKey,
  tokenYMint: PublicKey,
  amountX: BN,
  amountY: BN,
  minBinId: BN,
  maxBinId: BN,
  tokenXProgram = TOKEN_PROGRAM_ID,
  tokenYProgram = TOKEN_PROGRAM_ID,
  width?: number
) {
  const program = createDlmmProgram();
  const positionWidth = width ? width : DEFAULT_BIN_PER_POSITION.toNumber();
  const position = Keypair.generate();
  const userTokenX = getAssociatedTokenAddressSync(
    tokenXMint,
    creator.publicKey,
    true,
    tokenXProgram
  );

  const userTokenY = getAssociatedTokenAddressSync(
    tokenYMint,
    creator.publicKey,
    true,
    tokenYProgram
  );

  let [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);
  let binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);
  if (!binArrayBitmapExtensionState) {
    binArrayBitmapExtension = null;
  }

  const [reserveX] = deriveReserve(tokenXMint, lbPair);
  const [reserveY] = deriveReserve(tokenYMint, lbPair);

  const createPositionTx = await program.methods
    .initializePosition(lowerBinId, positionWidth)
    .accountsPartial({
      lbPair,
      owner: creator.publicKey,
      payer: creator.publicKey,
      position: position.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  const binArrays = getBinArrayAccountMetaByBinRange(
    lbPair,
    new BN(minBinId),
    new BN(maxBinId)
  );

  const transferHookXAccounts = getExtraAccountMetasForTransferHook(
    svm,
    tokenXMint
  );

  const transferHookYAccounts = getExtraAccountMetasForTransferHook(
    svm,
    tokenYMint
  );

  let remainingAccountsInfo: RemainingAccountsInfo = { slices: [] };

  if (transferHookXAccounts.length > 0) {
    remainingAccountsInfo.slices.push({
      accountsType: AccountsType.TransferHookX,
      length: transferHookXAccounts.length,
    });
  }

  if (transferHookYAccounts.length > 0) {
    remainingAccountsInfo.slices.push({
      accountsType: AccountsType.TransferHookY,
      length: transferHookYAccounts.length,
    });
  }
  const addLiquidityTx = await program.methods
    .addLiquidityByStrategy2(
      {
        amountX,
        amountY,
        activeId: activeId.toNumber(),
        maxActiveBinSlippage: 3,
        strategyParameters: {
          minBinId: minBinId.toNumber(),
          maxBinId: maxBinId.toNumber(),
          strategyType: { spotBalanced: {} },
          parameteres: Buffer.from(new Array<number>(64).fill(0)).toJSON().data,
        },
      },
      remainingAccountsInfo
    )
    .accountsPartial({
      lbPair,
      binArrayBitmapExtension,
      position: position.publicKey,
      reserveX: reserveX,
      reserveY: reserveY,
      tokenXMint,
      tokenYMint,
      tokenXProgram,
      tokenYProgram,
      sender: creator.publicKey,
      userTokenX,
      userTokenY,
    })
    .remainingAccounts([...transferHookXAccounts, ...transferHookYAccounts])
    .remainingAccounts(binArrays)
    .preInstructions([SET_COMPUTE_UNIT_LIMIT_IX])
    .transaction();

  const finalTx = new Transaction().add(createPositionTx).add(addLiquidityTx);
  finalTx.recentBlockhash = svm.latestBlockhash();
  finalTx.sign(creator, position);

  const result = svm.sendTransaction(finalTx);

  if (result instanceof FailedTransactionMetadata) {
    console.log(result.meta().toString());
  }
  expect(result).instanceOf(TransactionMetadata);

  return position.publicKey;
}

export async function removeAllLiquidity(
  svm: LiteSVM,
  lbPair: PublicKey,
  user: Keypair,
  position: PublicKey,
  tokenXMint: PublicKey,
  tokenYMint: PublicKey,
  userTokenX: PublicKey,
  userTokenY: PublicKey,
  lowerBinId: number,
  upperBinId: number,
  tokenXProgram = TOKEN_PROGRAM_ID,
  tokenYProgram = TOKEN_PROGRAM_ID
) {
  const program = createDlmmProgram();
  let [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);
  let binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);
  if (!binArrayBitmapExtensionState) {
    binArrayBitmapExtension = null;
  }

  const [reserveX] = deriveReserve(tokenXMint, lbPair);
  const [reserveY] = deriveReserve(tokenYMint, lbPair);

  const transferHookXAccounts = getExtraAccountMetasForTransferHook(
    svm,
    tokenXMint
  );

  const transferHookYAccounts = getExtraAccountMetasForTransferHook(
    svm,
    tokenYMint
  );

  let remainingAccountsInfo: RemainingAccountsInfo = { slices: [] };

  if (transferHookXAccounts.length > 0) {
    remainingAccountsInfo.slices.push({
      accountsType: AccountsType.TransferHookX,
      length: transferHookXAccounts.length,
    });
  }

  if (transferHookYAccounts.length > 0) {
    remainingAccountsInfo.slices.push({
      accountsType: AccountsType.TransferHookY,
      length: transferHookYAccounts.length,
    });
  }

  const binArrays = getBinArrayAccountMetaByBinRange(
    lbPair,
    new BN(lowerBinId),
    new BN(upperBinId)
  );

  getOrCreateAtA(svm, user, tokenXMint, user.publicKey, tokenXProgram);
  getOrCreateAtA(svm, user, tokenYMint, user.publicKey, tokenYProgram);

  return program.methods
    .removeLiquidityByRange2(
      lowerBinId,
      upperBinId,
      5000, // bps
      remainingAccountsInfo
    )
    .accountsPartial({
      lbPair,
      binArrayBitmapExtension,
      position,
      reserveX,
      reserveY,
      tokenXMint,
      tokenYMint,
      tokenXProgram,
      tokenYProgram,
      sender: user.publicKey,
      userTokenX: userTokenX,
      userTokenY: userTokenY,
      memoProgram: MEMO_PROGRAM_ID,
    })
    .remainingAccounts([...transferHookXAccounts, ...transferHookYAccounts])
    .remainingAccounts(binArrays)
    .preInstructions([SET_COMPUTE_UNIT_LIMIT_IX])
    .transaction();
}

export function getDlmmRemainingAccounts(
  svm: LiteSVM,
  lbPair: PublicKey,
  user: PublicKey,
  userInputTokenAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  tokenXProgram: PublicKey,
  tokenYProgram: PublicKey
): {
  remainingAccounts: Array<{
    isSigner: boolean;
    isWritable: boolean;
    pubkey: PublicKey;
  }>;
  remainingAccountsInfo: RemainingAccountsInfo;
} {
  const lbPairState = getLbPairState(svm, lbPair);
  let [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);
  let binArrayBitmapExtensionState = svm.getAccount(binArrayBitmapExtension);
  if (!binArrayBitmapExtensionState) {
    binArrayBitmapExtension = new PublicKey(DLMM_PROGRAM_ID_LOCAL);
  }

  const transferHookXAccounts = getExtraAccountMetasForTransferHook(
    svm,
    lbPairState.tokenXMint
  );
  const transferHookYAccounts = getExtraAccountMetasForTransferHook(
    svm,
    lbPairState.tokenYMint
  );
  let remainingAccountsInfo: RemainingAccountsInfo = { slices: [] };

  if (transferHookXAccounts.length > 0) {
    remainingAccountsInfo.slices.push({
      accountsType: AccountsType.TransferHookX,
      length: transferHookXAccounts.length,
    });
  }

  if (transferHookYAccounts.length > 0) {
    remainingAccountsInfo.slices.push({
      accountsType: AccountsType.TransferHookY,
      length: transferHookYAccounts.length,
    });
  }

  const [oracle] = deriveOracle(lbPair);
  const remainingAccounts = [
    {
      isSigner: false,
      isWritable: true,
      pubkey: lbPair,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: binArrayBitmapExtension,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: lbPairState.reserveX,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: lbPairState.reserveY,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: userInputTokenAccount,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: userTokenOutAccount,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: lbPairState.tokenXMint,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: lbPairState.tokenYMint,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: oracle,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: new PublicKey(DLMM_PROGRAM_ID_LOCAL), // host fee option
    },
    {
      isSigner: true,
      isWritable: false,
      pubkey: user,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: tokenXProgram,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: tokenYProgram,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: MEMO_PROGRAM_ID,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveDlmmEventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: DLMM_PROGRAM_ID_LOCAL,
    },
  ];

  const binArrays = getBinArraysForSwap(svm, lbPair, true);

  const binArraysAccountMeta: AccountMeta[] = binArrays.map((pubkey) => {
    return {
      isSigner: false,
      isWritable: true,
      pubkey,
    };
  });

  remainingAccounts.push(
    ...[...transferHookXAccounts, ...transferHookYAccounts]
  );
  remainingAccounts.push(...binArraysAccountMeta);

  return {
    remainingAccounts,
    remainingAccountsInfo,
  };
}

export function getBinArraysForSwap(
  svm: LiteSVM,
  lbPair: PublicKey,
  swapForY: boolean,
  binArraysNeeded = 4
) {
  const [minBinArrayIdx, maxBinArrayIdx] = BIN_ARRAY_INDEX_BOUND;

  const binArrays: PublicKey[] = [];
  const lbPairState = getLbPairState(svm, lbPair);
  const activeBinArrayIdx = binIdToBinArrayIndex(new BN(lbPairState.activeId));

  const [bitmapExtension] = deriveBinArrayBitmapExtension(lbPair);

  const bitmapExtState = getBinArrayBitmapExtension(svm, bitmapExtension);

  let binArrayIdx = activeBinArrayIdx;

  while (binArrays.length < binArraysNeeded) {
    if (
      binArrayIdx.gt(new BN(maxBinArrayIdx)) ||
      binArrayIdx.lt(new BN(minBinArrayIdx))
    ) {
      break;
    }

    const nextBinArrayIndex = getNextBinArrayIndexWithLiquidity(
      binArrayIdx,
      lbPairState,
      swapForY,
      bitmapExtState
    );

    // Bin array exhausted
    if (!nextBinArrayIndex) {
      break;
    } else {
      const [binArray] = deriveBinArray(lbPair, nextBinArrayIndex);

      binArrays.push(binArray);
      if (swapForY) {
        binArrayIdx = nextBinArrayIndex.sub(new BN(1));
      } else {
        binArrayIdx = nextBinArrayIndex.add(new BN(1));
      }
    }
  }

  return binArrays;
}

export function isOverflowDefaultBinArrayBitmap(binArrayIndex: BN) {
  return (
    binArrayIndex.gt(BIN_ARRAY_BITMAP_SIZE.sub(new BN(1))) ||
    binArrayIndex.lt(BIN_ARRAY_BITMAP_SIZE.neg())
  );
}

export function getBitFromBinArrayIndexInBitmapExtension(
  binArrayIndex: BN,
  state: BinArrayBitmapExtension
) {
  // In extension, the range start with -513 and 512
  // Brain burst, let's just shift back to the actual index and calculate from there ...
  const idx = binArrayIndex.isNeg()
    ? binArrayIndex.add(new BN(1)).abs().sub(BIN_ARRAY_BITMAP_SIZE)
    : binArrayIndex.sub(BIN_ARRAY_BITMAP_SIZE);

  const bitmapOffset = idx.div(BIN_ARRAY_BITMAP_SIZE);

  const bitmap = binArrayIndex.isNeg()
    ? state.negativeBinArrayBitmap[bitmapOffset.toNumber()]
    : state.positiveBinArrayBitmap[bitmapOffset.toNumber()];

  const { div: offsetToU64InBitmap, mod: offsetToBit } = idx.divmod(new BN(64));

  // Each U512 have 8 u64
  const { mod: offsetToU64InChunkBitmap } = offsetToU64InBitmap.divmod(
    new BN(8)
  );

  if (!bitmap) {
    console.log(binArrayIndex.toString());
    console.log(bitmapOffset.toString());
  }

  const chunkedBitmap = bitmap[offsetToU64InChunkBitmap.toNumber()];
  return chunkedBitmap.testn(offsetToBit.toNumber());
}

export function getNextBinArrayIndexWithLiquidity(
  binArrayIndex: BN,
  pairState: LbPairState,
  swapForY: boolean,
  state: BinArrayBitmapExtension | null
): BN | null {
  const [minBinArrayIndex, maxBinArrayIndex] = BIN_ARRAY_INDEX_BOUND;
  const step = swapForY ? new BN(-1) : new BN(1);
  // Start search from the next bin array index
  while (true) {
    if (isOverflowDefaultBinArrayBitmap(binArrayIndex)) {
      // Search in extension
      if (state) {
        const isBitSet = getBitFromBinArrayIndexInBitmapExtension(
          binArrayIndex,
          state
        );
        if (isBitSet) {
          return binArrayIndex;
        }
      } else {
        break;
      }
    } else {
      // Because bitmap in pair state is continuous, -512 will be index 0. The add will shift to the actual index.
      const actualIdx = binArrayIndex.add(BIN_ARRAY_BITMAP_SIZE);
      // FullBitmap = U1024
      let { div: offsetInFullBitmap, mod: index } = actualIdx.divmod(
        new BN(64)
      );
      if (
        pairState.binArrayBitmap[offsetInFullBitmap.toNumber()].testn(
          index.toNumber()
        )
      ) {
        return binArrayIndex;
      }
    }
    binArrayIndex = binArrayIndex.add(step);
    if (
      binArrayIndex.gt(maxBinArrayIndex) ||
      binArrayIndex.lt(minBinArrayIndex)
    ) {
      break;
    }
  }
  return null;
}

export function getBinArrayBitmapExtensionState(
  svm: LiteSVM,
  program: Program<LbClmm>,
  binArrayBitmapExtension: PublicKey,
  nullable = false
) {
  const binArrayBitmapExtensionInfo = svm.getAccount(binArrayBitmapExtension);
  if (!binArrayBitmapExtensionInfo || !binArrayBitmapExtensionInfo.data.length)
    if (nullable) return null;
    else throw new Error("Invalid binArrayBitmapExtension");

  return program.coder.accounts.decode<BinArrayBitmapExtension>(
    "binArrayBitmapExtension",
    Buffer.from(binArrayBitmapExtensionInfo.data)
  );
}

export async function dlmmSwap(
  svm: LiteSVM,
  user: PublicKey,
  lbPair: PublicKey,
  amount: BN,
  swapForY: boolean,
  remainingAccounts?: AccountMeta[]
) {
  const program = createDlmmProgram();
  const lbPairState = getLbPairState(svm, lbPair);

  const tokenXProgram = svm.getAccount(lbPairState.tokenXMint).owner;
  const tokenYProgram = svm.getAccount(lbPairState.tokenYMint).owner;

  let remainingAccountsData;
  if (remainingAccounts) {
    remainingAccountsData = remainingAccounts;
  } else {
    const binArrays = getBinArraysForSwap(svm, lbPair, swapForY);

    const binArraysAccountMeta: AccountMeta[] = binArrays.map((pubkey) => ({
      isSigner: false,
      isWritable: true,
      pubkey,
    }));
    remainingAccountsData = binArraysAccountMeta;
  }

  const tokenXAta = getAssociatedTokenAddressSync(
    lbPairState.tokenXMint,
    user,
    false,
    tokenXProgram
  );

  const tokenYAta = getAssociatedTokenAddressSync(
    lbPairState.tokenYMint,
    user,
    false,
    tokenYProgram
  );

  const [inToken, outToken] = swapForY
    ? [tokenXAta, tokenYAta]
    : [tokenYAta, tokenXAta];

  const [binArrayBitmapExtension] = deriveBinArrayBitmapExtension(lbPair);

  const bitmapExtensionState = getBinArrayBitmapExtensionState(
    svm,
    program,
    binArrayBitmapExtension,
    true
  );

  return await program.methods
    .swap(amount, new BN(0))
    .accountsPartial({
      lbPair,
      binArrayBitmapExtension:
        bitmapExtensionState != null ? binArrayBitmapExtension : null,
      reserveX: lbPairState.reserveX,
      reserveY: lbPairState.reserveY,
      tokenXMint: lbPairState.tokenXMint,
      tokenYMint: lbPairState.tokenYMint,
      tokenXProgram,
      tokenYProgram,
      user,
      userTokenIn: inToken,
      userTokenOut: outToken,
      oracle: lbPairState.oracle,
      hostFeeIn: null,
    })
    .remainingAccounts(remainingAccountsData)
    .transaction();
}

export function getBinArrayIndexesByBinArange(
  lowerBinId: BN,
  upperBinId: BN
): BN[] {
  let indexes = [];

  let binArrayIndex = binIdToBinArrayIndex(lowerBinId);
  while (true) {
    indexes.push(binArrayIndex);
    const [binArrayLowerBinId, binArrayUpperBinId] =
      getBinArrayLowerUpperBinId(binArrayIndex);

    if (
      upperBinId.gte(binArrayLowerBinId) &&
      upperBinId.lte(binArrayUpperBinId)
    ) {
      break;
    } else {
      binArrayIndex = binArrayIndex.add(new BN(1));
    }
  }
  return indexes;
}

export function getBinArrayState(
  svm: LiteSVM,
  binArray: PublicKey,
  nullable = false
) {
  const program = createDlmmProgram();
  const binArrayInfo = svm.getAccount(binArray);
  if (!binArrayInfo || !binArrayInfo.data.length)
    if (nullable) return null;
    else throw new Error("Invalid binArray");

  return program.coder.accounts.decode<BinArray>(
    "binArray",
    Buffer.from(binArrayInfo.data)
  );
}

export function getPositionTotalLiquidityAllBin(
  svm: LiteSVM,
  position: PublicKey
): Number[][] {
  const positionState = fetchAndDecodeDynamicPosition(svm, position);

  const lbPairState = getLbPairState(svm, positionState.globalData.lbPair);
  let liquidities = [];

  let binArrayIndexes = getBinArrayIndexesByBinArange(
    new BN(positionState.globalData.lowerBinId),
    new BN(positionState.globalData.upperBinId)
  );

  const binArrays = binArrayIndexes.map((index) => {
    let [key, _] = deriveBinArray(positionState.globalData.lbPair, index);
    return getBinArrayState(svm, key);
  });

  for (
    let binPosition = positionState.globalData.lowerBinId;
    binPosition <= positionState.globalData.upperBinId;
    binPosition++
  ) {
    const liquidity = getPositionLiquidityByBin(
      positionState,
      binArrays,
      binPosition,
      lbPairState.binStep
    );

    liquidities.push([binPosition, liquidity.toNumber()]);
  }

  return liquidities;
}

export type UserRewardInfo = IdlTypes<LbClmm>["userRewardInfo"];
export type FeeInfo = IdlTypes<LbClmm>["feeInfo"];
export type PositionBinData = {
  liquidityShare: BN;
  rewardInfo: UserRewardInfo;
  feeInfo: FeeInfo;
};
export type BinArray = IdlAccounts<LbClmm>["binArray"];

export type DynamicPosition = {
  globalData: {
    lbPair: PublicKey;
    owner: PublicKey;
    lowerBinId: number;
    upperBinId: number;
    lastUpdatedAt: BN;
    totalClaimedFeeXAmount: BN;
    totalClaimedFeeYAmount: BN;
    totalClaimedRewards: BN[];
    operator: PublicKey;
    lockReleasePoint: BN;
    padding0: number[];
    feeOwner: PublicKey;
    binCount: BN;
    length: BN;
    reserved: number[];
  };
  positionBinData: PositionBinData[];
};

export type Bin = IdlTypes<LbClmm>["bin"];

export function getBinIndexInArray(binId: BN) {
  const binArrayIndex = binIdToBinArrayIndex(binId);
  const [lowerBinId, upperBinId] = getBinArrayLowerUpperBinId(binArrayIndex);
  let index: BN;
  if (!binId.isNeg()) {
    index = binId.sub(lowerBinId);
  } else {
    index = MAX_BIN_PER_ARRAY.sub(upperBinId.sub(binId)).sub(new BN(1));
  }
  return index;
}

export function getPositionLiquidityByBin(
  position: DynamicPosition,
  binArrays: BinArray[],
  binId: number,
  binStep: number
) {
  if (
    position.globalData.lowerBinId <= binId &&
    position.globalData.upperBinId >= binId
  ) {
    const binArrayIndex = binIdToBinArrayIndex(new BN(binId));
    for (const binArray of binArrays) {
      if (binArrayIndex.eq(binArray.index)) {
        const idx = binId - position.globalData.lowerBinId;

        const share = position.positionBinData[idx].liquidityShare;

        const bin = binArray.bins[
          getBinIndexInArray(new BN(binId)).toNumber()
        ] as Bin;

        const supply = bin.liquiditySupply;

        if (supply.isZero()) {
          return share;
        }

        const x = share.mul(bin.amountX).div(supply);
        const y = share.mul(bin.amountY).div(supply);

        const price = getPriceFromBinId(new BN(binId), new BN(binStep));
        const liquidity = getLiquidity(x, y, price);

        return liquidity;
      }
    }
  }

  return null;
}

export function getPriceFromBinId(id: BN, binStep: BN) {
  const bps = binStep.shln(64).div(BASIS_POINT_MAX);
  const base = new BN(1).shln(64).add(bps);

  return pow(base, id);
}

export function getLiquidity(amountX: BN, amountY: BN, price: BN) {
  const px = amountX.mul(price);
  const y = amountY.shln(64);

  return px.add(y).shrn(64);
}

export function getDlmmPositionState(svm: LiteSVM, position: PublicKey) {
  const program = createDlmmProgram();
  const account = svm.getAccount(position);

  return program.coder.accounts.decode("positionV2", Buffer.from(account.data));
}

export function fetchAndDecodeDynamicPosition(
  svm: LiteSVM,
  position: PublicKey
): DynamicPosition {
  const positionState = getDlmmPositionState(svm, position);

  const positionData = svm.getAccount(position);

  const remainingBytes = positionData.data.subarray(8 + 8112);

  const positionWidth = Math.max(
    positionState.upperBinId - positionState.lowerBinId + 1,
    DEFAULT_BIN_PER_POSITION.toNumber()
  );

  const binCount = positionState.upperBinId - positionState.lowerBinId + 1;

  const outerBinCount =
    binCount > DEFAULT_BIN_PER_POSITION.toNumber()
      ? binCount - DEFAULT_BIN_PER_POSITION.toNumber()
      : 0;

  const positionBinDataSchema = {
    array: {
      type: {
        struct: {
          liquidityShare: "u128",
          rewardInfo: {
            struct: {
              rewardPerTokenCompletes: {
                array: {
                  type: "u128",
                  len: 2,
                },
              },
              rewardPendings: {
                array: {
                  type: "u64",
                  len: 2,
                },
              },
            },
          },
          feeInfo: {
            struct: {
              feeXPerTokenComplete: "u128",
              feeYPerTokenComplete: "u128",
              feeXPending: "u64",
              feeYPending: "u64",
            },
          },
        },
      },
      len: outerBinCount,
    },
  };

  // @ts-ignore
  // TODO: How to fix this? Somehow it decode it to bigint ...
  let extendedPositionBinData: PositionBinData[] =
    outerBinCount > 0
      ? borsh.deserialize(positionBinDataSchema, remainingBytes)
      : [];

  // Map back to BN ...
  extendedPositionBinData = extendedPositionBinData.map((b) => {
    return {
      liquidityShare: new BN(b.liquidityShare.toString()),
      rewardInfo: {
        rewardPendings: b.rewardInfo.rewardPendings.map(
          (r) => new BN(r.toString())
        ),
        rewardPerTokenCompletes: b.rewardInfo.rewardPerTokenCompletes.map(
          (r) => new BN(r.toString())
        ),
      },
      feeInfo: {
        feeXPending: new BN(b.feeInfo.feeXPending.toString()),
        feeYPending: new BN(b.feeInfo.feeYPending.toString()),
        feeXPerTokenComplete: new BN(b.feeInfo.feeXPerTokenComplete.toString()),
        feeYPerTokenComplete: new BN(b.feeInfo.feeYPerTokenComplete.toString()),
      },
    };
  });

  const innerPositionBinData: PositionBinData[] = [];

  for (let i = 0; i < DEFAULT_BIN_PER_POSITION.toNumber(); i++) {
    innerPositionBinData.push({
      liquidityShare: positionState.liquidityShares[i],
      rewardInfo: positionState.rewardInfos[i],
      feeInfo: positionState.feeInfos[i],
    });
  }

  const positionBinData = innerPositionBinData.concat(extendedPositionBinData);

  return {
    globalData: {
      lbPair: positionState.lbPair,
      owner: positionState.owner,
      lowerBinId: positionState.lowerBinId,
      upperBinId: positionState.upperBinId,
      lastUpdatedAt: positionState.lastUpdatedAt,
      totalClaimedFeeXAmount: positionState.totalClaimedFeeXAmount,
      totalClaimedFeeYAmount: positionState.totalClaimedFeeYAmount,
      totalClaimedRewards: positionState.totalClaimedRewards,
      operator: positionState.operator,
      length: new BN(positionWidth),
      binCount: new BN(binCount),
      lockReleasePoint: positionState.lockReleasePoint,
      feeOwner: positionState.feeOwner,
      padding0: [],
      reserved: [],
    },
    positionBinData,
  };

  //
}
