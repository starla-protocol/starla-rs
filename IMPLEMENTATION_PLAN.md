# Implementation Plan

## Current State

Built:

- repo bootstrap
- Rust crate
- minimal HTTP daemon stub
- claimant scope docs

Not built:

- `Core` resource state
- `Core` HTTP routes
- `submit work`
- `delegate execution`
- `context snapshot`
- `execution snapshot`
- conformance execution

## Target

Pass the seeded `Core` claim over `HTTP Binding v1`.

Do not activate excluded optional surfaces.

## Implementation Sequence

1. define internal core state and resource records
2. implement in-memory stores for agent definitions, agent instances, sessions, and executions
3. implement deterministic synthetic execution lifecycle
4. implement core inspection and listing routes
5. implement core mutation routes
6. implement `submit work`
7. implement `delegate execution`
8. implement `context snapshot` and `execution snapshot`
9. execute seeded conformance artifacts and close failures

## Acceptance

- `conformance/v1/claims/core-http-claim-seed.md` remains honest
- `conformance/v1/reports/core-http-report-seed.md` passes

## Deferred

- `Stream Binding v1`
- approvals
- tools
- channels
- persistence
- provider integration
