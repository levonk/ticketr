---
type: Practice
title: Exhaust the Design Space
description: When facing a novel interaction or architectural decision with no precedent in the codebase, build two or three competing prototypes and compare them before committing. The first design that looks reasonable is not the best — it is the first. Exhausting the design space means forcing yourself to see alternatives before the sunk-cost bias locks you in.
tags: [architecture, design, prototyping, design-space, alternatives, decision-making]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: open-pstack-exhaust-design-space
    resource: "https://github.com/ericlitman/open-pstack"
    title: "open-pstack poteto-mode — Exhaust the Design Space principle"
---

# Exhaust the Design Space

## The General Rule

For a novel interaction or architectural decision with no precedent in
the codebase, the first design that clears the "plausible" bar is not
the best design — it is the first one you thought of. Committing to it
immediately locks you into a structure you have not compared against
anything. The rule is simple: **build two or three competing prototypes
and compare them before committing.**

"Novel" is the key qualifier. For decisions with precedent — a new
endpoint that follows the existing API pattern, a new module that
follows the existing project structure — the design space is already
exhausted by the existing convention. Follow the convention. This rule
applies only when there is no convention to follow.

### Competing, Not Cosmetic

The prototypes must be **structurally distinct** — different module
boundaries, different data ownership, different control flow. Two
prototypes that differ only in naming or file layout are the same
design. The point of exhausting the design space is to see different
decompositions of the same problem, not to rearrange the same
decomposition.

### Compare Before Committing

After building the prototypes, compare them against criteria defined
**before** the prototypes were built. Defining criteria after seeing
the prototypes retrofits the rubric to justify a preferred option. The
comparison should produce a clear winner or a clear signal that the
problem is not yet understood (wild divergence between prototypes).

### Sunk-Cost Bias

The reason to exhaust the design space before committing is sunk-cost
bias. Once you have built one design, even as a prototype, you are
psychologically invested in it. Building a second design feels like
wasted effort. But the cost of building the wrong design into
production is higher than the cost of building two prototypes. The
prototypes are disposable; the production code is not.

## When to Apply

- **Novel architecture** — a new subsystem with no existing pattern to
  follow.
- **New interaction model** — a user interaction or API shape that
  does not exist elsewhere in the codebase.
- **High reversibility cost** — a decision that will be expensive to
  reverse once committed (data model, public API, module boundary).

## When Not to Apply

- **Followed convention exists** — the codebase has a pattern for this
  kind of decision. Follow it.
- **Low reversibility cost** — the decision is easy to change later.
  Pick one and move.
- **Trivial decision** — the design space is small enough that the
  alternatives are obvious without building prototypes.

## See Also

- [DRY — Don't Repeat Yourself](dry-rule-of-three.md) — the rule of
  three is the threshold for extracting a shared abstraction; exhaust
  the design space is the threshold for when to compare alternatives
  before committing.
- [YAGNI — You Aren't Gonna Need It](yagni-principle.md) — do not
  build for hypothetical futures; exhaust the design space for the
  current problem, not for imagined variants.
