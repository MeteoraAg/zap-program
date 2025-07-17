import { AnchorProvider, BN, Program, Wallet } from "@coral-xyz/anchor";
import {
  FailedTransactionMetadata,
  LiteSVM,
  TransactionMetadata,
} from "litesvm";

import ZapIDL from "../../target/idl/zap.json";
import { Zap } from "../../target/types/zap";
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  AccountMeta,
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { DAMM_V2_PROGRAM_ID, getDammV2RemainingAccounts } from "./damm_v2";
import {
  DLMM_PROGRAM_ID_LOCAL,
  getBinArraysForSwap,
  getDlmmRemainingAccounts,
} from "./dlmm";
import { expect } from "chai";

export const ZAP_PROGRAM_ID = new PublicKey(ZapIDL.address);

export type ZapProgram = Program<Zap>;

export function createZapProgram(): ZapProgram {
  const wallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(
    new Connection(clusterApiUrl("devnet")),
    wallet,
    {}
  );
  const program = new Program<Zap>(ZapIDL as Zap, provider);
  return program;
}

export function deriveZapAuthorityAddress(): PublicKey {
  const program = createZapProgram();
  return PublicKey.findProgramAddressSync(
    [Buffer.from("zap_authority")],
    program.programId
  )[0];
}

export function deriveTokenLedgerAddress(mintAddress: PublicKey): PublicKey {
  const program = createZapProgram();
  return PublicKey.findProgramAddressSync(
    [Buffer.from("token_ledger"), mintAddress.toBuffer()],
    program.programId
  )[0];
}

export async function initializeTokenLedger(
  svm: LiteSVM,
  user: Keypair,
  tokenMint: PublicKey,
  tokenProgram = TOKEN_PROGRAM_ID
): Promise<PublicKey> {
  const zapProgram = createZapProgram();
  const tokenLedgerAccountToken = deriveTokenLedgerAddress(tokenMint);

  const tx = await zapProgram.methods
    .initializeTokenLedger()
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: tokenLedgerAccountToken,
      tokenMint,
      payer: user.publicKey,
      tokenProgram,
    })
    .transaction();

  tx.recentBlockhash = svm.latestBlockhash();
  tx.sign(user);

  const result = svm.sendTransaction(tx);
  if (result instanceof FailedTransactionMetadata) {
    console.log(result.meta().logs());
  }
  expect(result).instanceOf(TransactionMetadata);

  return tokenLedgerAccountToken
}

export async function zapOutDammv2(
  svm: LiteSVM,
  pool: PublicKey,
  inputTokenAccount: PublicKey,
  outputTokenAccount: PublicKey
): Promise<Transaction> {
  const zapProgram = createZapProgram();

  const remainingAccounts = getDammV2RemainingAccounts(
    svm,
    pool,
    inputTokenAccount,
    outputTokenAccount
  );
  return await zapProgram.methods
    .zapOut({
      minimumAmountOut: new BN(0),
      padding0: [],
      remainingAccountsInfo: null,
    })
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: inputTokenAccount,
      ammProgram: DAMM_V2_PROGRAM_ID,
    })
    .remainingAccounts(remainingAccounts)
    .transaction();
}

export async function zapOutDlmm(
  svm: LiteSVM,
  lbPair: PublicKey,
  inputTokenAccount: PublicKey,
  outputTokenAccount: PublicKey,
  tokenXProgram = TOKEN_PROGRAM_ID,
  tokenYProgram = TOKEN_PROGRAM_ID,
): Promise<Transaction> {
  const zapProgram = createZapProgram();

  const { remainingAccounts, remainingAccountsInfo } = getDlmmRemainingAccounts(
    svm,
    lbPair,
    inputTokenAccount,
    outputTokenAccount,
    tokenXProgram,
    tokenYProgram
  );

  return await zapProgram.methods
    .zapOut({
      minimumAmountOut: new BN(0),
      padding0: [],
      remainingAccountsInfo,
    })
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: inputTokenAccount,
      ammProgram: DLMM_PROGRAM_ID_LOCAL,
    })
    .remainingAccounts(remainingAccounts)
    .transaction();
}
