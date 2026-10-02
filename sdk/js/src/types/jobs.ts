export const JOB_TYPES = ["vrf", "price", "data", "compute", "ai_inference"] as const;

export type JobType = (typeof JOB_TYPES)[number];

export class SdkNotImplementedError extends Error {
  constructor(operation: string) {
    super(`${operation} is not implemented`);
    this.name = "SdkNotImplementedError";
  }
}
