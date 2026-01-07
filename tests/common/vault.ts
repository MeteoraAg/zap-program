import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { LiteSVM } from "litesvm";
import type { Vault } from "./idl/vault";
import VaultIDL from "../../idls/mercurial_vault.json";
import {
  AnchorProvider,
  BN,
  IdlAccounts,
  Program,
  Wallet,
} from "@coral-xyz/anchor";
import { getOrCreateAtA, sendTransaction } from "./utils";

export function createVaultProgram() {
  const wallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(
    new Connection(clusterApiUrl("devnet")),
    wallet,
    {}
  );
  const program = new Program<Vault>(VaultIDL as Vault, provider);
  return program;
}

export const vaultProgram = createVaultProgram();

export const getVaultBase = (allowedRebalance: boolean) => {
  return allowedRebalance
    ? new PublicKey("HWzXGcGHy4tcpYfaRDCyLNzXqBTv3E6BttpCH2vJxArv")
    : PublicKey.default;
};

export const PUBLIC_KEY = 0;

export const getVaultPdas = (
  tokenMint: PublicKey,
  base: PublicKey,
  vaultProgram: Program<Vault>
) => {
  const vaultPda = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), tokenMint.toBuffer(), base.toBuffer()],
    vaultProgram.programId
  );

  const tokenVaultPda = PublicKey.findProgramAddressSync(
    [Buffer.from("token_vault"), vaultPda[PUBLIC_KEY].toBuffer()],
    vaultProgram.programId
  );

  const lpMintPda = PublicKey.findProgramAddressSync(
    [Buffer.from("lp_mint"), vaultPda[0].toBuffer()],
    vaultProgram.programId
  );

  return {
    vaultPda,
    tokenVaultPda,
    lpMintPda,
  };
};

export function getVaultAccount(svm: LiteSVM, vault: PublicKey) {
  const vaultAccountBytes = svm.getAccount(vault);
  if (vaultAccountBytes)
    return vaultProgram.coder.accounts.decode(
      "vault",
      Buffer.from(vaultAccountBytes.data)
    ) as IdlAccounts<Vault>["vault"];
}

export async function setupVault(
  svm: LiteSVM,
  params: {
    tokenMint: PublicKey;
    adminKeypair: Keypair;
  }
): Promise<PublicKey> {
  const { tokenMint, adminKeypair } = params;

  const vaultBase = getVaultBase(true);

  const admin = adminKeypair.publicKey;

  const { vaultPda, tokenVaultPda, lpMintPda } = await getVaultPdas(
    tokenMint,
    vaultBase,
    vaultProgram
  );

  const account = getVaultAccount(svm, vaultPda[PUBLIC_KEY]);

  if (!account) {
    const transaction = await vaultProgram.methods
      .initialize()
      .accountsPartial({
        vault: vaultPda[PUBLIC_KEY],
        tokenVault: tokenVaultPda[PUBLIC_KEY],
        tokenMint,
        payer: admin,
        lpMint: lpMintPda[PUBLIC_KEY],
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .transaction();

    sendTransaction(svm, transaction, [adminKeypair]);
  }

  return vaultPda[PUBLIC_KEY];
}

export const depositVault = async (
  svm: LiteSVM,
  depositVaultParams: {
    vault: PublicKey;
    userKeypair: Keypair;
    depositAmount: BN;
  }
) => {
  const { vault, userKeypair, depositAmount } = depositVaultParams;

  const vaultAccount = getVaultAccount(svm, vault);

  const userWsolLpMint = await getOrCreateAtA(
    svm,
    userKeypair,
    vaultAccount.lpMint,
    userKeypair.publicKey,
    TOKEN_PROGRAM_ID
  );

  const userToken = await getOrCreateAtA(
    svm,
    userKeypair,
    vaultAccount.tokenMint,
    userKeypair.publicKey,
    TOKEN_PROGRAM_ID
  );

  const transaction = await vaultProgram.methods
    .deposit(depositAmount, new BN(0))
    .accountsPartial({
      lpMint: vaultAccount.lpMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenVault: vaultAccount.tokenVault,
      userLp: userWsolLpMint,
      user: userKeypair.publicKey,
      userToken,
      vault,
    })
    .transaction();

  sendTransaction(svm, transaction, [userKeypair]);
};
