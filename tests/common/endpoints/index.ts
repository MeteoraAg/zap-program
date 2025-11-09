import { PublicKey, Transaction } from "@solana/web3.js";
import { LiteSVM } from "litesvm";
import { createZapProgram } from "./zapOut";
import { deriveLedgerAccount } from "../pda";
import BN from "bn.js";

export * from "./zapIn";
export * from "./zapOut";

export async function initializeLedgerAccount(
  owner: PublicKey
): Promise<Transaction> {
  const program = createZapProgram();
  return await program.methods
    .initializeLedgerAccount()
    .accountsPartial({
      legder: deriveLedgerAccount(owner),
      owner,
      payer: owner,
    })
    .transaction();
}

export async function closeLedgerAccount(
  owner: PublicKey
): Promise<Transaction> {
  const program = createZapProgram();

  return await program.methods
    .closeLedgerAccount()
    .accountsPartial({
      legder: deriveLedgerAccount(owner),
      owner,
      rentReceiver: owner,
    })
    .transaction();
}

export async function setLedgerBalance(
  owner: PublicKey,
  amount: BN,
  isTokenA: boolean
): Promise<Transaction> {
  const program = createZapProgram();

  return await program.methods
    .setLedgerBalance(amount, isTokenA)
    .accountsPartial({
      legder: deriveLedgerAccount(owner),
      owner,
    })
    .transaction();
}

export async function updateLedgerBalanceAfterSwap(
  owner: PublicKey,
  tokenAccount: PublicKey,
  preSourceTokenAccount: BN,
  maxTransferAmount: BN,
  isTokenA: boolean
): Promise<Transaction> {
  const program = createZapProgram();

  return await program.methods
    .updateLedgerBalanceAfterSwap(
      preSourceTokenAccount,
      maxTransferAmount,
      isTokenA
    )
    .accountsPartial({
      legder: deriveLedgerAccount(owner),
      tokenAccount,
      owner,
    })
    .transaction();
}
