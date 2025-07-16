import { LiteSVM, TransactionMetadata } from "litesvm";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  Transaction,
} from "@solana/web3.js";
import {
  createZapProgram,
  createToken,
  deriveTokenLedgerAddress,
  deriveZapAuthorityAddress,
  initializeTokenLedger,
  mintToken,
  ZapProgram,
  zapOut,
} from "./common";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { expect } from "chai";

import ZapIDL from "../target/idl/zap.json";
import DAMMV2IDL from "../idls/damm_v2.json";
import {
  createDammV2Pool,
  createPositionAndAddLiquidity,
  removeLiquidity,
} from "./common/damm_v2";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("Zap out damm V2", () => {
  let zapProgram: ZapProgram;
  let svm: LiteSVM;
  let tokenMint: PublicKey;
  let user: Keypair;
  let admin: Keypair;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;

  beforeEach(async () => {
    zapProgram = createZapProgram();
    svm = new LiteSVM();
    svm.addProgramFromFile(
      new PublicKey(ZapIDL.address),
      "./target/deploy/zap.so"
    );
    svm.addProgramFromFile(
      new PublicKey(DAMMV2IDL.address),
      "./tests/fixtures/damm_v2.so"
    );

    user = Keypair.generate();
    admin = Keypair.generate();
    svm.airdrop(user.publicKey, BigInt(LAMPORTS_PER_SOL));
    svm.airdrop(admin.publicKey, BigInt(LAMPORTS_PER_SOL));

    tokenAMint = createToken(svm, admin, admin.publicKey, null);
    tokenBMint = createToken(svm, admin, admin.publicKey, null);
    mintToken(svm, admin, tokenAMint, admin, admin.publicKey);
    mintToken(svm, admin, tokenBMint, admin, admin.publicKey);

    mintToken(svm, admin, tokenAMint, admin, user.publicKey);
    mintToken(svm, admin, tokenBMint, admin, user.publicKey);
  });

  it("initialize token ledger", async () => {
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

    const result = svm.sendTransaction(tx);

    expect(result).instanceOf(TransactionMetadata);
    const tokenLedgerAccountAData = svm.getAccount(tokenLedgerAccountTokenA);
    const tokenLedgerAccountBData = svm.getAccount(tokenLedgerAccountTokenA);
    expect(tokenLedgerAccountAData).not.to.be.null;
    expect(tokenLedgerAccountBData).not.to.be.null;
  });

  it("zap out", async () => {
    await initializeTokenLedger(svm, user, tokenAMint, tokenBMint);
    const tokenLedgerAccountTokenA = deriveTokenLedgerAddress(tokenAMint);
    const pool = await createDammV2Pool(svm, admin, tokenAMint, tokenBMint);
    const userPosition = await createPositionAndAddLiquidity(svm, user, pool);
    const tokenAAccount = tokenLedgerAccountTokenA;
    const tokenBAccount = getAssociatedTokenAddressSync(
      tokenBMint,
      user.publicKey,
      true,
      TOKEN_PROGRAM_ID
    );
    const removeLiquidityTx = await removeLiquidity(
      svm,
      user.publicKey,
      pool,
      userPosition,
      tokenAAccount,
      tokenBAccount
    );

    const zapOutTx = await zapOut(svm, pool, tokenAAccount, tokenBAccount);

    const finalTransaction = new Transaction()
      .add(removeLiquidityTx)
      .add(zapOutTx);

    finalTransaction.recentBlockhash = svm.latestBlockhash();
    finalTransaction.sign(user);

    const result = svm.sendTransaction(finalTransaction);
    if (result instanceof TransactionMetadata) {
      console.log(result.logs());
    } else {
      console.log(result.meta().logs());
    }
    expect(result).instanceOf(TransactionMetadata);
  });
});
