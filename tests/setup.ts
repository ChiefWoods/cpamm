import { AnchorError, Program } from "@coral-xyz/anchor";
import { Cpamm } from "../target/types/cpamm";
import idl from "../target/idl/cpamm.json";
import { MINT_SIZE, MintLayout, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { mintX, mintY } from "./constants";
import { AccountInfoBytes } from "litesvm";
import { fromWorkspace, LiteSVMProvider } from "anchor-litesvm";
import { expect } from "bun:test";

export async function getSetup(
  accounts: { pubkey: PublicKey; account: AccountInfoBytes }[] = [],
) {
  const litesvm = fromWorkspace("./");

  const [mintXData, mintYData] = Array.from({ length: 2 }, () =>
    Buffer.alloc(MINT_SIZE),
  );

  [mintXData, mintYData].forEach((data) => {
    MintLayout.encode(
      {
        decimals: 6,
        freezeAuthority: PublicKey.default,
        freezeAuthorityOption: 0,
        isInitialized: true,
        mintAuthority: PublicKey.default,
        mintAuthorityOption: 0,
        supply: 100n,
      },
      data,
    );
  });

  [mintXData, mintYData].forEach((data) => {
    MintLayout.encode(
      {
        decimals: 6,
        freezeAuthority: PublicKey.default,
        freezeAuthorityOption: 0,
        isInitialized: true,
        mintAuthority: PublicKey.default,
        mintAuthorityOption: 0,
        supply: 100n,
      },
      data,
    );
  });

  const mintMap = new Map<PublicKey, Buffer>([
    [mintX.publicKey, mintXData],
    [mintY.publicKey, mintYData],
  ]);

  for (const [pubkey, data] of mintMap.entries()) {
    litesvm.setAccount(pubkey, {
      data,
      executable: false,
      lamports: LAMPORTS_PER_SOL,
      owner: TOKEN_PROGRAM_ID,
    });
  }

  for (const { pubkey, account } of accounts) {
    litesvm.setAccount(new PublicKey(pubkey), {
      data: account.data,
      executable: account.executable,
      lamports: account.lamports,
      owner: new PublicKey(account.owner),
    });
  }

  const provider = new LiteSVMProvider(litesvm);
  const program = new Program<Cpamm>(idl, provider);

  return { litesvm, provider, program };
}

export function fundedSystemAccountInfo(
  lamports: number = LAMPORTS_PER_SOL,
): AccountInfoBytes {
  return {
    lamports,
    data: Buffer.alloc(0),
    owner: SystemProgram.programId,
    executable: false,
  };
}

export async function expectAnchorError(error: Error, code: string) {
  expect(error).toBeInstanceOf(AnchorError);
  const { errorCode } = (error as AnchorError).error;
  expect(errorCode.code).toBe(code);
}
