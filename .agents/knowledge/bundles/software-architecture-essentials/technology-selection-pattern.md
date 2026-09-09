---
type: Practice
title: Technology Selection Pattern
description: Convention for authoring technology-selection concept pages — state the requirement, list candidates, apply the tech-decision-risk hierarchy, recommend one, document why-not for the others, and state when to reconsider. Follows the auth-provider-selection and resilience-patterns precedent.
tags: [architecture, technology-selection, decision-making, convention, pattern, risk-assessment]
date:
  created: "2026-07-29"
  knowledge-basis: "2026-07-29"
  last-used: "2026-07-29"
sources:
  - id: auth-provider-selection-precedent
    resource: "../api-auth-payment-practices/auth-provider-selection.md"
    title: "Auth Provider Selection — the first worked example of this pattern"
  - id: resilience-patterns-precedent
    resource: "resilience-patterns.md"
    title: "Resilience Patterns — the second worked example (cross-language library matrix)"
  - id: tech-decision-risk-assessment
    resource: "tech-decision-risk-assessment.md"
    title: "Tech Decision Risk Assessment — the risk hierarchy applied in step 3"
---

# Technology Selection Pattern

## Failure Mode

Technology decisions documented as "we chose X" with no record of the
candidates considered, the risk trade-offs, or the conditions under which the
decision should be revisited. This produces:

- **Cargo-cult inheritance.** New projects copy the choice without
  understanding why it was made. When the original constraint no longer
  applies, the choice persists.
- **Re-litigation.** Without a documented why-not for the rejected
  alternatives, every new team member reopens the same decision.
- **Stale decisions.** Without a "when to reconsider" section, a choice made
  under constraints that have since changed (beta feature shipped, license
  changed, performance improved) is never revisited.
- **Single-axis arguments.** "It's faster" or "it's cheaper" without placing
  the choice on the risk hierarchy, so a low-risk option is rejected as "too
  much change" or a high-risk option is accepted because the upfront cost
  looks small.

## Practice

When a knowledge bundle needs a concept page that answers "which technology
should I use for X?", follow this structure:

### Required sections

1. **Failure Mode** — What goes wrong without this guidance (the generic
   failure mode is above; specialize it to the domain).

2. **Requirement** — State the problem the technology must solve. One
   paragraph. Do not name candidates here; describe the need.

3. **Candidates** — List the realistic options. Include the status quo if it
   is a viable candidate. For each, note its key properties (license, language
   support, maintenance status, maturity).

4. **Risk Assessment** — Apply the
   [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) hierarchy.
   Place each candidate on the hierarchy. The decision's risk is the highest
   level it touches, not the average.

5. **Recommendation** — Name the chosen technology. State why it wins on the
   risk hierarchy, not on preference.

6. **Why Not the Others** — For each rejected candidate, state the specific
   reason. One sentence each. This prevents re-litigation.

7. **When to Reconsider** — List the conditions under which the decision
   should be revisited. Be concrete: "if candidate Y graduates from beta",
   "if the team size drops below N", "if compliance requirement Z appears".

8. **See Also** — Cross-link to the risk hierarchy page, related concept
   pages, and any worked examples.

### Optional sections

- **Preference Ordering** — When the choice is not binary but a ranked list
  (e.g., auth methods, protocols), order from most to least preferred.
- **Verification** — When the choice requires proving a property (e.g., RLS
  enforcement, connection pool behavior), list the tests that must pass.
- **Per-language matrix** — When the technology is language-specific (e.g.,
  resilience libraries), include a matrix of recommended libraries per
  language runtime.

## Worked Examples

Two concept pages in the knowledge base follow this pattern:

- [Auth Provider Selection](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md)
  — better-auth vs. Supabase Auth. Applies the risk hierarchy (level 2:
  end-user impact for auth migration on paying users). Documents why-not
  Supabase Auth (passkey-first not supported, beta API). States when to
  reconsider (all four conditions must be true).

- [Resilience Patterns](resilience-patterns.md) — service mesh vs. per-language
  libraries vs. both. Applies the risk hierarchy (level 6: running a 3rd-party
  service for the mesh; level 11-12: new 3rd-party package for per-language
  libraries). Includes a per-language matrix. Documents why-not a single
  universal library (structurally impossible across concurrency models).

## When to Write a Selection Page

Write a technology-selection concept page when:

- The decision is **reusable** — multiple projects or services will face the
  same choice.
- The decision has a **non-obvious winner** — if the answer is "use the
  standard library", a page is not needed.
- The decision has a **reversibility cost** — migration risk, lock-in, or
  compliance surface that makes a wrong choice expensive.
- The decision sits on the **risk hierarchy** at level 6 or above — running a
  3rd-party service, modifying 3rd-party source, or impacting end users.

Do not write a selection page when:

- The choice is trivial and reversible (new constant, new function).
- The choice is project-specific and will not recur.
- The choice is a procedure (use a skill, not a knowledge bundle).

## Relationship to the Risk Hierarchy

The
[Tech Decision Risk Assessment](tech-decision-risk-assessment.md) is the
evaluation framework. This pattern is the **documentation format** for
decisions that use that framework. A selection page without the risk
hierarchy is an opinion; a selection page with the risk hierarchy is a
defensible decision record.

## See Also

- [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) — the risk
  hierarchy applied in step 4 of this pattern.
- [AI + Human Timeline Estimates](ai-human-timeline-estimates.md) — why
  upfront cost is reassessed as bounded under AI-assisted development (relevant
  to the "Recommendation" section).
- [Root-Cause First](root-cause-first.md) — the discipline behind documenting
  why-not, not just why.
- [Auth Provider Selection](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md)
  — worked example.
- [Resilience Patterns](resilience-patterns.md) — worked example with
  per-language matrix.

## Sources

- [Auth Provider Selection](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md)
  — the first concept page to follow this pattern (2026-07-18).
- [Resilience Patterns](resilience-patterns.md) — the second concept page,
  with a cross-language library matrix (2026-07-29).
- [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) — the risk
  hierarchy that step 4 applies.
