# Threat model

Status: updated for Milestone 5. Rows name what the current instructions and the price adapters actually do. "Planned" means a later milestone must address it. It does not mean the design is finished.

Assets, once the protocol holds them: request fees, node stake, the security fund, and the integrity of finalized results that other programs act on.

Trust boundary: Solana account state is authoritative. The API, indexer, and dashboard are not.

| Threat                        | Wanted outcome                              | Planned milestone | Mitigation today                                                                              |
| ----------------------------- | ------------------------------------------- | ----------------- | --------------------------------------------------------------------------------------------- |
| Fake node                     | Unregistered key submits a result           | 2                 | `fulfill` requires the node PDA, its authority, and a proof under the stored key              |
| Sybil cluster                 | Many identities, one operator               | 3                 | Each identity must lock `min_stake`. A minimum of zero does not stop the choice. See ADR 0003 |
| Collusion                     | Assigned set agrees on a false result       | 6                 | VRF accepts the first eligible proof. There is no assigned set                                |
| Forged VRF proof              | Randomness the prover did not produce       | 2                 | `solana-ecvrf` 0.0.1 rejects it. No audit of that crate was found                             |
| Replay                        | Old proof or old transaction accepted again | 2                 | Alpha binds the request account. The status leaves `Pending`                                  |
| Duplicate fulfillment         | Two rewards for one request                 | 2                 | A second `fulfill` is rejected. No reward is paid                                             |
| Unauthorized callback         | Oracle writes attacker-chosen accounts      | 2                 | The CPI passes zero accounts and does not include the request                                 |
| PDA substitution              | Attacker passes a lookalike account         | 1                 | Instructions that exist check PDA seeds                                                       |
| Stale data                    | Price or heartbeat older than the bound     | 5                 | `fulfill` rejects a heartbeat outside the configured window. Price adapters drop an observation whose provider time is outside the requested window. No price account is written |
| Timeout manipulation          | Clock or slot checks skipped                | 1                 | Cancel requires `slot < expires_slot`. Expire requires `slot >= expires_slot`                 |
| Stake and reward manipulation | Withdraw after misbehavior, or claim twice  | 7                 | Withdraw waits out the cooldown. Slash can take stake during it. No reward is paid            |
| Denial of service             | Account bloat or callback compute burn      | 2                 | Requests and proofs are fixed size. A failing callback reverts the transaction                |
| RPC manipulation              | Node trusts a lying RPC                     | 2                 | The node process does not send transactions                                                   |
| Malicious external data       | Source lies or is down                      | 5                 | The median uses only fresh observations and is omitted below the requested minimum. A source can still lie. The chain does not check the median |
| Model poisoning               | Inference follows a substituted model       | 8                 | No inference runtime                                                                          |

Adversarial tests for these rows are a checklist in `tests/adversarial/README.md`. The tests that exist today only assert that the closed paths stay closed.
