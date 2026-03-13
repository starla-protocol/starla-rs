# Implementation Plan

## Current State

Built:

- repo bootstrap
- Rust crate
- concrete `domain`, `store`, `runtime`, and `http` module split
- seeded in-memory state
- core inspection and listing routes
- core mutation routes for enabled state already in scope
- `submit work`
- `delegate execution`
- tool listing and inspection
- tool invocation without approvals or artifacts
- `context snapshot`
- `execution snapshot`
- claim-aligned route and integration tests for current slice
- claimant scope docs

Not built:

- standalone external conformance execution for the current claim

## Target

Pass the seeded `Core + Tools` claim over `HTTP Binding v1`.

Do not activate excluded optional surfaces.

## Implementation Sequence

1. define internal core state and resource records
2. close remaining `Core + Tools` HTTP vectors against the seeded report
3. execute seeded conformance artifacts and close failures
4. only then broaden the claimant or bindings

## Acceptance

- `conformance/v1/claims/core-tools-http-claim-seed.md` remains honest
- `conformance/v1/reports/core-tools-http-report-seed.md` passes
- local implementation-side status is tracked in `CLAIM_STATUS.md`
- automated local claim path is `scripts/run-core-tools-http-claim.sh`

## Deferred

- `Stream Binding v1`
- approvals
- channels
- approval-gated tool behavior
- visible tool-derived context contribution
- artifacts at the tool boundary
- persistence
- provider integration
