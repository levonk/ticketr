---
type: Practice
title: YAGNI — You Aren't Gonna Need It
description: Do not add config keys, interface methods, feature flags, or workflow branches without a concrete accepted use case. Do not introduce speculative abstractions without at least one current caller. Keep unsupported paths explicit (error out) rather than adding partial fake support.
tags: [architecture, yagni, speculative-abstraction, feature-flags, interface-design, maintainability]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# YAGNI — You Aren't Gonna Need It

## The General Rule

Build only what is needed now. Every line of code, every config key, every
interface method, and every feature flag is a liability — it must be tested,
documented, understood, and eventually changed or removed. Code written for a
hypothetical future is code that must be maintained today for a benefit that
may never arrive.

- **Do not add config keys, interface methods, feature flags, or workflow
  branches without a concrete accepted use case.** "Someone might want this"
  is not a use case. A use case has a caller, a scenario, and an accepted
  decision to build it.
- **Do not introduce speculative abstractions without at least one current
  caller.** An abstraction with zero callers is a guess. The guess may be
  wrong, and a wrong abstraction is harder to fix than no abstraction —
  because removing it means changing every caller that was forced through it.
- **Keep unsupported paths explicit (error out) rather than adding partial
  fake support.** A method that returns a dummy value for an unsupported case
  is worse than a method that throws. The dummy value hides the gap; the
  throw surfaces it.

## Why It Matters

Speculative code has a negative expected value. The probability that a guess
about future requirements is exactly right is low; the cost of maintaining
the guess until it is proven or disproven is certain. When the real
requirement arrives, it almost always differs from the guess — and the guess
must be refactored away before the real code can be written.

The damage is not limited to the speculative code itself. An interface with
extra methods forces every implementer to provide them. A config key with no
consumer forces every operator to wonder whether they should set it. A
feature flag with no feature forces every reader to trace both paths.

## How To Apply It

1. **Before adding an interface method, ask: who calls it today?** If the
   answer is "nobody yet," do not add it. Add it when the first caller
   appears — and add it for that caller, not for a generalized version of
   that caller.
2. **Before adding a config key, ask: what behavior changes when it is set?
   Who sets it?** If the behavior is the same with and without the key, the
   key is dead weight.
3. **Before adding a feature flag, ask: is the feature behind it complete and
   tested?** A flag that gates a half-built feature creates a hidden code
   path that no one tests. Ship the feature complete, or do not ship the
   flag.
4. **Before extracting a shared utility, ask: how many current call sites
   share this exact pattern?** If fewer than three, duplication is cheaper.
   See [DRY + Rule of Three](dry-rule-of-three.md).
5. **Before generalizing a function, ask: does the generalization serve a
   current caller, or a hypothetical one?** Generalize for the second
   concrete caller, not for the imagined third.

## The Meta-Application

YAGNI applies to the development process itself. Do not create a shared
include file, a new bundle, or a new tooling layer speculatively — create it
when a concrete third use case arrives. A process artifact built for one use
case is a guess; built for three, it is a pattern.

## Anti-Patterns

- **The "flexible" interface with methods nobody calls.** An interface with
  ten methods where only three are ever implemented is a YAGNI violation that
  also violates ISP — see [SRP + ISP](srp-isp.md).
- **The config key that does nothing.** `ENABLE_ADVANCED_MODE=true` that
  enables nothing because the advanced mode was never built. Operators set
  it, nothing happens, and a future reader must trace the dead flag to
  confirm it is dead.
- **The partial fake support.** A `deleteAll()` method that deletes one item
  because "we might need bulk delete later." The caller thinks it deletes
  all; it deletes one. A throw (`"bulk delete not supported"`) is safer.
- **The speculative abstraction layer.** A "plugin system" with one plugin
  (the built-in one). The plugin system exists for a second plugin that was
  never written. The indirection costs readability for zero extensibility
  benefit.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Engineering Principles state: "Do not
  add config keys, interface methods, feature flags, or workflow branches
  without a concrete accepted use case; do not introduce speculative
  abstractions without at least one current caller; keep unsupported paths
  explicit (error out) rather than adding partial fake support." The provider
  registry registers only providers that have concrete implementations
  (Claude, Codex, Pi); the capability tiers (`enforced`/`best-effort`/`false`)
  describe actual provider behavior, not hypothetical tiers.
- **Stripe API.** Stripe ships endpoints when they are needed, not before.
  The API has no "generic payment method" abstraction that predicts future
  payment types — each payment method type (card, bank_transfer, link) is a
  concrete addition with its own resource. YAGNI at the API design level.
- **Rust `std` evolution.** The standard library famously resisted adding an
  `async` runtime for years because no single design had proven dominant.
  `tokio` and `async-std` competed in the ecosystem; only after the pattern
  stabilized did `std` begin to absorb the proven parts. YAGNI at the
  language level.

## See Also

- [KISS Principle](kiss-principle.md) — simplicity and YAGNI reinforce each
  other; speculative code is almost always unnecessary complexity.
- [DRY + Rule of Three](dry-rule-of-three.md) — the Rule of Three is the
  YAGNI-compatible version of DRY: do not extract until the third case.
- [SRP + ISP](srp-isp.md) — speculative interface methods violate ISP; a
  narrow interface with no speculative methods is the YAGNI-compliant form.
- [Reversibility + Rollback-First](reversibility-rollback-first.md) —
  speculative code increases blast radius; smaller scope is easier to revert.
