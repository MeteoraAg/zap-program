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
  mintToken,
  ZapProgram,
  zapInDammV2,
  U64_MAX,
} from "./common";
import { expect } from "chai";

import ZapIDL from "../target/idl/zap.json";
import DAMMV2IDL from "../idls/damm_v2.json";
import {
  createDammV2Pool,
  createPositionAndAddLiquidity,
  swapDammV2,
} from "./common/damm_v2";
import BN from "bn.js";

describe.only("Zap in damm V2", () => {
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

  it("full flow zap in", async () => {
    const pool = await createDammV2Pool(svm, admin, tokenAMint, tokenBMint);
    const { position, positionNftAccount } =
      await createPositionAndAddLiquidity(svm, user, pool);
    const swapAmount = new BN("1000000000")
    const swapTx = await swapDammV2(
      svm,
      user.publicKey,
      pool,
      tokenBMint,
      swapAmount
    );

    const zapOutTx = await zapInDammV2(
      svm,
      user.publicKey,
      pool,
      position,
      positionNftAccount,
      U64_MAX,
      U64_MAX,
      swapAmount,
      swapAmount

    );

    const finalTransaction = new Transaction().add(swapTx).add(zapOutTx);

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
