# starla-rs Rules

## Purpose

`starla-rs` is the first Rust claimant for `starla-protocol`.

The code should be a clear, practical reference implementation.

## Architecture Style

- keep the code meticulous and clean
- prefer clear boundaries over cleverness
- keep modules small and responsibility-aligned
- keep the code concrete by default
- use explicit state, explicit transitions, and explicit errors
- design boundaries for real runtime needs, not for diagram aesthetics
- favor boring, testable code over framework ceremony

## Avoid

- enterprise-style controller or service or repository layering without a real seam
- generic `Manager`, `Kernel`, `Engine`, or `Platform` abstractions
- trait-heavy indirection before multiple credible implementations exist
- state represented as loose strings, maps, or boolean bags
- premature database, queue, or plugin abstractions
- UI or operator tooling concerns leaking into the claimant runtime

## Default Boundary Shape

- `domain` owns nouns, states, and protocol-facing records
- `store` owns concrete in-memory state and mutations
- `runtime` owns synthetic progression and background behavior
- `http` owns route mapping, request parsing, and response mapping

## Working Rule

If a new abstraction does not make the claimant clearer, smaller, or more realistic, do not add it.
