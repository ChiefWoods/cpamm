import { beforeEach, describe, expect, test } from "bun:test";
import { Cpamm } from "../../target/types/cpamm";
import { BN, Program } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { mintX, mintY } from "../constants";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { getConfigPda, getMintLpPda } from "../pda";
import { fetchConfigAcc } from "../accounts";
import { LiteSVM } from "litesvm";
import { LiteSVMProvider } from "anchor-litesvm";
import { fundedSystemAccountInfo, getSetup } from "../setup";

describe("initialize", () => {
  let { litesvm, provider, program } = {} as {
    litesvm: LiteSVM;
    provider: LiteSVMProvider;
    program: Program<Cpamm>;
  };

  const authority = Keypair.generate();

  beforeEach(async () => {
    ({ litesvm, provider, program } = await getSetup([
      {
        pubkey: authority.publicKey,
        account: fundedSystemAccountInfo(),
      },
    ]));
  });

  test("initialize a pool config", async () => {
    const seed = new BN(randomBytes(8));
    const locked = false;
    const fee = 100;

    await program.methods
      .initialize({
        seed,
        locked,
        fee,
      })
      .accounts({
        authority: authority.publicKey,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();

    const configPda = getConfigPda(seed);

    const configAcc = await fetchConfigAcc(program, configPda);
    const mintLpPda = getMintLpPda(configPda);

    expect(configAcc.seed).toStrictEqual(seed);
    expect(configAcc.locked).toEqual(locked);
    expect(configAcc.fee).toEqual(fee);
    expect(configAcc.mintX).toStrictEqual(mintX.publicKey);
    expect(configAcc.mintY).toStrictEqual(mintY.publicKey);
    expect(configAcc.authority).toStrictEqual(authority.publicKey);

    const mintLpAcc = litesvm.getAccount(mintLpPda);

    expect(mintLpAcc).not.toBeNull();

    const vaultX = getAssociatedTokenAddressSync(
      mintX.publicKey,
      configPda,
      true,
      TOKEN_PROGRAM_ID,
    );
    const vaultY = getAssociatedTokenAddressSync(
      mintY.publicKey,
      configPda,
      true,
      TOKEN_PROGRAM_ID,
    );

    const vaultXAcc = litesvm.getAccount(vaultX);
    const vaultYAcc = litesvm.getAccount(vaultY);

    expect(vaultXAcc).not.toBeNull();
    expect(vaultYAcc).not.toBeNull();
  });
});
