# Incident response

No production deployment exists. This is the procedure to fill in before mainnet.

## Severity

- Sev-1: funds can move, or a finalized result can be forged.
- Sev-2: liveness only. Requests stall, callbacks fail, nodes cannot heartbeat.
- Sev-3: indexer or API defect with chain state unaffected.

## First hour

1. Confirm the claim against the account, not against the API.
2. If Sev-1 and the pause flag exists, the config authority pauses new requests. Pause is not implemented yet.
3. Snapshot the slot, the affected signatures, and the program buffer.
4. Open a private channel with the upgrade multisig. Do not publish an unfixed exploit.

## After

Publish a timeline, the transactions involved, and the upgrade signature. Update the threat model if the incident used a path that was marked out of scope.

## Contacts

Maintainers are not named in Milestone 0. Add them here before announcing a deployment.
