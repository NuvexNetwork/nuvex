# Contributing

Work lands milestone by milestone. A milestone is not done because the crate compiles. It needs tests, docs that match the code, and an explicit statement of what is still unsafe or undecided.

## Local checks

```bash
make check
```

Rust formatting is `rustfmt`. TypeScript formatting is Prettier. Clippy runs with warnings denied. Do not silence a lint to land a stub.

## Protocol changes

- Do not add an instruction that returns success for an unimplemented job.
- Do not add a cryptographic function in this repository. Depend on a named, audited crate and update ADR 0004 in the documentation repository first.
- Economic numbers belong in config accounts. Do not hardcode shares or minimum stake in an instruction.
- New account fields need an ADR amendment before the account is allocated. Allocated accounts are a migration problem.
- `unwrap` is denied by Clippy. Production paths return errors.

## Secrets

Do not commit keypairs, `.env` files, or API credentials. `scripts/check-secrets.sh` scans tracked files. Development program keypairs stay in `keys/program/`.

## Reviews

Security-sensitive changes need the threat-model section updated in the same change. If a decision is ambiguous, stop and extend the ADR instead of picking a behavior in code.
