import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export interface PdaSeeds {
  protocol: string;
  registry: string;
  node: string;
  request: string;
  verification: string;
  challenge: string;
  reward_vault: string;
  job_config: string;
}

export function loadSeeds(): PdaSeeds {
  const here = dirname(fileURLToPath(import.meta.url));
  const path = resolve(here, "../../../../tests/fixtures/pda-seeds.json");
  return JSON.parse(readFileSync(path, "utf8")) as PdaSeeds;
}
