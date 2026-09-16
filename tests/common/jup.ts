import {
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { LiteSVM } from "litesvm";
import { Jupiter } from "./idl/jupiter";
import JupIDL from "../../idls/jupiter.json";
import { IdlTypes } from "@anchor-lang/core";
import { DAMM_V2_PROGRAM_ID } from "./damm_v2";
import {
  deriveDammV2EventAuthority,
  deriveDammV2PoolAuthority,
  getDammV2Pool,
} from "./pda";

export type RoutePlanStep = IdlTypes<Jupiter>["routePlanStep"];

export const JUP_V6_PROGRAM_ID = new PublicKey(JupIDL.address);
export const JUP_ROUTE_DISC = [229, 23, 203, 151, 122, 227, 173, 42];
export const JUP_ROUTE_V2_DISC = [187, 100, 250, 204, 49, 196, 175, 20];
export const JUP_SHARED_ACCOUNT_ROUTE_DISC = [
  193, 32, 155, 51, 65, 214, 156, 129,
];
export const JUP_SHARED_ACCOUNT_ROUTE_V2_DISC = [
  209, 152, 83, 147, 124, 254, 216, 233,
];
// `id` arg of shared_accounts_route / shared_accounts_route_v2, also the program authority seed
export const JUP_SHARED_ACCOUNT_AUTHORITY_ID = 0;
export function deriveJupV6EventAuthority() {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    JUP_V6_PROGRAM_ID
  )[0];
}

export function deriveJupV6ProgramAuthority(
  id = JUP_SHARED_ACCOUNT_AUTHORITY_ID
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("authority"), Buffer.from([id])],
    JUP_V6_PROGRAM_ID
  )[0];
}

type RemainingAccount = {
  isSigner: boolean;
  isWritable: boolean;
  pubkey: PublicKey;
};

// https://explorer.solana.com/tx/4r5gcvi3j2RoPedr1zYxUmLRfMt29U9FNucCfGkxoYSC5sxnv6U5nuYNzVqjJpV4RCZb9qBrMzp2A3dhN4NHH6G9
export function getJupRemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userTokenInAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  outputMint: PublicKey,
  tokenAProgram = TOKEN_PROGRAM_ID,
  tokenBProgram = TOKEN_PROGRAM_ID
): Array<RemainingAccount> {
  const accounts: Array<RemainingAccount> = [
    {
      isSigner: false,
      isWritable: false,
      pubkey: TOKEN_PROGRAM_ID,
    },
    {
      pubkey: user,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: userTokenInAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenOutAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: JUP_V6_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: JUP_V6_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveJupV6EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: JUP_V6_PROGRAM_ID,
    },
    ...getDammV2SwapAccounts(
      svm,
      pool,
      user,
      userTokenInAccount,
      userTokenOutAccount,
      tokenAProgram,
      tokenBProgram
    ),
  ];
  return accounts;
}

export function getJupRouteV2RemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userTokenInAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  inputMint: PublicKey,
  outputMint: PublicKey,
  inputTokenProgram = TOKEN_PROGRAM_ID,
  outputTokenProgram = TOKEN_PROGRAM_ID,
  tokenAProgram = TOKEN_PROGRAM_ID,
  tokenBProgram = TOKEN_PROGRAM_ID
): Array<RemainingAccount> {
  const accounts: Array<RemainingAccount> = [
    {
      pubkey: user,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: userTokenInAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenOutAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: inputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: inputTokenProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputTokenProgram,
      isSigner: false,
      isWritable: false,
    },
    // optional destination_token_account
    {
      pubkey: JUP_V6_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveJupV6EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: JUP_V6_PROGRAM_ID,
    },
    ...getDammV2SwapAccounts(
      svm,
      pool,
      user,
      userTokenInAccount,
      userTokenOutAccount,
      tokenAProgram,
      tokenBProgram
    ),
  ];
  return accounts;
}

export function getJupSharedAccountRouteRemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userTokenInAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  inputMint: PublicKey,
  outputMint: PublicKey,
  tokenAProgram = TOKEN_PROGRAM_ID,
  tokenBProgram = TOKEN_PROGRAM_ID
): Array<RemainingAccount> {
  const programAuthority = deriveJupV6ProgramAuthority();
  const programSourceTokenAccount = getAssociatedTokenAddressSync(
    inputMint,
    programAuthority,
    true
  );
  const programDestinationTokenAccount = getAssociatedTokenAddressSync(
    outputMint,
    programAuthority,
    true
  );
  const accounts: Array<RemainingAccount> = [
    {
      pubkey: TOKEN_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: programAuthority,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: user,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: userTokenInAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: programSourceTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: programDestinationTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenOutAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: inputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputMint,
      isSigner: false,
      isWritable: false,
    },
    // optional platform_fee_account
    {
      pubkey: JUP_V6_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    // optional token_2022_program
    {
      pubkey: TOKEN_2022_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveJupV6EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: JUP_V6_PROGRAM_ID,
    },
    // the swap is executed from the program owned token accounts, signed by the program authority PDA
    ...getDammV2SwapAccounts(
      svm,
      pool,
      programAuthority,
      programSourceTokenAccount,
      programDestinationTokenAccount,
      tokenAProgram,
      tokenBProgram,
      false
    ),
  ];
  return accounts;
}

export function getJupSharedAccountRouteV2RemainingAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userTokenInAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  inputMint: PublicKey,
  outputMint: PublicKey,
  inputTokenProgram = TOKEN_PROGRAM_ID,
  outputTokenProgram = TOKEN_PROGRAM_ID,
  tokenAProgram = TOKEN_PROGRAM_ID,
  tokenBProgram = TOKEN_PROGRAM_ID
): Array<RemainingAccount> {
  const programAuthority = deriveJupV6ProgramAuthority();
  const programSourceTokenAccount = getAssociatedTokenAddressSync(
    inputMint,
    programAuthority,
    true,
    inputTokenProgram
  );
  const programDestinationTokenAccount = getAssociatedTokenAddressSync(
    outputMint,
    programAuthority,
    true,
    outputTokenProgram
  );
  const accounts: Array<RemainingAccount> = [
    {
      pubkey: programAuthority,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: user,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: userTokenInAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: programSourceTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: programDestinationTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenOutAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: inputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: inputTokenProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: outputTokenProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveJupV6EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: JUP_V6_PROGRAM_ID,
    },
    // the swap is executed from the program owned token accounts, signed by the program authority PDA
    ...getDammV2SwapAccounts(
      svm,
      pool,
      programAuthority,
      programSourceTokenAccount,
      programDestinationTokenAccount,
      tokenAProgram,
      tokenBProgram,
      false
    ),
  ];
  return accounts;
}

function getDammV2SwapAccounts(
  svm: LiteSVM,
  pool: PublicKey,
  user: PublicKey,
  userTokenInAccount: PublicKey,
  userTokenOutAccount: PublicKey,
  tokenAProgram: PublicKey,
  tokenBProgram: PublicKey,
  userIsSigner = true
): Array<RemainingAccount> {
  const poolState = getDammV2Pool(svm, pool);
  return [
    // swap pool account
    {
      pubkey: DAMM_V2_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: deriveDammV2PoolAuthority(),
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: pool,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenInAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenOutAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: poolState.tokenAVault,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: poolState.tokenBVault,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: poolState.tokenAMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: poolState.tokenBMint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: user,
      isSigner: userIsSigner,
      isWritable: false,
    },
    {
      pubkey: tokenAProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: tokenBProgram,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: DAMM_V2_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: deriveDammV2EventAuthority(),
    },
    {
      isSigner: false,
      isWritable: false,
      pubkey: DAMM_V2_PROGRAM_ID,
    },
  ];
}
