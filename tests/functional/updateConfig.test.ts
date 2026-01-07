import { beforeEach, describe, expect, test } from "bun:test";
import { Cpamm } from "../../target/types/cpamm";
import { BN, Program } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { mintX, mintY } from "../constants";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { getConfigPda } from "../pda";
import { fetchConfigAcc } from "../accounts";
import { LiteSVM } from "litesvm";
import { LiteSVMProvider } from "anchor-litesvm";
import { expectAnchorError, fundedSystemAccountInfo, getSetup } from "../setup";

describe("update", () => {
  let { litesvm, provider, program } = {} as {
    litesvm: LiteSVM;
    provider: LiteSVMProvider;
    program: Program<Cpamm>;
  };

  const [authorityA, authorityB] = Array.from({ length: 2 }, Keypair.generate);
  const seed = new BN(randomBytes(8));
  const configPda = getConfigPda(seed);

  beforeEach(async () => {
    ({ litesvm, provider, program } = await getSetup(
      [authorityA, authorityB].map((kp) => ({
        pubkey: kp.publicKey,
        account: fundedSystemAccountInfo(),
      })),
    ));

    await program.methods
      .initialize({
        seed,
        locked: false,
        fee: 100,
      })
      .accounts({
        authority: authorityA.publicKey,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authorityA])
      .rpc();
  });

  test("update a pool config", async () => {
    const locked = true;
    const fee = 200;
    const authority = authorityB.publicKey;

    await program.methods
      .updateConfig({
        locked,
        fee,
        authority,
      })
      .accountsPartial({
        authority: authorityA.publicKey,
        config: configPda,
      })
      .signers([authorityA])
      .rpc();

    const configAcc = await fetchConfigAcc(program, configPda);

    expect(configAcc.locked).toEqual(locked);
    expect(configAcc.fee).toEqual(fee);
    expect(configAcc.authority).toStrictEqual(authority);
  });

  test("throws if signer is not config authority", async () => {
    const locked = true;
    const fee = 200;
    const authority = authorityB.publicKey;

    try {
      await program.methods
        .updateConfig({
          locked,
          fee,
          authority,
        })
        .accountsPartial({
          authority: authorityB.publicKey,
          config: configPda,
        })
        .signers([authorityB])
        .rpc();
    } catch (err) {
      expectAnchorError(err, "InvalidConfigAuthority");
    }
  });
});
