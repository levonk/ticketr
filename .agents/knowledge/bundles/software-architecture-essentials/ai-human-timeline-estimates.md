---
type: Practice
title: AI + Human Timeline Estimates
description: Timelines must be based on AI + human collaborative estimates, not previous human-only estimates. Pre-AI time estimates (days/weeks of human work) are no longer valid units — a unit of work that took 3 days of human time in 2022 may take hours of AI-assisted time in 2026. Estimate as "human review + AI execution" pairs, never as "human days" alone.
tags: [estimation, timelines, ai-assisted-dev, planning, anti-patterns]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
sources:
  - id: bookkeeping-saas-mvp
    resource: internal-docs/feature/2026/07/bookkeeping-saas-mvp/
    title: "bookkeep-saas decision dialogue where pre-AI estimates were corrected"
---

# AI + Human Timeline Estimates

## Failure Mode

Estimating work using pre-AI human-only time units ("3 days of engineering",
"2-4 weeks of work", "a sprint"). These estimates were calibrated to a world
where the human wrote the code, ran the tests, debugged the failures, and
iterated. With AI-assisted development, the human reviews and the AI executes
— the cost axis has shifted, and old estimates systematically overstate the
upfront cost of changes that are now AI-cheap.

This failure mode produces two kinds of bad decisions:

1. **Rejecting low-risk, AI-cheap changes** because their pre-AI cost estimate
   is high. Example: rejecting better-auth from day one because "the
   session-variable RLS middleware is 3-5 days of engineering" — when the
   middleware is a well-trodden pattern that an AI agent can scaffold in
   hours, leaving only verification as the real cost.
2. **Accepting high-risk, AI-expensive changes** because their pre-AI cost
   estimate looks comparable. Example: treating "migrate auth later" as
   cheap because the code is similar — when the migration's real cost is
   unbounded tail risk on paying users, which AI assistance does not reduce
   (AI cannot absorb a user lockout event for you).

## Practice

Estimate every unit of work as a **human review + AI execution** pair, never
as a "human day" alone. The two costs are on different axes and do not add
linearly.

### The cost axes

| Axis | What it measures | How it scales with AI |
|------|-----------------|----------------------|
| **AI execution cost** | Wall-clock time for the AI to produce the code / run the command / iterate | Collapses for well-trodden patterns; stays high for novel work |
| **Human review cost** | Wall-clock time for the human to verify correctness, security, fit | Stays roughly constant; may rise for higher-risk changes |
| **Verification cost** | Wall-clock time to prove the change is correct (tests, contract checks, migration plans) | Stays constant or rises — AI can write tests fast, but proving correctness is still human-bound |
| **Tail risk cost** | Expected loss if the change goes wrong in production | Unchanged by AI — AI cannot absorb user lockout, data loss, or trust damage |

### Estimation format

For every unit of work, state:

```
AI execution: <hours | days>      // wall-clock for the AI to produce it
Human review: <hours | days>      // wall-clock for the human to verify it
Verification: <hours | days>      // wall-clock for tests / contracts / migration plans
Tail risk:    <bounded | unbounded> // expected loss if it goes wrong
```

Example — "build the session-variable RLS middleware for better-auth":

```
AI execution: 1-3 hours      // well-trodden pattern, reference implementations exist
Human review: 2-4 hours      // verify the middleware sets context correctly
Verification: 4-8 hours      // write and run the isolation tests (concurrent tenants, pool reuse, fail-closed)
Tail risk:    bounded        // middleware bug is caught by tests before deploy
```

Example — "migrate auth from Supabase Auth to better-auth on a live product":

```
AI execution: 1-2 days       // rewriting RLS policies, Edge Functions, Storage, Realtime
Human review: 3-5 days       // review every rewritten surface
Verification: 5-10 days      // migration rehearsals, rollback plans, dual-run period
Tail risk:    unbounded      // user lockout during close period = churn + trust damage
```

The first change is **AI-cheap and bounded**. The second is **AI-cheap but
unbounded**. Pre-AI estimation treated them as similar because both involve
"writing auth code". The AI + human split exposes that the second change's
real cost is the tail risk, which AI does not reduce.

### What does NOT collapse with AI

- **Verification of security-critical code.** AI can write a session-variable
  RLS middleware in hours. Proving it actually enforces isolation under
  connection pooling, concurrent tenants, and fail-closed behavior is still
  human-bound — the human must design the test that would catch the silent
  cross-tenant leak.
- **Migration risk on live systems.** AI cannot absorb a user lockout event.
  The tail risk of an auth migration is the same whether a human or an AI
  writes the migration code.
- **Novel work.** Things nobody has done before (level 1 on the risk
  hierarchy) have no reference implementation for the AI to draw on. AI
  execution cost stays high; human review cost rises because the human cannot
  rely on prior art either.
- **Compliance and audit work.** Documenting security controls for an audit,
  running a pen test, responding to an incident — these are human-bound and
  AI does not speed them up meaningfully.

### What DOES collapse with AI

- **Well-trodden patterns.** Anything with a reference implementation the AI
  can find: standard middleware, standard CRUD, standard auth flows, standard
  test scaffolds. AI execution drops from days to hours.
- **Boilerplate.** Configuration, wiring, type definitions, migration
  skeletons. AI execution drops from hours to minutes.
- **Iteration speed.** "Try it, see if it works, fix the error" loops run at
  AI speed, not human speed. This collapses the cost of exploratory work.
- **Test generation.** AI writes test cases fast. (Verification — proving the
  tests are the *right* tests — stays human-bound.)

### Anti-patterns to avoid

- **"3-5 days of engineering"** — pre-AI unit. Replace with the four-axis
  estimate above.
- **"It's just an hour of AI coding"** — understates verification and tail
  risk. AI execution is one axis, not the whole cost.
- **"We'll migrate later"** — assumes migration cost is constant. It is not:
  migration risk on paying users is unbounded and does not collapse with AI.
- **"It's too expensive to do now"** — check whether "expensive" is measured
  in pre-AI human days or in the four-axis estimate. A change that was
  expensive in 2022 may be AI-cheap in 2026.
- **"It's cheap to do now"** — check the tail risk axis. A change can be
  AI-cheap and still have unbounded tail risk (e.g., a quick auth migration).

## Interaction with the risk hierarchy

The risk hierarchy in
[Tech Decision Risk Assessment](tech-decision-risk-assessment.md) is the
primary axis — it determines *which* path is lower-risk. This estimate format
is the secondary axis — it determines *how much* the upfront cost of each
path actually is, given AI-assisted dev.

A decision is the conjunction of the two:

1. Pick the lower-risk path (per the hierarchy).
2. Estimate its upfront cost using the four-axis format (not pre-AI human
   days).
3. If the lower-risk path's upfront cost is genuinely prohibitive *on the
   four axes*, reconsider — but the bar for "prohibitive" is higher than it
   was in 2022, because AI execution has collapsed for well-trodden patterns.

## Worked example: better-auth from day one

- **Risk hierarchy**: better-auth now is level 14 + level 20 (facade + new
  internal algorithm). Supabase Auth now + migrate later is level 2
  (end-user UI impact). Lower-risk path: better-auth now.
- **Pre-AI estimate**: "3-5 days of engineering for the middleware" →
  sounds expensive → bias toward Supabase Auth now.
- **AI + human estimate**: AI execution 1-3 hours, human review 2-4 hours,
  verification 4-8 hours, tail risk bounded. Total: ~1-2 days of
  human+AI collaborative work, not 3-5 days of human-only work.
- **Decision**: better-auth now. The lower-risk path is also the
  AI-cheap path. See
  [Auth Provider Selection](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md).

## Related Concepts

- [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) — The
  risk hierarchy this estimate format pairs with
- [Root-Cause First](root-cause-first.md) — Diagnose before estimating;
  estimates applied to the wrong root cause are wasted
- [Auth Provider Selection](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md)
  — Worked example of this estimate format in action
