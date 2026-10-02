import { PublicKey } from "@solana/web3.js";

import { loadSeeds } from "./seeds.js";

export function protocolPda(programId: PublicKey): [PublicKey, number] {
  const seeds = loadSeeds();
  return PublicKey.findProgramAddressSync([Buffer.from(seeds.protocol)], programId);
}

export function requestPda(
  programId: PublicKey,
  requester: PublicKey,
  requestId: Uint8Array,
): [PublicKey, number] {
  if (requestId.length !== 32) {
    throw new Error("request id must be 32 bytes");
  }
  const seeds = loadSeeds();
  return PublicKey.findProgramAddressSync(
    [Buffer.from(seeds.request), requester.toBuffer(), Buffer.from(requestId)],
    programId,
  );
}
