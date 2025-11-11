import { BN } from "@coral-xyz/anchor";
import { binIdToBinArrayIndex, DLMM_PROGRAM_ID_LOCAL } from "./dlmm";
import { PublicKey } from "@solana/web3.js";
import {
  createDammV2Program,
  DAMM_V2_PROGRAM_ID,
  Pool,
  Position,
} from "./damm_v2";
import { LiteSVM } from "litesvm";
import { ZAP_PROGRAM_ID } from "./endpoints";

////// ZAP PDA /////

export function deriveLedgerAccount(owner: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("user_ledger"), owner.toBuffer()],
    ZAP_PROGRAM_ID
  )[0];
}

///////// DAMM V2 ////////////

export function getDammV2Pool(svm: LiteSVM, pool: PublicKey): Pool {
  const program = createDammV2Program();
  const account = svm.getAccount(pool);
  return program.coder.accounts.decode("pool", Buffer.from(account.data));
}

export function getDammV2Position(svm: LiteSVM, position: PublicKey): Position {
  const program = createDammV2Program();
  const account = svm.getAccount(position);
  return program.coder.accounts.decode("position", Buffer.from(account.data));
}

export function getSecondKey(key1: PublicKey, key2: PublicKey) {
  const buf1 = key1.toBuffer();
  const buf2 = key2.toBuffer();
  // Buf1 > buf2
  if (Buffer.compare(buf1, buf2) === 1) {
    return buf2;
  }
  return buf1;
}

export function getFirstKey(key1: PublicKey, key2: PublicKey) {
  const buf1 = key1.toBuffer();
  const buf2 = key2.toBuffer();
  // Buf1 > buf2
  if (Buffer.compare(buf1, buf2) === 1) {
    return buf1;
  }
  return buf2;
}

export function deriveDammV2CustomizablePoolAddress(
  tokenAMint: PublicKey,
  tokenBMint: PublicKey
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("cpool"),
      getFirstKey(tokenAMint, tokenBMint),
      getSecondKey(tokenAMint, tokenBMint),
    ],
    DAMM_V2_PROGRAM_ID
  )[0];
}

export function deriveDammV2TokenVaultAddress(
  tokenMint: PublicKey,
  pool: PublicKey
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("token_vault"), tokenMint.toBuffer(), pool.toBuffer()],
    DAMM_V2_PROGRAM_ID
  )[0];
}

export function deriveDammV2EventAuthority() {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    DAMM_V2_PROGRAM_ID
  )[0];
}
export function deriveDammV2PositionAddress(positionNft: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("position"), positionNft.toBuffer()],
    DAMM_V2_PROGRAM_ID
  )[0];
}

export function deriveDammV2PositionNftAccount(
  positionNftMint: PublicKey
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("position_nft_account"), positionNftMint.toBuffer()],
    DAMM_V2_PROGRAM_ID
  )[0];
}

export function deriveDammV2PoolAuthority(): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("pool_authority")],
    DAMM_V2_PROGRAM_ID
  )[0];
}

////// DLMM PDA ACCOUNT ///////////
export function derivePresetParameter2(index: BN) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("preset_parameter2"), new Uint8Array(index.toBuffer("le", 2))],
    DLMM_PROGRAM_ID_LOCAL
  );
}

export function sortTokenMints(tokenX: PublicKey, tokenY: PublicKey) {
  const [minKey, maxKey] =
    tokenX.toBuffer().compare(tokenY.toBuffer()) == 1
      ? [tokenY, tokenX]
      : [tokenX, tokenY];

  return [minKey, maxKey];
}

export function deriveLbPermissionless2(
  presetParameter2: PublicKey,
  tokenX: PublicKey,
  tokenY: PublicKey
) {
  const [minKey, maxKey] = sortTokenMints(tokenX, tokenY);
  return PublicKey.findProgramAddressSync(
    [presetParameter2.toBuffer(), minKey.toBuffer(), maxKey.toBuffer()],
    DLMM_PROGRAM_ID_LOCAL
  );
}

export function deriveLbCustomizablePermissionless2(
  tokenX: PublicKey,
  tokenY: PublicKey
): PublicKey {
  const baseKey = new PublicKey("MFGQxwAmB91SwuYX36okv2Qmdc9aMuHTwWGUrp4AtB1");
  const [minKey, maxKey] = sortTokenMints(tokenX, tokenY);
  return PublicKey.findProgramAddressSync(
    [baseKey.toBuffer(), minKey.toBuffer(), maxKey.toBuffer()],
    DLMM_PROGRAM_ID_LOCAL
  )[0];
}

export function deriveReserve(token: PublicKey, lbPair: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [lbPair.toBuffer(), token.toBuffer()],
    DLMM_PROGRAM_ID_LOCAL
  );
}

export function deriveOracle(lbPair: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("oracle"), lbPair.toBytes()],
    DLMM_PROGRAM_ID_LOCAL
  );
}

export function deriveBinArrayBitmapExtension(lbPair: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("bitmap"), lbPair.toBytes()],
    DLMM_PROGRAM_ID_LOCAL
  );
}

export function deriveDlmmEventAuthority() {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    DLMM_PROGRAM_ID_LOCAL
  )[0];
}

export function deriveBinArray(lbPair: PublicKey, index: BN) {
  let binArrayBytes: Uint8Array;
  if (index.isNeg()) {
    binArrayBytes = new Uint8Array(index.toTwos(64).toBuffer("le", 8));
  } else {
    binArrayBytes = new Uint8Array(index.toBuffer("le", 8));
  }

  return PublicKey.findProgramAddressSync(
    [Buffer.from("bin_array"), lbPair.toBytes(), binArrayBytes],
    DLMM_PROGRAM_ID_LOCAL
  );
}

export function deriveTokenBadge(token: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("token_badge"), token.toBuffer()],
    DLMM_PROGRAM_ID_LOCAL
  )[0];
}

export async function getBinArraysForModifyLiquidity(
  lbPair: PublicKey,
  minBinId: number
) {
  const lowerBinId = new BN(minBinId);

  const binArrayLowerIndex = binIdToBinArrayIndex(lowerBinId);
  const binArrayUpperIndex = binArrayLowerIndex.add(new BN(1));

  const [binArrayLower] = deriveBinArray(lbPair, binArrayLowerIndex);
  const [binArrayUpper] = deriveBinArray(lbPair, binArrayUpperIndex);

  return {
    binArrayLower,
    binArrayUpper,
  };
}
