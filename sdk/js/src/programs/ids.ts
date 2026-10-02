export interface ProgramIds {
  oracleCore?: string;
  oracleRegistry?: string;
  verification?: string;
}

export function programIdsFromEnv(env: Record<string, string | undefined>): ProgramIds {
  return {
    oracleCore: env.NUVEX_ORACLE_CORE_PROGRAM_ID,
    oracleRegistry: env.NUVEX_ORACLE_REGISTRY_PROGRAM_ID,
    verification: env.NUVEX_VERIFICATION_PROGRAM_ID,
  };
}
