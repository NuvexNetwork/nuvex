import { PublicKey } from "@solana/web3.js";
import { describe, expect, it } from "vitest";

import { buildCallbackInstruction, createClient, protocolPda, submitRequest } from "./index.js";
import { SdkNotImplementedError } from "./types/jobs.js";

describe("sdk", () => {
  it("derives a stable protocol pda", () => {
    const programId = new PublicKey(new Uint8Array(32).fill(7));
    expect(protocolPda(programId)[0].toBase58()).toBe(protocolPda(programId)[0].toBase58());
  });

  it("refuses to submit a request", () => {
    expect(() => submitRequest({ jobType: "vrf", maxFeeLamports: 0n })).toThrow(
      SdkNotImplementedError,
    );
  });

  it("builds the callback payload", () => {
    const output = new Uint8Array(64).fill(4);
    const data = buildCallbackInstruction({ output, data: Uint8Array.from([9, 8]) });
    expect(Array.from(data.slice(0, 8))).toEqual([245, 250, 10, 62, 218, 252, 239, 91]);
    expect(data[8]).toBe(4);
    expect(data[76]).toBe(9);
    expect(data[77]).toBe(8);
  });

  it("requires an rpc url and does not invent one", () => {
    expect(() => createClient({ rpcUrl: "", oracleCoreProgramId: "core" })).toThrow(/rpcUrl/);
  });
});
