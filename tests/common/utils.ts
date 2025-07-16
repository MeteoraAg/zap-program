import { BN } from "@coral-xyz/anchor";
import {
  FailedTransactionMetadata,
  LiteSVM,
  TransactionMetadata,
} from "litesvm";

import ZapIDL from "../../target/idl/zap.json";
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";

export const TOKEN_DECIMALS = 9;
export const RAW_AMOUNT = 1_000_000_000 * 10 ** TOKEN_DECIMALS;
export const MIN_SQRT_PRICE = new BN("4295048016");
export const MAX_SQRT_PRICE = new BN("79226673521066979257578248091");
export const LIQUIDITY_DELTA = new BN("1844674407800459963300003758876517305");
export const INIT_PRICE = new BN("18446744073709551616");
export const LIQUIDITY_DELTA_2 = new BN("18446744078004599633000037588765");
export const U64_MAX = new BN("18446744073709551615");

export function createToken(
  svm: LiteSVM,
  payer: Keypair,
  mintAuthority: PublicKey,
  freezeAuthority?: PublicKey
): PublicKey {
  const mintKeypair = Keypair.generate();
  const rent = svm.getRent();
  const lamports = rent.minimumBalance(BigInt(MINT_SIZE));

  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: mintKeypair.publicKey,
    space: MINT_SIZE,
    lamports: Number(lamports.toString()),
    programId: TOKEN_PROGRAM_ID,
  });

  const initializeMintIx = createInitializeMint2Instruction(
    mintKeypair.publicKey,
    TOKEN_DECIMALS,
    mintAuthority,
    freezeAuthority
  );

  let transaction = new Transaction();
  transaction.recentBlockhash = svm.latestBlockhash();
  transaction.add(createAccountIx, initializeMintIx);
  transaction.sign(payer, mintKeypair);

  svm.sendTransaction(transaction);

  return mintKeypair.publicKey;
}

export function mintToken(
  svm: LiteSVM,
  payer: Keypair,
  mint: PublicKey,
  mintAuthority: Keypair,
  toWallet: PublicKey
) {
  const destination = getOrCreateAtA(svm, payer, mint, toWallet);

  const mintIx = createMintToInstruction(
    mint,
    destination,
    mintAuthority.publicKey,
    RAW_AMOUNT
  );

  let transaction = new Transaction();
  transaction.recentBlockhash = svm.latestBlockhash();
  transaction.add(mintIx);
  transaction.sign(payer, mintAuthority);

  svm.sendTransaction(transaction);
}

export function getOrCreateAtA(
  svm: LiteSVM,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey,
  tokenProgram = TOKEN_PROGRAM_ID
): PublicKey {
  const ataKey = getAssociatedTokenAddressSync(mint, owner, true, tokenProgram);

  const account = svm.getAccount(ataKey);
  if (account === null) {
    const createAtaIx = createAssociatedTokenAccountInstruction(
      payer.publicKey,
      ataKey,
      owner,
      mint,
      tokenProgram
    );
    let transaction = new Transaction();

    transaction.recentBlockhash = svm.latestBlockhash();
    transaction.add(createAtaIx);
    transaction.sign(payer);
    svm.sendTransaction(transaction);
  }

  return ataKey;
}

export function generateUsers(svm: LiteSVM, numberOfUsers: number) {
  const res = [];
  for (let i = 0; i < numberOfUsers; i++) {
    const user = Keypair.generate();
    svm.airdrop(user.publicKey, BigInt(LAMPORTS_PER_SOL));
    res.push(user);
  }

  return res;
}

export function getProgramErrorCodeHexString(errorMessage: String) {
  const error = ZapIDL.errors.find(
    (e) =>
      e.name.toLowerCase() === errorMessage.toLowerCase() ||
      e.msg.toLowerCase() === errorMessage.toLowerCase()
  );

  if (!error) {
    throw new Error(
      `Unknown Zap Program error message / name: ${errorMessage}`
    );
  }

  return error.code;
}

export function expectThrowsErrorCode(
  response: TransactionMetadata | FailedTransactionMetadata,
  errorCode: number
) {
  if (response instanceof FailedTransactionMetadata) {
    const message = response.err().toString();

    if (!message.toString().includes(errorCode.toString())) {
      throw new Error(
        `Unexpected error: ${message}. Expected error: ${errorCode}`
      );
    }

    return;
  } else {
    throw new Error("Expected an error but didn't get one");
  }
}
