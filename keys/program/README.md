# Development program keypairs

JSON keypairs in this directory are gitignored. They must match the public ids in `Anchor.toml`. `scripts/generate-program-keys.py` checks that match and does not create a replacement key.

These keys can claim the development program address. They are not an upgrade multisig and they are not a node signer.
