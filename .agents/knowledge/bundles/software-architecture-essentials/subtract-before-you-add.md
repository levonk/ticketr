---
type: Practice
title: Subtract Before You Add
description: When sequencing an addition, refactor, or rewrite, remove the dead weight first — then build on the simpler base. Adding to a cluttered codebase compounds the clutter; subtracting first makes the addition smaller and the result cleaner. The order matters: subtract, then add.
tags: [architecture, subtraction, refactoring, dead-code, simplification, sequencing]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: open-pstack-subtract-before-add
    resource: "https://github.com/ericlitman/open-pstack"
    title: "open-pstack poteto-mode — Subtract Before You Add principle"
---

# Subtract Before You Add

## The General Rule

When sequencing an addition, refactor, or rewrite, the instinct is to
start building the new thing. The rule says: **remove the dead weight
first, then build on the simpler base.** Adding to a cluttered codebase
compounds the clutter — the new code navigates around the old dead
code, the dead code constrains the design of the new code, and the
result is larger and harder to understand than either the old or new
code alone.

### What to Subtract

- **Dead code** — functions, modules, and files that are no longer
  called. They add navigation cost and constrain the design space.
- **Redundant abstractions** — interfaces with one implementation,
  wrappers that add no behavior, config options that are always set
  to the same value. They add indirection without value.
- **Expired workarounds** — code that worked around a bug or
  limitation that has since been fixed upstream. The workaround is now
  dead weight (or worse, it masks the fix).
- **Premature generality** — abstractions built for hypothetical
  futures that never arrived. They generalize the wrong axis.

### The Order Matters

Subtract first, then add. The reverse order — add first, then
subtract — produces a larger diff, a larger review surface, and a
higher risk of the subtraction never happening (the new code starts
depending on the old dead code, making it harder to remove). By
subtracting first, the addition is built on a clean base and is smaller
than it would have been on the cluttered base.

### Subtraction Is Harder Than Addition

Adding code feels productive; removing code feels destructive. But
removing dead code is one of the highest-leverage changes you can make:
it reduces the codebase's surface area, makes the remaining code easier
to understand, and removes constraints on future design. The
psychological barrier is the reason this rule needs to be explicit —
without it, the instinct is always to add.

## When to Apply

- **Before a refactor** — remove the dead code that the refactor would
  otherwise navigate around.
- **Before a feature addition** — remove the expired workarounds and
  redundant abstractions that the new feature would otherwise
  incorporate.
- **Before a rewrite** — remove everything that will not survive the
  rewrite, so the rewrite starts from the minimal base.

## When Not to Apply

- **Stable, working code** — do not subtract code that is in active use
  just because it looks old. Subtraction is for dead weight, not for
  aesthetic preferences.
- **Time-critical fixes** — if the fix is urgent, subtract later. But
  track the subtraction as a follow-up task.

## See Also

- [KISS — Keep It Simple, Stupid](kiss-principle.md) — subtraction is
  one of the primary tools for keeping a design simple.
- [YAGNI — You Aren't Gonna Need It](yagni-principle.md) — preventing
  premature generality is subtraction at the design stage; this rule
  is subtraction at the maintenance stage.
