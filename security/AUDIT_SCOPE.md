# Audit scope

No audit has been commissioned. This file is the scope a future auditor would be asked to cover. It is not a report.

## In scope, when the instructions exist

- `programs/oracle-core`
- `programs/oracle-registry`
- `programs/verification`
- PDA seeds in `crates/common`
- Fee and stake arithmetic
- Callback account binding
- Upgrade and pause authorities

## Out of scope for a chain audit

- The Next.js site
- The read API and indexer, except where a client could confuse them for finality
- Infrastructure Terraform, which currently creates no resources
- AI inference, which has no runtime

## Current revision

Milestone 0 has no instruction handlers. An audit of this revision would only confirm that absence. Schedule the audit after VRF verification and callbacks exist, and again before mainnet.
