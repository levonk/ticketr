---
type: Practice
title: DRY + Rule of Three
description: Duplicate small, local logic when it preserves clarity; extract shared utilities only after the same pattern appears at least three times and has stabilized. When extracting, preserve module boundaries and avoid hidden coupling. The Rule of Three is the safety brake that prevents premature abstraction.
tags: [architecture, dry, rule-of-three, duplication, abstraction, coupling, maintainability]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# DRY + Rule of Three

## The General Rule

**DRY** (Don't Repeat Yourself) says every piece of knowledge must have a
single, unambiguous, authoritative representation within a system. **The Rule
of Three** says you may copy code twice, but on the third time, extract.

The two rules are a pair, not alternatives. DRY without the Rule of Three
produces premature abstractions — shared utilities extracted from a single
occurrence, coupling modules that had no reason to depend on each other. The
Rule of Three without DRY produces copy-paste sprawl that drifts. Together
they say: **duplicate until the pattern is proven, then extract once.**

- **Duplicate small, local logic when it preserves clarity.** Two copies of
  a three-line validation check in different modules are clearer than a
  shared utility that introduces a dependency edge between them. The
  duplication is a signal, not a sin.
- **Extract shared utilities only after the same pattern appears at least
  three times and has stabilized.** "Stabilized" means the three occurrences
  have converged on the same shape — same inputs, same outputs, same error
  handling. If the three are still diverging, extraction will force a
  generalization that fits none of them.
- **When extracting, preserve module boundaries and avoid hidden coupling.**
  The extracted utility belongs in the module that owns the concept, not in
  a catch-all `utils` package. The extraction must not introduce a circular
  dependency or a hidden shared mutable state.

## Why The Rule Of Three

The first occurrence is just code. The second occurrence tells you a pattern
might exist. The third occurrence confirms it. Extracting before the third
risks generalizing from an incomplete sample — the "two cases" are often
special cases of a pattern that looks different once the third arrives.

The cost of waiting is low: two copies of a small block. The cost of
extracting too early is high: a shared utility with the wrong interface,
coupling that is hard to undo, and a generalization that must be refactored
when the real pattern emerges.

## What Counts As "The Same Pattern"

The three occurrences must share:

1. **The same intent.** Not just similar code — the same purpose. Two
   `forEach` loops are not the same pattern unless they are doing the same
   kind of work for the same kind of reason.
2. **The same inputs and outputs.** If one takes a string and the other takes
   a list, they are not the same pattern yet.
3. **The same error handling.** If one swallows errors and the other throws,
   extracting them into one function forces a choice that may be wrong for
   one of the call sites.

If any of these differ, the pattern has not stabilized. Wait.

## Extraction Discipline

When the third occurrence arrives and the pattern has stabilized:

- **Put the utility where the concept lives.** A date-parsing helper belongs
  in the date module, not in `utils`. A validation helper belongs in the
  validation module. This keeps the dependency graph acyclic and the
  ownership clear.
- **Name the utility after what it does, not after where it was extracted
  from.** `parseIsoDate` is a good name; `extractedFromInvoiceHandler` is
  not.
- **Do not introduce hidden coupling.** The extracted utility must not reach
  into the internal state of its callers. If it needs state, pass it as a
  parameter.
- **Update all three call sites.** Leaving one call site on the old
  copy-pasted version defeats the purpose — now there are four copies (the
  utility plus three call sites, one of which does not use it).

## Anti-Patterns

- **Premature extraction (Rule of One).** Extracting a utility from a single
  occurrence. The utility now has one caller and a generalized interface
  that is a guess. When the second real case arrives, it does not fit the
  guess.
- **The `utils` dumping ground.** A package named `utils` that contains
  unrelated helpers extracted from across the codebase. It has no concept
  owner, no coherent interface, and every module depends on it. This is DRY
  without module-boundary discipline.
- **DRY-driven generalization that fits no caller.** A function with five
  optional parameters, each added for a different caller, such that no
  caller uses all five. The function is "DRY" but each call site is harder
  to read than the original inline code.
- **Extracting before stabilization.** Three occurrences that look similar
  but have different error handling. The extracted function forces one error
  policy on all three; one of them was correct before and is now wrong.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Engineering Principles state: "Duplicate
  small, local logic when it preserves clarity; extract shared utilities
  only after the same pattern appears at least three times and has
  stabilized; when extracting, preserve module boundaries and avoid hidden
  coupling." The Zod row schemas were extracted into one-file-per-shape in
  `packages/core/src/schemas/` only after the same validation shape appeared
  across multiple DB query paths and route handlers.
- **Go `sort` package.** The `sort.Slice`, `sort.Search`, and `sort.Interface`
  patterns were not extracted from one use case — they emerged after the Go
  team saw the same sort-and-search pattern across the standard library three
  or more times. The extraction is minimal (three methods on `Interface`) and
  lives where the concept lives (`sort`), not in a `utils` package.
- **React hooks.** Custom hooks like `useFetch` or `useDebounce` are the
  canonical Rule-of-Three extraction in frontend code. The first component
  that fetches data inlines the logic; the second duplicates it; the third
  triggers extraction into a hook. Extracting on the first would produce a
  hook with the wrong return shape for the second component's needs.

## See Also

- [KISS Principle](kiss-principle.md) — duplication is sometimes the simpler
  choice; KISS and the Rule of Three agree.
- [YAGNI Principle](yagni-principle.md) — do not extract speculatively; YAGNI
  is the "do not" side of the Rule of Three.
- [SRP + ISP](srp-isp.md) — extracted utilities must have a single
  responsibility; a `utils` module that mixes concerns violates SRP.
- [Adapter + Provider Pattern](adapter-provider-pattern.md) — the decision to
  extract an interface follows the same logic: wait for the second concrete
  adapter before abstracting.
