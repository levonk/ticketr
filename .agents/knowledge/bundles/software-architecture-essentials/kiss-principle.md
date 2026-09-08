---
type: Practice
title: KISS — Keep It Simple, Stupid
description: Prefer straightforward control flow over clever meta-programming; favor explicit branches and typed interfaces over hidden dynamic behavior; keep error paths obvious and localized. Simplicity is a property of the design, not of the implementation language or framework.
tags: [architecture, kiss, simplicity, control-flow, readability, maintainability]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-27"
  last-used: "2026-08-27"
---

# KISS — Keep It Simple, Stupid

## The General Rule

Prefer the simplest design that solves the problem at hand. Simplicity is
measured by how long it takes a new reader to understand the code, not by how
few lines it has. The rule applies at every level: control flow, data flow,
module boundaries, and system topology.

- **Prefer straightforward control flow over clever meta-programming.** A
  sequence of `if`/`else` branches is easier to audit than a reflection-driven
  dispatch table. Reach for metaprogramming only when the cost of not using it
  (boilerplate volume, drift across hand-written copies) exceeds the cost of
  the indirection it introduces.
- **Prefer explicit branches and typed interfaces over hidden dynamic
  behavior.** Dynamic dispatch, plugin registries, and "magic" decorators
  hide what code runs. When behavior is selected at runtime, surface the
  selection point and type the interface so the reader can follow the path.
- **Keep error paths obvious and localized.** A reader should be able to find
  every failure mode of a function by reading the function — not by tracing
  through five layers of callbacks. Centralized error handling is acceptable
  only when the centralizer is the single obvious place to look.

## Why It Matters

Complexity compounds. Each layer of indirection multiplies the number of
states a reader must hold in their head. A design that is "clever" at one
layer becomes opaque at the next, and debugging an opaque system costs orders
of magnitude more than writing it. The cost is paid every time someone touches
the code — not just once at authoring time.

Simplicity is not the same as brevity. Compressing logic into a one-liner with
a ternary chain is shorter but harder to read than a four-line `if`/`else`.
The goal is the fewest concepts per unit of code, not the fewest characters.

## When Simplicity Conflicts With Other Principles

- **KISS vs DRY.** Duplicating a small, local block is simpler than extracting
  a shared utility that introduces a new module boundary and a coupling edge.
  See [DRY + Rule of Three](dry-rule-of-three.md) — extract only after the
  pattern has stabilized and appeared three times.
- **KISS vs abstraction.** An abstraction is simpler than its concrete
  alternatives only when the reader already understands the abstraction. A
  novel abstraction imposes a learning tax on every new reader. Prefer the
  concrete until the abstraction is proven.
- **KISS vs performance.** A straightforward O(n) loop is simpler than a
  clever O(log n) data structure — until n grows. Optimize only when
  measurement shows the simple path is the bottleneck; premature optimization
  adds complexity without proven benefit.

## Anti-Patterns

- **Reflection-driven dispatch where a switch would do.** A registry keyed by
  string that resolves to a handler at runtime is harder to trace than a
  typed `switch` the compiler can check. Use registries when the set of
  handlers is open (plugins); use `switch` when it is closed.
- **Decorators that hide control flow.** A `@retry` decorator that silently
  re-invokes a function on failure hides the retry loop from the reader. If
  the retry is important, make it visible — either inline or in a named
  wrapper the reader can navigate to.
- **"Smart" constructors that infer configuration.** A constructor that
  inspects its environment and picks defaults silently makes the system's
  behavior depend on where it runs. Pass configuration explicitly; infer only
  when the inference is documented and overridable.
- **Over-generalized generics.** A function parameterized over five type
  variables to cover a hypothetical future case is harder to read than one
  written for the current case. See [YAGNI](yagni-principle.md).

## Concrete Instances

- **Archon (Bun + TypeScript).** The project's Engineering Principles list
  KISS first: "Prefer straightforward control flow over clever
  meta-programming; prefer explicit branches and typed interfaces over hidden
  dynamic behavior; keep error paths obvious and localized." The workflow
  engine uses a typed `switch` over node types rather than a reflective
  dispatch table, and route registration goes through a single typed wrapper
  so the reader can find every API route in one place.
- **Go standard library.** The `net/http` handler is a single method
  (`ServeHTTP`) on an interface — no annotations, no reflection, no base
  class. A reader can follow a request from `ListenAndServe` to the handler
  in one jump. This is KISS at the language-design level.
- **Rust `clippy`.** The linter warns against overly clever constructs
  (`needless_collect`, `manual_map`, `option_map_unit_fn`) — not because they
  are wrong, but because the simpler form is easier to read. The linter
  encodes KISS as a mechanical check.
- **Rob Pike's 5 Rules of Programming (Rules 3-4).** "Fancy algorithms are
  slow when n is small, and n is usually small. Fancy algorithms have big
  constants. ... Fancy algorithms are buggier than simple ones, and they're
  much harder to implement." Ken Thompson rephrased this as "When in doubt,
  use brute force." The simple algorithm is the KISS choice until measurement
  proves n is big and the simple path is the bottleneck — see
  [Measure Before Optimizing](measure-before-optimizing.md).

## See Also

- [YAGNI Principle](yagni-principle.md) — do not add complexity for
  hypothetical futures; the two principles reinforce each other.
- [DRY + Rule of Three](dry-rule-of-three.md) — extract only after the third
  occurrence; premature extraction violates KISS.
- [SRP + ISP](srp-isp.md) — focused modules and narrow interfaces are the
  structural expression of simplicity.
- [Root-Cause First](root-cause-first.md) — a workaround stack is often a
  KISS violation in disguise; the simple path is usually the root-cause fix.
- [Measure Before Optimizing](measure-before-optimizing.md) — the simple
  algorithm is the KISS choice until a measurement proves the simple path is
  the bottleneck; Pike's rules 3-4 are KISS applied to algorithm choice.
- [Data Structures First](data-structures-first.md) — smart structures and
  stupid code is the KISS-compatible form; the complexity lives in the data,
  not the control flow.
