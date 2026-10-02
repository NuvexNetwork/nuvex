# Tests

`make test` builds the programs with `cargo build-sbf` and then runs `cargo test --workspace` and `pnpm test`.

Host tests compile Anchor programs as native libraries. `tests/program` loads `target/deploy/*.so` in LiteSVM 0.17 with the `sol_sha512` feature enabled. That is the on-chain check. It is not a validator deployment.
