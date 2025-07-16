import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { LiteSVM } from "litesvm";

import ZapIDL from "../../target/idl/zap.json";
import { Zap } from "../../target/types/zap";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { DAMM_V2_PROGRAM_ID, getDammV2RemainingAccounts } from "./damm_v2";

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
  tokenAMint: PublicKey,
  tokenBMint: PublicKey
) {
  const zapProgram = createZapProgram();
  const tokenLedgerAccountTokenA = deriveTokenLedgerAddress(tokenAMint);
  const tokenLedgerAccountTokenB = deriveTokenLedgerAddress(tokenBMint);

  const tx1 = await zapProgram.methods
    .initializeTokenLedger()
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: tokenLedgerAccountTokenA,
      tokenMint: tokenAMint,
      payer: user.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .transaction();

  const tx2 = await zapProgram.methods
    .initializeTokenLedger()
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: tokenLedgerAccountTokenB,
      tokenMint: tokenBMint,
      payer: user.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .transaction();

  const tx = new Transaction().add(tx1).add(tx2);

  tx.recentBlockhash = svm.latestBlockhash();
  tx.sign(user);

  svm.sendTransaction(tx);
}

export async function zapOut(
  svm: LiteSVM,
  pool: PublicKey,
  inputTokenAccount: PublicKey,
  outputTokenAccount: PublicKey
): Promise<Transaction> {
  const zapProgram = createZapProgram();

  const remainingAccounts = getDammV2RemainingAccounts(
    svm,
    pool,
    outputTokenAccount
  );
  return await zapProgram.methods
    .zapOut(Buffer.from([]))
    .accountsPartial({
      zapAuthority: deriveZapAuthorityAddress(),
      tokenLedgerAccount: inputTokenAccount,
      ammProgram: DAMM_V2_PROGRAM_ID,
    })
    .remainingAccounts(remainingAccounts)
    .transaction();
}
