# starla-rs

Rust reference claimant for `starla-protocol`.

## Status

- state: early implementation
- target claim:
  - `Core + Tools`
  - `HTTP Binding v1`
- protocol repo:
  - `https://github.com/starla-protocol/starla-protocol`

## Scope

This repo targets the first Rust claimant defined by:

- `starla-protocol/FIRST_RUST_CLAIMANT_SCOPE.md`

Included:

- single-process HTTP daemon
- in-memory state
- deterministic synthetic execution behavior
- only the public surface needed for the seeded `Core + Tools` claim
- first `Core + Tools` HTTP slice in progress

Excluded:

- `Stream Binding v1`
- `Core + Approvals`
- `Core + Channels`
- durability across restart
- product UI and packaging

## Immediate Goal

Implement enough public HTTP behavior to satisfy:

- `conformance/v1/claims/core-tools-http-claim-seed.md`
- `conformance/v1/reports/core-tools-http-report-seed.md`

Implementation sequence:

- `IMPLEMENTATION_PLAN.md`
- `IMPLEMENTATION_DECISIONS.md`
- `CLAIM_STATUS.md`
- `RECOVERY.md`

Claim automation:

- `scripts/run-core-tools-http-claim.sh`
- `.github/workflows/core-tools-http-claim.yml`

## Development

Run:

```bash
cargo run
```
