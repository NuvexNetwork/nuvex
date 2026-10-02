import { SdkNotImplementedError, type JobType } from "../types/jobs.js";

export interface RequestInput {
  jobType: JobType;
  maxFeeLamports: bigint;
}

export function submitRequest(_input: RequestInput): never {
  throw new SdkNotImplementedError("submitRequest");
}
