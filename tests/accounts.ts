import { PublicKey } from "@solana/web3.js";
import { Cpamm } from "../target/types/cpamm";
import { Program } from "@coral-xyz/anchor";

export async function fetchConfigAcc(
  program: Program<Cpamm>,
  configPda: PublicKey,
) {
  return await program.account.config.fetchNullable(configPda);
}
