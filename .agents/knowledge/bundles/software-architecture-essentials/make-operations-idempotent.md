---
type: Practice
title: Make Operations Idempotent
description: When designing commands, lifecycle steps, or loops that may run amid crashes and retries, make each operation converge to the same end state regardless of how many times it runs. An idempotent operation is safe to retry; a non-idempotent operation accumulates side effects on each retry. Idempotency is the property that makes crash recovery possible.
tags: [architecture, idempotency, crash-recovery, retries, convergence, state-management]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: open-pstack-make-operations-idempotent
    resource: "https://github.com/ericlitman/open-pstack"
    title: "open-pstack poteto-mode — Make Operations Idempotent principle"
---

# Make Operations Idempotent

## The General Rule

An operation is **idempotent** if running it once produces the same
result as running it N times. When designing commands, lifecycle
steps, or loops that may run amid crashes and retries, idempotency is
the property that makes recovery safe: an idempotent operation can be
retried without accumulating side effects; a non-idempotent operation
duplicates side effects on each retry.

### Convergence, Not Just No-Op

Idempotency does not mean "do nothing on the second run." It means
**converge to the same end state.** An operation that creates a
resource if it does not exist and updates it if it does is idempotent
— it converges to "the resource exists with the desired state"
regardless of how many times it runs. An operation that appends a row
to a log on every run is not idempotent — it diverges (more rows on
each run).

### Where Idempotency Matters

- **Crash recovery** — if a crash interrupts a multi-step operation,
  the recovery re-runs the interrupted step. The step must be
  idempotent or the recovery duplicates the side effect.
- **Retries** — if a network failure causes a retry, the retried
  operation must produce the same result as the original. A
  non-idempotent retry double-charges, double-creates, or
  double-sends.
- **Loops with external state** — a loop that processes items from an
  external source may see the same item twice (at-least-once
  delivery). The processing must be idempotent or the item is handled
  twice.
- **Lifecycle steps** — a lifecycle step (init, start, stop, cleanup)
  may run multiple times if the lifecycle is interrupted and resumed.
  Each step must be idempotent or the lifecycle accumulates state.

### How to Achieve Idempotency

- **Check-then-act** — before performing the side effect, check
  whether it has already been performed. "Create if not exists" is
  idempotent; "create" is not.
- **Use deterministic identifiers** — if the operation creates a
  resource, use a deterministic identifier (derived from the input)
  so re-running the operation updates the same resource instead of
  creating a duplicate.
- **Make updates absolute, not relative** — "set the count to 5" is
  idempotent; "increment the count by 1" is not. Absolute updates
  converge; relative updates diverge.
- **Use event logs with fold** — instead of storing current state and
  updating it, append events to a log and derive current state by
  folding. Re-running the operation appends the same event (or a
  deduplicated event); the fold produces the same state.

## Concrete Instances

### Database Migration

A migration that adds a column: "ALTER TABLE ADD COLUMN IF NOT
EXISTS" is idempotent. Running it twice produces the same schema.
"ALTER TABLE ADD COLUMN" is not — the second run fails (column
already exists) or creates a duplicate.

### Resource Provisioning

A provisioning script that creates a resource: use a deterministic
name (derived from the input) and "create or update" semantics.
Re-running the script updates the existing resource instead of
creating a duplicate.

### Message Processing

A message handler that processes an event: use the event's
deduplication key to skip already-processed events. Re-delivering the
same event is a no-op. Without the key, the event is processed twice.

## Anti-Patterns

- **Relative updates** — "increment by 1" diverges on retry. Use
  absolute updates ("set to 5") or fold-based derivation.
- **Create without check** — "create" without "if not exists"
  diverges on retry. Use "create or update" with deterministic
  identifiers.
- **Assuming at-most-once delivery** — designing for exactly-once
  delivery when the system provides at-least-once. Design for
  idempotency instead.
- **Non-deterministic identifiers** — using random IDs for created
  resources. Re-running creates a duplicate. Use deterministic IDs.

## See Also

- [Restart-Proof State](https://github.com/levonk/skills-releases/blob/main/knowledge/agent-orchestration-practices/restart-proof-state.md)
  — idempotency is what makes restart-proof state work: re-running
  the fold on the same event log produces the same state.
- [Reversibility and Rollback First](reversibility-rollback-first.md)
  — idempotent operations are easier to roll back because re-running
  them does not compound the side effect.
