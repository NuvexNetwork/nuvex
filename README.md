# Nuvex

Solana-native verifiable compute and oracle protocol.

Programs accept a job, an input, constraints, and a callback. Randomness is the first job. An active node can fulfill a VRF request with an ECVRF proof after it locks the configured stake and sends a heartbeat. Fees still do not move.

## Status

Milestone 3 verifies VRF proofs with `solana-ecvrf` 0.0.1 (RFC 9381 ECVRF-EDWARDS25519-SHA512-TAI, 80-byte proof, 64-byte output). No audit of that crate was found. A node fulfills only when it is active, its stake meets the configured minimum, its heartbeat is inside the configured window, and the key was registered before the request. The callback, when set, receives the output and no accounts. `max_fee` is stored and never charged. The node process still does not submit transactions. The SDK can prove on the host and still refuses to send a transaction. An operator who funds several keys can still choose among those outputs. The cost of each extra key is the configured minimum stake. A minimum of zero does not resist that. Off-chain, the indexer can copy those accounts into PostgreSQL and the read API can serve them. That copy is not protocol truth. `GET /v1/prices` returns a median of fresh public observations for SOL, BTC, and ETH. USD and USDT are not mixed. A short or stale set returns no median. Programs still reject a Price request.

Do not deploy this tree to mainnet. `scripts/deploy-mainnet.sh` exits before any transaction.

## Toolchain

| Component     | Version                               | Why                                                                       |
| ------------- | ------------------------------------- | ------------------------------------------------------------------------- |
| Anchor        | 1.2.0                                 | Latest stable release. 2.0.0-rc.1 is not stable.                          |
| Solana CLI    | 4.1.2                                 | Version named by the Anchor 1.2.0 release notes.                          |
| Solana crates | 3.x                                   | The crates `anchor-lang` 1.2.0 links. Agave 4.x crates are not that line. |
| Rust          | 1.89 MSRV, stable 1.98 in development | Anchor's `rust-version`.                                                  |
| Node.js       | 22                                    | Current LTS line used in this environment.                                |
| TypeScript    | 5.9.3                                 | Version used by the JavaScript SDK.                                       |
| pnpm          | 10.15.1                               | Package manager for `sdk/js`.                                             |

## Layout

This repository is the protocol. The website, the documentation site, and the off-chain services are separate repositories.

- `programs/` — `oracle-core`, `oracle-registry`, `verification`
- `crates/` — shared types, seeds, and the cryptography boundary
- `sdk/rust` and `sdk/js` — PDA helpers. The SDK still refuses to submit work
- `cli/` — command entry that still refuses to send. `network` reads `NUVEX_API_URL` when set
- `node/` — health-only oracle node
- `tests/` — LiteSVM, integration, and the callback consumer
- `env/` — example environment files
- `security/` — threat model and operational policy

Decision records live in the documentation repository, under `architecture/adr/`.

## Open decisions

These are blocked on purpose. The next milestone must not invent them in a pull request that only wants the code to compile.

1. **VRF audit.** The verifier is `solana-ecvrf` 0.0.1. No audit report was found. ADR 0004 records that absence.
2. **Challenge, reject, and fail edges.** Cancel, expire, and VRF fulfillment are the live transitions. ADR 0001 leaves every challenge edge forbidden.
3. **Fee shares and selection weights.** Roles exist. Basis points do not. `max_fee` is not collected. VRF fulfillment is first-come among nodes that meet the ADR 0003 predicate. There is no weighted lottery.

## Develop

```bash
make check
```

That runs formatting, Clippy, `cargo build-sbf` for the three programs and the test callback consumer, Rust tests, and the JavaScript SDK tests. Program tests load `target/deploy/*.so`. LiteSVM 0.17 loads the default SBF architecture from that command, because verification calls `sol_sha512`. It needs the Solana CLI and platform tools.

The node image is built from this repository:

```bash
docker build -f node/Dockerfile -t nuvex-oracle-node:local .
```

Copy an environment file before pointing a process at a cluster:

```bash
cp env/.env.local.example .env.local
```

RPC URLs, program ids, and key paths are variables. They are not defaults in source.

Development program keypairs, when generated, live in `keys/program/` and are gitignored. Public ids are in `Anchor.toml`. Those ids are local ids.

## Next milestone

Milestone 6: compute jobs and commit/reveal. Data jobs stay unspecified. Fees stay unset until ADR 0005 names basis points. No mainnet deployment.
