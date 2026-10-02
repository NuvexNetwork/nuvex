import { SdkNotImplementedError } from "./types/jobs.js";

export interface ClientOptions {
  rpcUrl: string;
  oracleCoreProgramId: string;
}

export function createClient(options: ClientOptions): ClientOptions {
  if (!options.rpcUrl) {
    throw new Error("rpcUrl is required");
  }
  return options;
}

export function getRequest(): never {
  throw new SdkNotImplementedError("getRequest");
}
