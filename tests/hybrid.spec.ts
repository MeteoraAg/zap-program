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
  TransactionInstruction,
} from "@solana/web3.js";
import { expect } from "chai";

import ZapIDL from "../target/idl/zap.json";
import { Program } from "@coral-xyz/anchor";
import { Zap } from "../target/types/zap";
import { BN } from "bn.js";
import { DAMM_V2_PROGRAM_ID } from "./common";

export function sendTransaction(
  svm: LiteSVM,
  tx: Transaction,
  signers: Keypair[] = [],
  debug = false
) {
  tx.recentBlockhash = svm.latestBlockhash();
  tx.sign(...signers);
  const re = svm.sendTransaction(tx);
  svm.expireBlockhash(); // To move the slot forward and avoid duplicated tx hash
  if (re instanceof FailedTransactionMetadata)
    if (debug) console.trace(re.meta().logs());
  return re;
}

describe("Zero zap", () => {
  const zapProgram = new Program<Zap>(ZapIDL);
  const svm = new LiteSVM();
  const user = Keypair.generate();

  beforeEach(async () => {
    svm.addProgramFromFile(
      new PublicKey(ZapIDL.address),
      "./target/deploy/zap.so"
    );
    svm.airdrop(user.publicKey, BigInt(LAMPORTS_PER_SOL));
  });

  it("default", async () => {
    const instructionDiscriminator = Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]);
    const instructionData = Buffer.from([]);

    const ix = new TransactionInstruction({
      keys: [
        {
          pubkey: user.publicKey,
          isSigner: true,
          isWritable: true,
        },
      ],
      programId: zapProgram.programId,
      data: Buffer.concat([instructionDiscriminator, instructionData]),
    });

    const tx = new Transaction().add(ix);

    const re = sendTransaction(svm, tx, [user]);
    expect(re).instanceOf(TransactionMetadata);

    if (re instanceof TransactionMetadata) console.log(re.logs());
  });

  it("zapout", async () => {
    const tx = await zapProgram.methods
      .zapOut({
        percentage: 100,
        offsetAmountIn: 8,
        preUserTokenBalance: new BN("100000000000"),
        maxSwapAmount: new BN("100000000000"),
        payloadData: Buffer.from([]),
      })
      .accountsPartial({
        userTokenInAccount: user.publicKey,
        ammProgram: DAMM_V2_PROGRAM_ID,
      })
      .transaction();

    const re = sendTransaction(svm, tx, [user]);
    expect(re).instanceOf(TransactionMetadata);

    if (re instanceof TransactionMetadata) console.log(re.logs());
  });
});
