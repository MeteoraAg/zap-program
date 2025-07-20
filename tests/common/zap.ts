import { AnchorProvider, BN, Program, Wallet } from "@coral-xyz/anchor";
import {
  FailedTransactionMetadata,
  LiteSVM,
  TransactionMetadata,
} from "litesvm";

import ZapIDL from "../../target/idl/zap.json";
import { Zap } from "../../target/types/zap";
import {  TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { DAMM_V2_PROGRAM_ID, getDammV2RemainingAccounts } from "./damm_v2";
import {
  DLMM_PROGRAM_ID_LOCAL,
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

  return tokenLedgerAccountToken;
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
  const actionType = Buffer.from([0]);
  const dammv2Data = new BN(10).toArrayLike(Buffer, "le", 8);
  const data = Buffer.concat([actionType, dammv2Data]);
  return await zapProgram.methods
    .zapOut(data)
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
  tokenYProgram = TOKEN_PROGRAM_ID
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

  const actionType = Buffer.from([1]);
  const minimumAmountOutData = new BN(10).toArrayLike(Buffer, "le", 8);

  const sliceCount = Buffer.alloc(4);
  sliceCount.writeUInt32LE(remainingAccountsInfo.slices.length, 0);

  // Serialize each slice (accounts_type: u8, length: u8)
  const slicesData = Buffer.concat(
    remainingAccountsInfo.slices.map((slice) => {
      const sliceBuffer = Buffer.alloc(2);
      sliceBuffer.writeUInt8(convertAccountTypeToNumber(slice.accountsType), 0);
      sliceBuffer.writeUInt8(slice.length, 1);
      return sliceBuffer;
    })
  );

  const data = Buffer.concat([
    actionType, // 1 byte
    minimumAmountOutData, // 8 bytes
    sliceCount, // 4 bytes anchor prefix
    slicesData, // 2 bytes per slice
  ]);

  return await zapProgram.methods
    .zapOut(data)
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: inputTokenAccount,
      ammProgram: DLMM_PROGRAM_ID_LOCAL,
    })
    .remainingAccounts(remainingAccounts)
    .transaction();
}

function convertAccountTypeToNumber(accountType: object): number {
  if (JSON.stringify(accountType) === JSON.stringify({ transferHookX: {} })) {
    return 0;
  }

  if (JSON.stringify(accountType) === JSON.stringify({ transferHookY: {} })) {
    return 1;
  }
  if (
    JSON.stringify(accountType) === JSON.stringify({ transferHookReward: {} })
  ) {
    return 2;
  }
}
