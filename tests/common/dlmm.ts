import {
  AnchorProvider,
  BN,
  IdlAccounts,
  IdlTypes,
  Program,
  Wallet,
} from "@coral-xyz/anchor";

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
import { deriveZapAuthorityAddress } from "./zap";
import {
  deriveBinArray,
  deriveBinArrayBitmapExtension,
  deriveDlmmEventAuthority,
  deriveLbPermissionless2,
  deriveOracle,
  derivePresetParameter2,
  deriveReserve,
  deriveTokenBadge,
  getBinArraysForModifyLiquidity,
} from "./pda";
import { getExtraAccountMetasForTransferHook } from "./transferHook/transferHookUtils";
import { min } from "bn.js";

export type PresetParameter = Omit<IdlAccounts<LbClmm>["presetParameter"], "">;
export type BinLiquidityDistribution =
  IdlTypes<LbClmm>["binLiquidityDistribution"];

export type BinArrayBitmapExtension =
  IdlTypes<LbClmm>["binArrayBitmapExtension"];
export type RemainingAccountsInfo = IdlTypes<LbClmm>["remainingAccountsInfo"];

export type LbPairState = IdlAccounts<LbClmm>["lbPair"];
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
  "LbVRzDTvBDEcrthxfZ4RL6yiq3uZw8bS6MwtdY6UhFQ"
);

export const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
);

const CONSTANTS = Object.entries(DlmmIDL.constants);
export const BIN_ARRAY_BITMAP_SIZE = new BN(
  CONSTANTS.find(([k, v]) => v.name == "BIN_ARRAY_BITMAP_SIZE")[1].value
);
export const DEFAULT_BITMAP_RANGE = [
  BIN_ARRAY_BITMAP_SIZE.neg(),
  BIN_ARRAY_BITMAP_SIZE.sub(new BN(1)),
];

export const MAX_BIN_PER_ARRAY = new BN(
  CONSTANTS.find(([k, v]) => v.name == "MAX_BIN_PER_ARRAY")[1].value
);

export const MAX_BIN_PER_POSITION = new BN(
  CONSTANTS.find(([k, v]) => v.name == "MAX_BIN_PER_POSITION")[1].value
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
        .accounts({
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
    .accounts({
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
    tx.recentBlockhash = svm.latestBlockhash();
    tx.sign(payer);
    const result = svm.sendTransaction(tx);
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
  const positionWidth = width ? width : MAX_BIN_PER_POSITION.toNumber();
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
  inputTokenAccount: PublicKey,
  outputTokenAccount: PublicKey,
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
      pubkey: inputTokenAccount,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: outputTokenAccount,
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
      isSigner: false,
      isWritable: false,
      pubkey: deriveZapAuthorityAddress(),
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
  
  

  return { remainingAccounts, remainingAccountsInfo };
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
