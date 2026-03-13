# Implementation Decisions

## Purpose

These decisions apply to the first Rust claimant only.

They exist to reduce avoidable implementation drift while keeping the claimant narrow.

## Style

- concrete by default
- explicit state enums and transition helpers
- small literal modules
- traits only at real seams
- async and I/O at the edges
- typed errors at domain and HTTP boundaries

## Chosen Libraries

- HTTP server: `axum`
- async runtime: `tokio`
- serialization: `serde`

## Not Chosen

- `ratatui`
- `postgres`
- any persistence crate
- any ORM
- any trait-heavy service or repository framework

## Runtime Shape

- single process
- in-memory state only
- deterministic synthetic execution engine
- no background worker system
- no plugin runtime

## State Shape

- explicit domain records for `agent definition`, `agent instance`, `session`, and `execution`
- explicit lifecycle enums
- concrete in-memory stores
- opaque externally visible IDs

## Persistence

No database is part of the first claimant.

If persistence becomes necessary later, evaluate `SQLite` before `Postgres`.

Do not add persistence before the seeded `Core` claim is proven.

## UI

No TUI is part of this repo.

Do not add `ratatui` unless this repo later takes on operator tooling as an explicit scope change.

## Testing

- implement against the seeded `Core` claim and report
- prefer route and state-machine tests over mock-heavy unit tests
- add conformance-aligned tests as the public surface appears

## Deferral Rule

If a decision does not change the first claimant scope, defer it.
