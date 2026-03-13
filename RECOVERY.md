# Recovery

Use this file after context loss.

Keep it narrow.

Update it only when:

- the target claim changes
- the working branch changes
- the verification command changes
- the immediate next step changes materially

## Current Target

- repo: `starla-rs`
- branch: `implement/core-http-claimant`
- claim: `Core`
- binding: `HTTP Binding v1`

## Read Order

1. `README.md`
2. `AGENTS.md`
3. `IMPLEMENTATION_DECISIONS.md`
4. `IMPLEMENTATION_PLAN.md`
5. `CLAIM_STATUS.md`

## Verify Current State

Run:

```bash
git branch --show-current
git status --short
./scripts/run-core-http-claim.sh
```

Expected:

- branch is `implement/core-http-claimant`
- working tree is clean
- claim script passes

## Current Boundary

- do not broaden beyond the seeded `Core` HTTP claim without an explicit scope change
- do not add approvals, tools, channels, stream, persistence, or product UI work here
- keep module boundaries aligned with `domain`, `store`, `runtime`, and `http`

## Immediate Next Step

After a green claim run:

- turn the provisional local claim status into a dated implementation report, or
- explicitly choose the next claimant scope before adding new runtime surface
