import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import idl from "../target/idl/cpamm.json";

const AMM_PROGRAM_ID = new PublicKey(idl.address);

export function getConfigPda(seed: BN) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    AMM_PROGRAM_ID,
  )[0];
}

export function getMintLpPda(configPda: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), configPda.toBuffer()],
    AMM_PROGRAM_ID,
  )[0];
}
