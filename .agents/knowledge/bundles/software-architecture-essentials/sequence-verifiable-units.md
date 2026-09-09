---
type: Practice
title: Sequence Verifiable Units
description: Break multi-step work into small units that each end in a check. Verify each unit before starting the next. Order the delivery so the sequence proves itself — each step's verification depends on the prior steps passing. A unit without a check is a unit that can fail silently.
tags: [architecture, sequencing, verification, incremental-delivery, checkpoints, fail-fast]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: open-pstack-sequence-verifiable-units
    resource: "https://github.com/ericlitman/open-pstack"
    title: "open-pstack poteto-mode — Sequence Verifiable Units principle"
---

# Sequence Verifiable Units

## The General Rule

Multi-step work fails silently when the steps are not individually
verified. A ten-step migration where only the final result is checked
produces a single failure at step ten — and the failure could be in
any of the ten steps. The rule: **break the work into small units that
each end in a check. Verify each unit before starting the next.**

### Each Unit Ends in a Check

A unit without a check is a unit that can fail silently. The check is
what makes the unit verifiable — it is the proof that the unit
succeeded. The check can be:

- **A test** — the unit adds behavior; the test verifies the behavior.
- **A build** — the unit changes code; the build verifies it compiles.
- **A lint pass** — the unit changes style; the lint verifies
  conformance.
- **A manual inspection** — the unit produces a document; the
  inspection verifies it meets the structure.

The check must be defined as part of the unit, not added afterward. A
unit without a defined check is not a verifiable unit.

### Verify Before Proceeding

Do not start the next unit until the current unit's check passes. This
localizes failures: if unit five fails, you know the problem is in
unit five, not in units one through four (they passed) or units six
through ten (they have not started). The alternative — doing all ten
units and checking at the end — produces a single failure with no
localization.

### Order So the Sequence Proves Itself

Order the units so each step's verification depends on the prior steps
passing. This makes the sequence self-proving: if step N passes, steps
1 through N-1 are verified by transitivity. The alternative — ordering
units so later steps can pass even if earlier steps failed — produces
false confidence.

## Concrete Instances

### Migration in Stages

A database migration from schema A to schema B: break into units
(add new columns, backfill data, switch reads, switch writes, drop old
columns). Each unit ends in a check (schema matches expected, data
count matches, reads return correct results, writes produce correct
rows, old columns are gone). Verify each before the next.

### Refactoring in Passes

A refactoring that extracts a module: break into units (extract
interface, move implementation, update callers, remove old code). Each
unit ends in a check (interface compiles, implementation passes tests,
callers pass tests, old code is gone). Verify each before the next.

### Feature in Slices

A feature with multiple capabilities: build the thinnest end-to-end
slice first (one capability, full stack). Verify it works. Then add
capabilities one at a time, each ending in a check. This is vertical
slice delivery — each slice is a verifiable unit.

## Anti-Patterns

- **Big-bang verification** — doing all the work and checking at the
  end. One failure, no localization.
- **Units without checks** — a unit that "looks done" but has no
  defined verification. It can fail silently.
- **Out-of-order units** — later units that can pass even if earlier
  units failed. False confidence.
- **Skipping the check** — "I'll verify it later." Later never comes.
  The check is part of the unit, not a separate task.

## See Also

- [Root-Cause First](root-cause-first.md) — when a unit's check fails,
  diagnose the root cause before proceeding; do not work around it.
- [Fail Fast, Explicit Errors](fail-fast-explicit-errors.md) — the
  check should fail fast and explicitly, not silently degrade.
