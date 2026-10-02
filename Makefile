.PHONY: fmt lint test check build keys secrets

export PATH := $(HOME)/.local/share/solana/install/active_release/bin:$(PATH)

fmt:
	cargo fmt --all
	pnpm format

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	pnpm format:check
	pnpm lint

test:
	# Core depends on the other programs with no-entrypoint. A single cargo
	# invocation unifies that feature and strips their entrypoints, so the
	# programs that are loaded as binaries are built on their own afterwards.
	cargo build-sbf -- -p nuvex-oracle-core -p nuvex-callback-consumer
	cargo build-sbf -- -p nuvex-oracle-registry
	cargo build-sbf -- -p nuvex-verification
	cargo test --workspace
	pnpm test

check: lint test
	bash scripts/check-secrets.sh

build:
	cargo build --workspace
	pnpm build

keys:
	python3 scripts/generate-program-keys.py

secrets:
	bash scripts/check-secrets.sh
