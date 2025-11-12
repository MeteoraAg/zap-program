import {
  FailedTransactionMetadata,
  LiteSVM,
  TransactionMetadata,
} from "litesvm";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  Transaction,
} from "@solana/web3.js";
import {
  createZapProgram,
  createToken,
  mintToken,
  ZapProgram,
  zapOutJupV6,
} from "../common";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { expect } from "chai";

import ZapIDL from "../../target/idl/zap.json";
import {
  createDammV2Pool,
  createPositionAndAddLiquidity,
  DAMM_V2_PROGRAM_ID,
  removeLiquidity,
} from "../common/damm_v2";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { JUP_V6_PROGRAM_ID } from "../common/jup";

describe("Zap out Jup V6", () => {
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
    svm.addProgramFromFile(DAMM_V2_PROGRAM_ID, "./tests/fixtures/damm_v2.so");
    svm.addProgramFromFile(JUP_V6_PROGRAM_ID, "./tests/fixtures/jup_v6.so");

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

  it("full flow zap out", async () => {
    const inputTokenAccount = tokenAMint;
    const pool = await createDammV2Pool(svm, admin, tokenAMint, tokenBMint);
    const userPosition = await createPositionAndAddLiquidity(svm, user, pool);
    const tokenAAccount = getAssociatedTokenAddressSync(
      tokenAMint,
      user.publicKey,
      true,
      TOKEN_PROGRAM_ID
    );
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

    const zapOutTx = await zapOutJupV6(
      svm,
      user.publicKey,
      inputTokenAccount,
      pool
    );

    const finalTransaction = new Transaction()
      .add(removeLiquidityTx)
      .add(zapOutTx);

    finalTransaction.recentBlockhash = svm.latestBlockhash();
    finalTransaction.sign(user);

    const result = svm.sendTransaction(finalTransaction);
    if (result instanceof FailedTransactionMetadata) {
      console.log(result.meta().logs());
    }
    expect(result).instanceOf(TransactionMetadata);
  });
});
