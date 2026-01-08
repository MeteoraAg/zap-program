import { LiteSVM } from "litesvm";
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
  zapOutDammV1,
  sendTransaction,
  getTokenBalance,
} from "../common";

import ZapIDL from "../../target/idl/zap.json";
import DAMMV1IDL from "../../idls/damm_v1.json";
import MercurialVaultIDL from "../../idls/mercurial_vault.json";
import { depositVault, setupVault } from "../common/vault";
import BN from "bn.js";
import {
  addBalanceLiquidity,
  removeLiquidity,
  CurveType,
  getAddLiquidityInputAmount,
  METAPLEX_PROGRAM,
  getDammV1Pool,
  initializePermissionedPool,
} from "../common/damm_v1";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";

describe("Zap out damm V1", () => {
  let zapProgram: ZapProgram;
  let svm: LiteSVM;
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
      new PublicKey(METAPLEX_PROGRAM),
      "./tests/fixtures/metaplex.so"
    );
    svm.addProgramFromFile(
      new PublicKey(MercurialVaultIDL.address),
      "./tests/fixtures/mercurial_vault.so"
    );
    svm.addProgramFromFile(
      new PublicKey(DAMMV1IDL.address),
      "./tests/fixtures/damm_v1.so"
    );

    user = Keypair.generate();
    admin = Keypair.generate();
    svm.airdrop(user.publicKey, BigInt(100 * LAMPORTS_PER_SOL));
    svm.airdrop(admin.publicKey, BigInt(100 * LAMPORTS_PER_SOL));

    tokenAMint = createToken(svm, admin, admin.publicKey, null);
    tokenBMint = createToken(svm, admin, admin.publicKey, null);
    mintToken(svm, admin, tokenAMint, admin, admin.publicKey);
    mintToken(svm, admin, tokenBMint, admin, admin.publicKey);

    mintToken(svm, admin, tokenAMint, admin, user.publicKey);
    mintToken(svm, admin, tokenBMint, admin, user.publicKey);
  });

  it("full flow zap out", async () => {
    const inputTokenMint = tokenAMint;

    const aVault = await setupVault(svm, {
      adminKeypair: admin,
      tokenMint: tokenAMint,
    });

    const bVault = await setupVault(svm, {
      adminKeypair: admin,
      tokenMint: tokenBMint,
    });

    await depositVault(svm, {
      vault: aVault,
      depositAmount: new BN(1000),
      userKeypair: admin,
    });

    await depositVault(svm, {
      vault: bVault,
      depositAmount: new BN(1000),
      userKeypair: admin,
    });

    const poolKeypair = Keypair.generate();

    const pool = await initializePermissionedPool(svm, {
      aVault,
      bVault,
      aDepositAmount: new BN(1000),
      bDepositAmount: new BN(1000),
      adminKeypair: admin,
      curve: CurveType.constantProduct(),
      poolKeypair,
    });

    const poolAccount = getDammV1Pool(svm, pool);

    const { aDepositAmount, bDepositAmount, lpAmount } =
      await getAddLiquidityInputAmount(svm, {
        pool,
        depositTokenMint: tokenAMint,
        depositAmount: new BN(10000),
      });

    await addBalanceLiquidity(svm, {
      pool,
      lpAmount,
      maxATokenAmount: aDepositAmount,
      maxBTokenAmount: bDepositAmount,
      userKeypair: user,
    });

    const userPoolLp = getAssociatedTokenAddressSync(
      poolAccount.lpMint,
      user.publicKey
    );

    const removeLiquidityTx = await removeLiquidity(svm, {
      pool,
      minAOutAmount: new BN(0),
      minBOutAmount: new BN(0),
      withdrawAmount: getTokenBalance(svm, userPoolLp),
      userKeypair: user,
    });

    const zapOutTx = await zapOutDammV1(
      svm,
      user.publicKey,
      inputTokenMint,
      pool
    );

    const userTokenB = getAssociatedTokenAddressSync(
      tokenBMint,
      user.publicKey
    );

    const beforeUserTokenB = getTokenBalance(svm, userTokenB);

    const finalTransaction = new Transaction()
      .add(removeLiquidityTx)
      .add(zapOutTx);

    sendTransaction(svm, finalTransaction, [user]);

    const afterUserTokenB = getTokenBalance(svm, userTokenB);

    expect(afterUserTokenB.gt(beforeUserTokenB)).to.be.true;
  });
});
