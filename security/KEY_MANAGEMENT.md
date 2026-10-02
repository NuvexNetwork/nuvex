# Key management

Four roles stay on separate keys:

- node identity, which signs fulfillment
- operator authority, which registers and stakes
- treasury and security-fund authorities, which move fees
- upgrade authority, which is a multisig

Milestone 0 loads none of them.

## Development program keypairs

`scripts/generate-program-keys.py` writes keypairs to `keys/program/` and `target/deploy/`. Both paths are gitignored. The public keys are recorded in `Anchor.toml` and in each program's `declare_id!`.

Those keypairs can deploy the development program address. They must not be the upgrade authority on a deployment that holds value. Anyone with the file can deploy to that address first. Treat a leaked development key as burned: generate a new id, change `declare_id!`, and do not reuse the address.

## Environments

- Local and devnet: a key file path in `NUVEX_NODE_KEY_PATH` is acceptable. The node does not open the file yet.
- Shared environments: a secret manager. The process receives a path or a handle, not a printed key.
- Mainnet: hardware or KMS for the upgrade authority. Hot fulfillment keys are funded only with the operating balance.

Do not commit `.env` files. Example files contain empty secrets.

## Logging

Config debug output redacts RPC URLs and key paths. Do not add a log line that prints key bytes, seed phrases, or request payloads that a user marked confidential.
