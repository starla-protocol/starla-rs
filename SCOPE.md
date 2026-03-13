# Scope

First Rust claimant scope:

- language: `Rust`
- claimed protocol version: `v1`
- claimed binding versions:
  - `HTTP Binding v1`
- claimed compliance profiles:
  - `Core`

Allowed:

- in-memory state
- single local process
- deterministic synthetic execution engine

Excluded:

- `Stream Binding v1`
- `Core + Approvals`
- `Core + Tools`
- `Core + Channels`
- durability across restart
- provider integration
- workflow and automation behavior

Success condition:

- pass the seeded `Core` report without activating excluded optional surfaces
