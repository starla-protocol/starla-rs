# starla-rs

Rust reference claimant for `starla-protocol`.

## Status

- state: bootstrap
- target claim:
  - `Core`
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
- only the public surface needed for the seeded `Core` claim

Excluded:

- `Stream Binding v1`
- `Core + Approvals`
- `Core + Tools`
- `Core + Channels`
- durability across restart
- product UI and packaging

## Immediate Goal

Implement enough public HTTP behavior to satisfy:

- `conformance/v1/claims/core-http-claim-seed.md`
- `conformance/v1/reports/core-http-report-seed.md`

Implementation sequence:

- `IMPLEMENTATION_PLAN.md`

## Development

Run:

```bash
cargo run
```
