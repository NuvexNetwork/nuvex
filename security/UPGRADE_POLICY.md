# Upgrade policy

See ADR 0007.

## Development

Upgrade authority is a multisig. The program deploy keypair is not that multisig.

## Before mainnet

All of the following are required and none of them are done:

- named multisig members and threshold
- timelock between buffer upload and upgrade
- public changelog with the buffer hash
- emergency pause on `ProtocolConfig` that does not require an upgrade
- an independent audit of the deployed revision
- a rehearsal on devnet

## Emergency

An emergency upgrade still needs the multisig. It skips the timelock only for a written class of bugs: theft of funds, forged finality, or a bricked callback path. The incident record explains why the timelock was skipped.

## What this repository will not do

`scripts/deploy-mainnet.sh` does not deploy. Burning upgrade authority is not a setup step.
