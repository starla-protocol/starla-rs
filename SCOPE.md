# Scope

First Rust claimant scope:

- language: `Rust`
- claimed protocol version: `v1`
- claimed binding versions:
  - `HTTP Binding v1`
- claimed compliance profiles:
  - `Core + Tools`

Allowed:

- in-memory state
- single local process
- deterministic synthetic execution engine

Excluded:

- `Stream Binding v1`
- `Core + Approvals`
- `Core + Channels`
- approval-gated tool invocation
- visible terminal approval denial during tool invocation
- idempotent `invoke tool`
- emitted artifact behavior at the tool boundary
- artifact inspection on the public binding
- visible tool-derived contribution at the context boundary
- durability across restart
- provider integration
- workflow and automation behavior

Success condition:

- pass the seeded `Core + Tools` report without activating excluded optional surfaces
