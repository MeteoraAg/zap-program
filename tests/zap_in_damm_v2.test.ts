import { LiteSVM, TransactionMetadata } from "litesvm";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  Transaction,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import {
  AccountLayout,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { BN } from "@coral-xyz/anchor";
import {
  createZapProgram,
  createToken,
  mintToken,
  ZapProgram,
  TOKEN_DECIMALS,
} from "./common";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { deserialize } from "borsh";
import fc from "fast-check";
import { expect } from "chai";

import ZapIDL from "../target/idl/zap.json";
import DAMMV2IDL from "../idls/damm_v2.json";
import {
  createDammV2Pool,
  createDammV2Program,
  createPositionAndAddLiquidity,
} from "./common/damm_v2";
import {
  deriveDammV2PoolAuthority,
  deriveDammV2PositionNftAccount,
  getDammV2Pool,
  getDammV2Position,
} from "./common/pda";
import { CuBenchmark } from "./common/benchmark";

const ZapInReturnSchema = {
  array: {
    type: {
      struct: {
        liquidity_delta: "u128",
        token_a_amount: "u64",
        token_b_amount: "u64",
        token_a_remaining_amount: "u64",
        token_b_remaining_amount: "u64",
        token_swapped_amount: "u64",
        token_returned_amount: "u64",
      },
    },
  },
};

describe("Zap in damm V2", () => {
  const cuBenchmark = new CuBenchmark();
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
      new PublicKey(DAMMV2IDL.address),
      "./tests/fixtures/damm_v2.so"
    );

    admin = Keypair.generate();
    user = Keypair.generate();
    svm.airdrop(admin.publicKey, BigInt(LAMPORTS_PER_SOL));
    svm.airdrop(user.publicKey, BigInt(LAMPORTS_PER_SOL));

    tokenAMint = createToken(svm, admin, admin.publicKey, null);
    tokenBMint = createToken(svm, admin, admin.publicKey, null);

    mintToken(svm, admin, tokenAMint, admin, admin.publicKey);
    mintToken(svm, admin, tokenBMint, admin, admin.publicKey);

    mintToken(svm, admin, tokenAMint, admin, user.publicKey);
    mintToken(svm, admin, tokenBMint, admin, user.publicKey);
  });

  const testcases = fc.sample(
    fc.record({
      a: fc.bigInt({
        min: BigInt(0),
        max: BigInt(999_999_000) * BigInt(10 ** TOKEN_DECIMALS),
      }),
      b: fc.bigInt({
        min: BigInt(0),
        max: BigInt(999_999_000) * BigInt(10 ** TOKEN_DECIMALS),
      }),
      verbose: fc.constant(false), // Visible logs
    }),
    100 // Number of testcases
  );

  testcases.forEach(({ a, b, verbose }, i) => {
    it(`zap-in #${i + 1}`, async () => {
      const pool = await createDammV2Pool(svm, admin, tokenAMint, tokenBMint);
      const position = await createPositionAndAddLiquidity(svm, user, pool);

      const dammV2Program = createDammV2Program();
      const poolState = getDammV2Pool(svm, pool);
      const positionState = getDammV2Position(svm, position);

      // Util to construct a zap-in tx
      const getZapInTx = async (params: {
        a: BN;
        b: BN;
        reference: BN | null;
      }) => {
        let zapIn = await zapProgram.methods
          .zapIn(params)
          .accountsPartial({
            poolAuthority: deriveDammV2PoolAuthority(),
            pool,
            position,
            tokenAAccount: getAssociatedTokenAddressSync(
              tokenAMint,
              user.publicKey
            ),
            tokenBAccount: getAssociatedTokenAddressSync(
              tokenBMint,
              user.publicKey
            ),
            tokenAVault: poolState.tokenAVault,
            tokenBVault: poolState.tokenBVault,
            tokenAMint,
            tokenBMint,
            positionNftAccount: deriveDammV2PositionNftAccount(
              positionState.nftMint
            ),
            owner: user.publicKey,
            tokenAProgram: TOKEN_PROGRAM_ID,
            tokenBProgram: TOKEN_PROGRAM_ID,
            dammV2EventAuthority: PublicKey.findProgramAddressSync(
              [Buffer.from("__event_authority")],
              dammV2Program.programId
            )[0],
            dammV2Program: dammV2Program.programId,
            referralTokenAccount: null,
          })
          .transaction();
        let tx = new Transaction();
        tx.add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 14_000_000 })
        ).add(zapIn);
        tx.recentBlockhash = svm.latestBlockhash();
        tx.sign(user);
        return tx;
      };

      // Read previous balances
      const { amount: prevA } = AccountLayout.decode(
        svm.getAccount(
          getAssociatedTokenAddressSync(tokenAMint, user.publicKey)
        ).data
      );
      const { amount: prevB } = AccountLayout.decode(
        svm.getAccount(
          getAssociatedTokenAddressSync(tokenBMint, user.publicKey)
        ).data
      );
      if (verbose) console.log("Zapped-in amounts:", { a, b });
      if (verbose) console.log("Balances before the zap-in:", { prevA, prevB });

      // Get reference
      const simulatedTx = await getZapInTx({
        a: new BN(a),
        b: new BN(b),
        reference: null,
      });
      const simulatedResults = svm.simulateTransaction(simulatedTx);
      let simulatedData = deserialize(
        ZapInReturnSchema,
        simulatedResults.meta().returnData().data()
      ) as Array<{
        token_a_amount: bigint;
        token_b_amount: bigint;
        token_swapped_amount: bigint;
      }>;
      const { token_a_amount, token_b_amount, token_swapped_amount } =
        simulatedData[simulatedData.length - 1];
      const rerefence = !token_a_amount
        ? token_b_amount - token_swapped_amount
        : token_a_amount - token_swapped_amount;

      // Zap-in
      const actualTx = await getZapInTx({
        a: new BN(a),
        b: new BN(b),
        reference: new BN(rerefence),
      });
      const result = svm.sendTransaction(actualTx);
      const meta =
        result instanceof TransactionMetadata ? result : result.meta();
      const logs = meta.logs();

      // Read next balances
      const { amount: nextA } = AccountLayout.decode(
        svm.getAccount(
          getAssociatedTokenAddressSync(tokenAMint, user.publicKey)
        ).data
      );
      const { amount: nextB } = AccountLayout.decode(
        svm.getAccount(
          getAssociatedTokenAddressSync(tokenBMint, user.publicKey)
        ).data
      );
      if (verbose) console.log("Balances after the zap-in:", { nextA, nextB });
      if (verbose) console.log(logs);

      // Avg CU
      cuBenchmark.add(logs);

      // Validate results
      let data = meta.returnData().data();
      let stream = logs.join("\n");
      if (!data.length) {
        expect(stream).contain("Error Code: AmountIsZero");
      } else {
        let results = deserialize(ZapInReturnSchema, data) as Array<{
          liquidity_delta: bigint;
          token_a_amount: bigint;
          token_b_amount: bigint;
          token_a_remaining_amount: bigint;
          token_b_remaining_amount: bigint;
          token_returned_amount: bigint;
          token_swapped_amount: bigint;
        }>;
        if (verbose) console.log("Result:", results);
        const { token_a_remaining_amount, token_b_remaining_amount } =
          results[results.length - 1];
        expect(
          results.reduce(
            (a, { token_a_amount }) => a + token_a_amount,
            nextA - token_a_remaining_amount
          )
        ).eq(prevA);
        expect(
          results.reduce(
            (b, { token_b_amount }) => b + token_b_amount,
            nextB - token_b_remaining_amount
          )
        ).eq(prevB);
      }
    });
  });

  it("report CU usage", () => {
    cuBenchmark.report();
  });
});
