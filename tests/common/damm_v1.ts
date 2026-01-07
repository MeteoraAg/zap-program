import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  unpackMint,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  ComputeBudgetProgram,
  Connection,
  clusterApiUrl,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  FailedTransactionMetadata,
  LiteSVM,
  SimulatedTransactionInfo,
  TransactionMetadata,
} from "litesvm";
import { expect } from "chai";
import type { Amm } from "./idl/damm_v1";
import DAMMV1IDL from "../../idls/damm_v1.json";
import {
  AnchorProvider,
  BN,
  Coder,
  EventParser,
  IdlAccounts,
  Program,
  Wallet,
} from "@coral-xyz/anchor";
import { getVaultAccount, PUBLIC_KEY, vaultProgram } from "./vault";
import { getOrCreateAtA, sendTransaction } from "./utils";

export function createDammV1Program() {
  const wallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(
    new Connection(clusterApiUrl("devnet")),
    wallet,
    {}
  );
  const program = new Program<Amm>(DAMMV1IDL as Amm, provider);
  return program;
}

export const dammV1Program = createDammV1Program();

export const DAMM_V1_PROGRAM_ID = new PublicKey(DAMMV1IDL.address);

export const DAMM_V1_SWAP_DISC = [248, 198, 158, 145, 225, 117, 135, 200];

const FEE_OWNER = new PublicKey("6WaLrrRfReGKBYUSkmx2K6AuT21ida4j8at2SUiZdXu8");

export const getRemoveLiquidityOutAmount = async (
  svm: LiteSVM,
  params: {
    pool: PublicKey;
    withdrawAmount: BN;
    userKeypair: Keypair;
  }
): Promise<{
  aOutAmount: BN;
  bOutAmount: BN;
}> => {
  const { pool, withdrawAmount, userKeypair } = params;
  const poolAccount = getDammV1Pool(svm, pool);

  const aVault = getVaultAccount(svm, poolAccount.aVault);
  const bVault = getVaultAccount(svm, poolAccount.bVault);

  const [userPoolLp, userTokenA, userTokenB] = await Promise.all([
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.lpMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.tokenAMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.tokenBMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
  ]);

  const transaction = await dammV1Program.methods
    .removeBalanceLiquidity(withdrawAmount, new BN(0), new BN(0))
    .accountsPartial({
      aTokenVault: aVault.tokenVault,
      bTokenVault: bVault.tokenVault,
      aVault: poolAccount.aVault,
      bVault: poolAccount.bVault,
      pool,
      user: userKeypair.publicKey,
      userAToken: userTokenA,
      userBToken: userTokenB,
      aVaultLp: poolAccount.aVaultLp,
      bVaultLp: poolAccount.bVaultLp,
      aVaultLpMint: aVault.lpMint,
      bVaultLpMint: bVault.lpMint,
      lpMint: poolAccount.lpMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      vaultProgram: vaultProgram.programId,
      userPoolLp: userPoolLp,
    })
    .transaction();

  transaction.feePayer = userKeypair.publicKey;
  transaction.recentBlockhash = svm.latestBlockhash();
  transaction.sign(userKeypair);
  const simulation = svm.simulateTransaction(transaction);

  expect(simulation).instanceOf(SimulatedTransactionInfo);

  const events = parseAnchorEvents(
    simulation.meta().logs(),
    dammV1Program.programId,
    dammV1Program.coder
  );

  const { tokenAOutAmount, tokenBOutAmount } = events.find(
    (e) => e.name === "removeLiquidity"
  ).data;

  return {
    aOutAmount: tokenAOutAmount,
    bOutAmount: tokenBOutAmount,
  };
};

export const getProtocolFeeTokenPDA = (
  tokenA: PublicKey,
  tokenB: PublicKey,
  poolPubkey: PublicKey,
  ammProgram: Program<Amm>
) => {
  const feeTokenA = PublicKey.findProgramAddressSync(
    [Buffer.from("fee"), tokenA.toBuffer(), poolPubkey.toBuffer()],
    ammProgram.programId
  )[0];

  const feeTokenB = PublicKey.findProgramAddressSync(
    [Buffer.from("fee"), tokenB.toBuffer(), poolPubkey.toBuffer()],
    ammProgram.programId
  )[0];

  return [feeTokenA, feeTokenB];
};

export const getPoolPdas = (
  poolPubkey: PublicKey,
  aVault: PublicKey,
  bVault: PublicKey,
  ammProgram: Program<Amm>
) => {
  const aVaultLpPda = PublicKey.findProgramAddressSync(
    [aVault.toBuffer(), poolPubkey.toBuffer()],
    ammProgram.programId
  );
  const bVaultLpPda = PublicKey.findProgramAddressSync(
    [bVault.toBuffer(), poolPubkey.toBuffer()],
    ammProgram.programId
  );

  return {
    aVaultLpPda,
    bVaultLpPda,
  };
};

export const METAPLEX_PROGRAM = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

export const deriveMetadataPda = (mint: PublicKey) => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METAPLEX_PROGRAM.toBuffer(), mint.toBuffer()],
    METAPLEX_PROGRAM
  );
};

export interface ConstantProduct {
  constantProduct: {};
}
export interface Stable {
  stable: {
    amp: BN;
    tokenMultiplier: TokenMultiplier;
    depeg: Depeg;
    lastAmpUpdatedTimestamp: BN;
  };
}

export interface DepegNone {
  none: {};
}

export interface DepegMarinade {
  marinade: {};
}

export interface DepegLido {
  lido: {};
}

export interface TokenMultiplier {
  tokenAMultiplier: BN;
  tokenBMultiplier: BN;
  precisionFactor: number;
}

export interface Depeg {
  baseVirtualPrice: BN;
  baseCacheUpdated: BN;
  depegType: DepegNone | DepegLido | DepegMarinade;
}

export const DepegType = {
  none: (): DepegNone => {
    return {
      none: {},
    };
  },
  marinade: (): DepegMarinade => {
    return {
      marinade: {},
    };
  },
  lido: (): DepegLido => {
    return {
      lido: {},
    };
  },
};

export const CurveType = {
  stable: (
    amp: BN,
    tokenMultiplier: TokenMultiplier,
    depeg: Depeg,
    lastAmpUpdatedTimestamp: BN
  ): Stable => {
    return {
      stable: {
        amp,
        tokenMultiplier,
        depeg,
        lastAmpUpdatedTimestamp,
      },
    };
  },

  constantProduct: (): ConstantProduct => {
    return {
      constantProduct: {},
    };
  },
};

export function encodeCurveType(curve: Stable | ConstantProduct) {
  if (curve["constantProduct"]) {
    return 0;
  } else if (curve["stable"]) {
    return 1;
  } else {
    throw new Error("Unknown curve type");
  }
}

export function getDammV1Pool(svm: LiteSVM, pool: PublicKey) {
  const poolAccountBytes = svm.getAccount(pool);
  if (poolAccountBytes)
    return dammV1Program.coder.accounts.decode(
      "pool",
      Buffer.from(poolAccountBytes.data)
    ) as IdlAccounts<Amm>["pool"];
}

export function parseAnchorEvents(
  logs: string[],
  programId: PublicKey,
  coder: Coder
) {
  const eventParser = new EventParser(programId, coder);
  const events: any[] = [];

  try {
    const parsedEvents = eventParser.parseLogs(logs);
    events.push(...parsedEvents);
  } catch (e) {
    // Event parsing may fail if no events in logs
    console.warn("Failed to parse events:", e);
  }

  return events;
}

export const getPoolInfo = async (svm: LiteSVM, pool: PublicKey) => {
  const poolAccount = getDammV1Pool(svm, pool);
  const aVault = getVaultAccount(svm, poolAccount.aVault);
  const bVault = getVaultAccount(svm, poolAccount.bVault);

  const transaction = await dammV1Program.methods
    .getPoolInfo()
    .accountsPartial({
      aVault: poolAccount.aVault,
      aVaultLp: poolAccount.aVaultLp,
      aVaultLpMint: aVault.lpMint,
      bVault: poolAccount.bVault,
      bVaultLp: poolAccount.bVaultLp,
      bVaultLpMint: bVault.lpMint,
      lpMint: poolAccount.lpMint,
      pool,
    })
    .transaction();

  const payer = Keypair.generate();
  svm.airdrop(payer.publicKey, BigInt(LAMPORTS_PER_SOL));
  transaction.feePayer = payer.publicKey;
  transaction.recentBlockhash = svm.latestBlockhash();
  transaction.sign(payer);
  const simulation = svm.simulateTransaction(transaction);

  expect(simulation).instanceOf(SimulatedTransactionInfo);

  const events = parseAnchorEvents(
    simulation.meta().logs(),
    dammV1Program.programId,
    dammV1Program.coder
  );

  const { tokenAAmount, tokenBAmount, virtualPrice } = events.find(
    (e) => e.name === "poolInfo"
  ).data;

  return {
    virtualPrice,
    tokenAAmount,
    tokenBAmount,
  } as {
    tokenAAmount: BN;
    tokenBAmount: BN;
    virtualPrice: number;
  };
};

export const getShareByAmount = (
  depositAmount: BN,
  lpTotalSupply: BN,
  totalTokenAmount: BN
) => {
  return depositAmount.mul(lpTotalSupply).div(totalTokenAmount);
};

export const getQuote = (amountA: BN, reserveA: BN, reserveB: BN) => {
  return reserveB.mul(amountA).div(reserveA);
};

export const getAddLiquidityInputAmount = async (
  svm: LiteSVM,
  getAddLiquidityInputAmountParams: {
    pool: PublicKey;
    depositAmount: BN;
    depositTokenMint: PublicKey;
  }
): Promise<{
  lpAmount: BN;
  aDepositAmount: BN;
  bDepositAmount: BN;
}> => {
  const { depositAmount, depositTokenMint, pool } =
    getAddLiquidityInputAmountParams;

  const poolAccount = await getDammV1Pool(svm, pool);

  const lpMintAccount = svm.getAccount(poolAccount.lpMint);
  const lpMintAccountWithBuffer = {
    ...lpMintAccount,
    data: Buffer.from(lpMintAccount.data),
  };
  const lpMint = unpackMint(poolAccount.lpMint, lpMintAccountWithBuffer);
  const poolLpSupply = lpMint.supply;

  const { tokenAAmount, tokenBAmount } = await getPoolInfo(svm, pool);

  let lpAmount: BN;
  let aDepositAmount: BN;
  let bDepositAmount: BN;

  if (depositTokenMint.equals(poolAccount.tokenAMint)) {
    lpAmount = getShareByAmount(
      depositAmount,
      new BN(poolLpSupply),
      tokenAAmount
    );
    aDepositAmount = depositAmount;
    bDepositAmount = getQuote(aDepositAmount, tokenAAmount, tokenBAmount);
  } else {
    lpAmount = getShareByAmount(
      depositAmount,
      new BN(poolLpSupply),
      tokenBAmount
    );
    bDepositAmount = depositAmount;
    aDepositAmount = getQuote(bDepositAmount, tokenBAmount, tokenAAmount);
  }

  return {
    lpAmount,
    aDepositAmount,
    bDepositAmount,
  };
};

export const addBalanceLiquidity = async (
  svm: LiteSVM,
  addLiquidityParams: {
    pool: PublicKey;
    lpAmount: BN;
    maxATokenAmount: BN;
    maxBTokenAmount: BN;
    userKeypair: Keypair;
  }
) => {
  const { pool, lpAmount, maxATokenAmount, maxBTokenAmount, userKeypair } =
    addLiquidityParams;

  const poolAccount = getDammV1Pool(svm, pool);

  const [userPoolLp, userTokenA, userTokenB] = await Promise.all([
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.lpMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.tokenAMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.tokenBMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
  ]);
  const account = svm.getAccount(userPoolLp);
  console.log("userPoolLpAmount in", userPoolLp.toBase58());

  const aVault = getVaultAccount(svm, poolAccount.aVault);
  const bVault = getVaultAccount(svm, poolAccount.bVault);

  const transaction = await dammV1Program.methods
    .addBalanceLiquidity(lpAmount, maxATokenAmount, maxBTokenAmount)
    .accountsPartial({
      aTokenVault: aVault.tokenVault,
      bTokenVault: bVault.tokenVault,
      aVault: poolAccount.aVault,
      bVault: poolAccount.bVault,
      pool,
      user: userKeypair.publicKey,
      userAToken: userTokenA,
      userBToken: userTokenB,
      aVaultLp: poolAccount.aVaultLp,
      bVaultLp: poolAccount.bVaultLp,
      aVaultLpMint: aVault.lpMint,
      bVaultLpMint: bVault.lpMint,
      lpMint: poolAccount.lpMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      vaultProgram: vaultProgram.programId,
      userPoolLp,
    })
    .transaction();

  sendTransaction(svm, transaction, [userKeypair]);
};

export const removeLiquidity = async (
  svm: LiteSVM,
  removeLiquidityParams: {
    pool: PublicKey;
    withdrawAmount: BN;
    minAOutAmount: BN;
    minBOutAmount: BN;
    userKeypair: Keypair;
  }
) => {
  const { minAOutAmount, minBOutAmount, pool, userKeypair, withdrawAmount } =
    removeLiquidityParams;

  const poolAccount = getDammV1Pool(svm, pool);

  const aVault = getVaultAccount(svm, poolAccount.aVault);
  const bVault = getVaultAccount(svm, poolAccount.bVault);

  const [userPoolLp, userTokenA, userTokenB] = await Promise.all([
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.lpMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.tokenAMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      userKeypair,
      poolAccount.tokenBMint,
      userKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
  ]);

  return await dammV1Program.methods
    .removeBalanceLiquidity(withdrawAmount, minAOutAmount, minBOutAmount)
    .accountsPartial({
      aTokenVault: aVault.tokenVault,
      bTokenVault: bVault.tokenVault,
      aVault: poolAccount.aVault,
      bVault: poolAccount.bVault,
      pool,
      user: userKeypair.publicKey,
      userAToken: userTokenA,
      userBToken: userTokenB,
      aVaultLp: poolAccount.aVaultLp,
      bVaultLp: poolAccount.bVaultLp,
      aVaultLpMint: aVault.lpMint,
      bVaultLpMint: bVault.lpMint,
      lpMint: poolAccount.lpMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      vaultProgram: vaultProgram.programId,
      userPoolLp: userPoolLp,
    })
    .transaction();
};

export function getDammV1RemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userInputTokenAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  protocolTokenFee: PublicKey
) {
  const poolState = getDammV1Pool(svm, pool);
  const aVault = getVaultAccount(svm, poolState.aVault);
  const bVault = getVaultAccount(svm, poolState.bVault);
  const remainingAccounts: {
    isSigner: boolean;
    isWritable: boolean;
    pubkey: PublicKey;
  }[] = [
    {
      isSigner: false,
      isWritable: true,
      pubkey: pool,
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
      isWritable: true,
      pubkey: poolState.aVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: poolState.bVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: aVault.tokenVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: bVault.tokenVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: aVault.lpMint,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: bVault.lpMint,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: poolState.aVaultLp,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: poolState.bVaultLp,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: protocolTokenFee,
    },
    {
      isSigner: true,
      isWritable: false,
      pubkey: user,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: vaultProgram.programId,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: TOKEN_PROGRAM_ID,
    },
  ];

  return remainingAccounts;
}

export const initializePermissionedPool = async (
  svm,
  param: {
    aVault: PublicKey;
    bVault: PublicKey;
    aDepositAmount: BN;
    bDepositAmount: BN;
    curve: ConstantProduct | Stable;
    adminKeypair: Keypair;
    poolKeypair: Keypair;
    adminLpMint?: PublicKey;
  }
): Promise<PublicKey> => {
  const {
    aVault,
    bVault,
    adminKeypair,
    curve,
    aDepositAmount,
    bDepositAmount,
    poolKeypair,
  } = param;

  const [poolLpMint, _poolLpMintBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp_mint"), poolKeypair.publicKey.toBuffer()],
    dammV1Program.programId
  );

  const [mintMetadata, _mintMetadataBump] = deriveMetadataPda(poolLpMint);

  const aVaultAccount = getVaultAccount(svm, aVault);
  const bVaultAccount = getVaultAccount(svm, bVault);

  const { aVaultLpPda, bVaultLpPda } = getPoolPdas(
    poolKeypair.publicKey,
    aVault,
    bVault,
    dammV1Program
  );

  const adminPoolLp = getAssociatedTokenAddressSync(
    poolLpMint,
    adminKeypair.publicKey
  );

  const [adminTokenA, adminTokenB] = await Promise.all([
    getOrCreateAtA(
      svm,
      adminKeypair,
      aVaultAccount.tokenMint,
      adminKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
    getOrCreateAtA(
      svm,
      adminKeypair,
      bVaultAccount.tokenMint,
      adminKeypair.publicKey,
      TOKEN_PROGRAM_ID
    ),
  ]);

  const [protocolTokenAFee, protocolTokenBFee] = getProtocolFeeTokenPDA(
    aVaultAccount.tokenMint,
    bVaultAccount.tokenMint,
    poolKeypair.publicKey,
    dammV1Program
  );

  const setComputeUnitLimitIx = ComputeBudgetProgram.setComputeUnitLimit({
    units: 1_400_000,
  });

  const bootstrapLiquidityIx = await dammV1Program.methods
    .bootstrapLiquidity(aDepositAmount, bDepositAmount)
    .accountsPartial({
      pool: poolKeypair.publicKey,
      aVault,
      bVault,
      aVaultLpMint: aVaultAccount.lpMint,
      bVaultLpMint: bVaultAccount.lpMint,
      aVaultLp: aVaultLpPda[PUBLIC_KEY],
      bVaultLp: bVaultLpPda[PUBLIC_KEY],
      lpMint: poolLpMint,
      userPoolLp: adminPoolLp,
      user: adminKeypair.publicKey,
      userAToken: adminTokenA,
      userBToken: adminTokenB,
      aTokenVault: aVaultAccount.tokenVault,
      bTokenVault: bVaultAccount.tokenVault,
      vaultProgram: vaultProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

  const transaction = await dammV1Program.methods
    .initializePermissionedPool(curve as any)
    .accountsPartial({
      pool: poolKeypair.publicKey,
      tokenAMint: aVaultAccount.tokenMint,
      tokenBMint: bVaultAccount.tokenMint,
      aVault,
      bVault,
      aVaultLpMint: aVaultAccount.lpMint,
      bVaultLpMint: bVaultAccount.lpMint,
      aVaultLp: aVaultLpPda[PUBLIC_KEY],
      bVaultLp: bVaultLpPda[PUBLIC_KEY],
      lpMint: poolLpMint,
      adminTokenA,
      adminTokenB,
      protocolTokenAFee,
      protocolTokenBFee,
      adminPoolLp,
      admin: adminKeypair.publicKey,
      feeOwner: FEE_OWNER,
      rent: SYSVAR_RENT_PUBKEY,
      metadataProgram: METAPLEX_PROGRAM,
      mintMetadata,
      vaultProgram: vaultProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .preInstructions([setComputeUnitLimitIx])
    .postInstructions([bootstrapLiquidityIx])
    .transaction();

  transaction.recentBlockhash = svm.latestBlockhash();
  transaction.sign(adminKeypair, poolKeypair);
  const result = svm.sendTransaction(transaction);
  if (result instanceof FailedTransactionMetadata) {
    console.log(result.toString());
    // console.log(result.meta().logs());
  }
  expect(result).instanceOf(TransactionMetadata);

  return poolKeypair.publicKey;
};
