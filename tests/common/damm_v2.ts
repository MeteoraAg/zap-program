import {
  AnchorProvider,
  BN,
  IdlAccounts,
  Program,
  Wallet,
} from "@coral-xyz/anchor";

import { CpAmm } from "./idl/damm_v2";
import CpAmmIDL from "../../idls/damm_v2.json";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { LiteSVM, TransactionMetadata } from "litesvm";
import {
  AccountLayout,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  INIT_PRICE,
  LIQUIDITY_DELTA,
  LIQUIDITY_DELTA_2,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  U64_MAX,
} from "./utils";
import { expect } from "chai";
import {
  deriveDammV2CustomizablePoolAddress,
  deriveDammV2EventAuthority,
  deriveDammV2PoolAuthority,
  deriveDammV2PositionAddress,
  deriveDammV2PositionNftAccount,
  deriveDammV2TokenVaultAddress,
  getDammV2Pool,
  getDammV2Position,
} from "./pda";

export const DAMM_V2_PROGRAM_ID = new PublicKey(CpAmmIDL.address);

export const DAMM_V2_SWAP_DISC = [248, 198, 158, 145, 225, 117, 135, 200]

export type Pool = IdlAccounts<CpAmm>["pool"];
export type Position = IdlAccounts<CpAmm>["position"];

export function createDammV2Program() {
  const wallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(
    new Connection(clusterApiUrl("devnet")),
    wallet,
    {}
  );
  const program = new Program<CpAmm>(CpAmmIDL as CpAmm, provider);
  return program;
}

export function getDammV2RemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userInputTokenAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  tokenAProgram = TOKEN_PROGRAM_ID,
  tokenBProgram = TOKEN_PROGRAM_ID
): Array<{
  isSigner: boolean;
  isWritable: boolean;
  pubkey: PublicKey;
}> {
  const poolState = getDammV2Pool(svm, pool);
  const remainingAccounts = [
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveDammV2PoolAuthority(),
    },
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
      pubkey: poolState.tokenAVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: poolState.tokenBVault,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: poolState.tokenAMint,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: poolState.tokenBMint,
    },
    {
      isSigner: true,
      isWritable: false,
      pubkey: user,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: tokenAProgram,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: tokenBProgram,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: DAMM_V2_PROGRAM_ID,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveDammV2EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: DAMM_V2_PROGRAM_ID,
    },
  ];

  return remainingAccounts;
}

export async function createDammV2Pool(
  svm: LiteSVM,
  creator: Keypair,
  tokenAMint: PublicKey,
  tokenBMint: PublicKey
): Promise<PublicKey> {
  const program = createDammV2Program();

  const poolAuthority = deriveDammV2PoolAuthority();
  const pool = deriveDammV2CustomizablePoolAddress(tokenAMint, tokenBMint);

  const positionNftKP = Keypair.generate();
  const position = deriveDammV2PositionAddress(positionNftKP.publicKey);
  const positionNftAccount = deriveDammV2PositionNftAccount(
    positionNftKP.publicKey
  );

  const tokenAVault = deriveDammV2TokenVaultAddress(tokenAMint, pool);
  const tokenBVault = deriveDammV2TokenVaultAddress(tokenBMint, pool);

  const payerTokenA = getAssociatedTokenAddressSync(
    tokenAMint,
    creator.publicKey,
    true,
    TOKEN_PROGRAM_ID
  );
  const payerTokenB = getAssociatedTokenAddressSync(
    tokenBMint,
    creator.publicKey,
    true,
    TOKEN_PROGRAM_ID
  );

  const transaction = await program.methods
    .initializeCustomizablePool({
      poolFees: {
        baseFee: {
          cliffFeeNumerator: new BN(2_500_000),
          numberOfPeriod: 0,
          reductionFactor: new BN(0),
          periodFrequency: new BN(0),
          feeSchedulerMode: 0,
        },
        protocolFeePercent: 20,
        partnerFeePercent: 0,
        referralFeePercent: 20,
        dynamicFee: null,
      },
      sqrtMinPrice: MIN_SQRT_PRICE,
      sqrtMaxPrice: MAX_SQRT_PRICE,
      hasAlphaVault: false,
      liquidity: LIQUIDITY_DELTA,
      sqrtPrice: INIT_PRICE,
      activationType: 0,
      collectFeeMode: 0,
      activationPoint: null,
    })
    .accountsPartial({
      creator: creator.publicKey,
      positionNftAccount,
      positionNftMint: positionNftKP.publicKey,
      payer: creator.publicKey,
      poolAuthority,
      pool,
      position,
      tokenAMint,
      tokenBMint,
      tokenAVault,
      tokenBVault,
      payerTokenA,
      payerTokenB,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      tokenAProgram: TOKEN_PROGRAM_ID,
      tokenBProgram: TOKEN_PROGRAM_ID,
    })
    .transaction();
  transaction.recentBlockhash = svm.latestBlockhash();
  transaction.sign(creator, positionNftKP);

  const result = svm.sendTransaction(transaction);
  expect(result).instanceOf(TransactionMetadata);

  const tokenAVaultData = svm.getAccount(tokenAVault).data;
  const tokenBVaultData = svm.getAccount(tokenBVault).data;
  const vaultABalance = Number(AccountLayout.decode(tokenAVaultData).amount);

  const vaultBBalance = Number(AccountLayout.decode(tokenBVaultData).amount);

  expect(vaultABalance).greaterThan(0);
  expect(vaultBBalance).greaterThan(0);

  return pool;
}

export async function createPositionAndAddLiquidity(
  svm: LiteSVM,
  user: Keypair,
  pool: PublicKey
): Promise<PublicKey> {
  const program = createDammV2Program();

  const positionNftKP = Keypair.generate();
  const position = deriveDammV2PositionAddress(positionNftKP.publicKey);
  const poolAuthority = deriveDammV2PoolAuthority();
  const positionNftAccount = deriveDammV2PositionNftAccount(
    positionNftKP.publicKey
  );

  const poolState = getDammV2Pool(svm, pool);

  const tokenAAccount = getAssociatedTokenAddressSync(
    poolState.tokenAMint,
    user.publicKey,
    true,
    TOKEN_PROGRAM_ID
  );
  const tokenBAccount = getAssociatedTokenAddressSync(
    poolState.tokenBMint,
    user.publicKey,
    true,
    TOKEN_PROGRAM_ID
  );

  const createPositionTx = await program.methods
    .createPosition()
    .accountsPartial({
      owner: user.publicKey,
      positionNftMint: positionNftKP.publicKey,
      poolAuthority,
      positionNftAccount,
      payer: user.publicKey,
      pool,
      position,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .transaction();

  const addLiquidityTx = await program.methods
    .addLiquidity({
      liquidityDelta: LIQUIDITY_DELTA_2,
      tokenAAmountThreshold: U64_MAX,
      tokenBAmountThreshold: U64_MAX,
    })
    .accountsPartial({
      pool,
      position,
      positionNftAccount,
      owner: user.publicKey,
      tokenAAccount,
      tokenBAccount,
      tokenAVault: poolState.tokenAVault,
      tokenBVault: poolState.tokenBVault,
      tokenAProgram: TOKEN_PROGRAM_ID,
      tokenBProgram: TOKEN_PROGRAM_ID,
      tokenAMint: poolState.tokenAMint,
      tokenBMint: poolState.tokenBMint,
    })
    .transaction();

  const finalTransaction = new Transaction()
    .add(createPositionTx)
    .add(addLiquidityTx);

  finalTransaction.recentBlockhash = svm.latestBlockhash();
  finalTransaction.sign(user, positionNftKP);

  const result = svm.sendTransaction(finalTransaction);
  expect(result).instanceOf(TransactionMetadata);
  return position;
}

export async function removeLiquidity(
  svm: LiteSVM,
  user: PublicKey,
  pool: PublicKey,
  position: PublicKey,
  tokenAAccount: PublicKey,
  tokenBAccount: PublicKey
): Promise<Transaction> {
  const program = createDammV2Program();
  const poolState = getDammV2Pool(svm, pool);
  const positionState = getDammV2Position(svm, position);
  const positionNftAccount = deriveDammV2PositionNftAccount(
    positionState.nftMint
  );

  const poolAuthority = deriveDammV2PoolAuthority();

  const tokenAVault = poolState.tokenAVault;
  const tokenBVault = poolState.tokenBVault;
  const tokenAMint = poolState.tokenAMint;
  const tokenBMint = poolState.tokenBMint;

  return await program.methods
    .removeLiquidity({
      liquidityDelta: positionState.unlockedLiquidity,
      tokenAAmountThreshold: new BN(0),
      tokenBAmountThreshold: new BN(0),
    })
    .accountsPartial({
      poolAuthority,
      pool,
      position,
      positionNftAccount,
      owner: user,
      tokenAAccount,
      tokenBAccount,
      tokenAVault,
      tokenBVault,
      tokenAProgram: TOKEN_PROGRAM_ID,
      tokenBProgram: TOKEN_PROGRAM_ID,
      tokenAMint,
      tokenBMint,
    })
    .transaction();
}
