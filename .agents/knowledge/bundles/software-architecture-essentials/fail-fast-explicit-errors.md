---
type: Practice
title: Fail Fast + Explicit Errors
description: Prefer throwing early with a clear error for unsupported or unsafe states — never silently swallow errors. Never silently broaden permissions or capabilities. Document fallback behavior with a comment when a fallback is intentional and safe; otherwise throw. Silent fallback in agent runtimes and automated systems can create unsafe or costly behavior.
tags: [architecture, fail-fast, explicit-errors, error-handling, safety, agent-runtime, defensive-programming]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Fail Fast + Explicit Errors

## The General Rule

Fail at the earliest point where an invalid state is detected, and make the
failure explicit — a thrown error, a non-zero exit, a rejected promise. Never
silently swallow an error, never silently broaden a permission, and never
silently fall back to a less-safe behavior. If a fallback is intentional and
safe, document it with a comment; otherwise, throw.

- **Prefer throwing early with a clear error for unsupported or unsafe
  states.** The earlier the throw, the closer it is to the cause, and the
  easier it is to diagnose. A throw at the boundary is better than a `null`
  that propagates five layers deep and crashes somewhere unrelated.
- **Never silently swallow errors.** A `catch` block that does nothing
  (or only logs) hides a failure. The system continues in an invalid state
  and the failure surfaces later as an unrelated symptom.
- **Never silently broaden permissions or capabilities.** If a user lacks a
  permission, reject the action. If a provider lacks a capability, report it.
  Silently downgrading a restricted operation to an unrestricted one is a
  security hole.
- **Document fallback behavior with a comment when a fallback is intentional
  and safe; otherwise throw.** A fallback is acceptable only when (a) the
  fallback is strictly safer than the primary, (b) the user is informed, and
  (c) the fallback is documented in a comment so a future reader knows it is
  deliberate.

## Why It Matters In Automated Systems

In a human-driven system, a silent failure is annoying — the human notices
something is wrong and investigates. In an automated system (an agent
runtime, a CI pipeline, a background job), a silent failure is dangerous —
no human is watching, and the system continues as if nothing happened. The
cost of the failure compounds: a silently swallowed error in step 1 becomes
an invalid input to step 2, which produces an invalid output to step 3, and
the symptom surfaces at step 5 with no traceable connection to step 1.

This is especially critical in agent runtimes where a silent fallback can
cause the agent to take an action the user did not authorize — for example,
silently broadening a permission scope, or silently falling back from a
restricted tool set to an unrestricted one.

## How To Apply

1. **Validate at the boundary.** Check inputs at the entry point — the API
   handler, the CLI argument parser, the config loader. Throw immediately on
   invalid input. Do not pass invalid input deeper into the system.
2. **Throw, do not return null.** A null return value forces every caller to
   check for null, and some will forget. A thrown error propagates
   automatically and cannot be silently ignored (unless explicitly caught).
3. **Include the cause in the error message.** The error message should tell
   the reader what went wrong, what input caused it, and what to do about it.
   `"Invalid config"` is not enough; `"config.schema_version is 99, max
   supported is 3 — upgrade the tool"` is.
4. **Log the raw error, send a classified message to the user.** The raw
   error (stack trace, internal details) goes to the log for debugging. The
   user sees a classified, actionable message. See
   [Error Classification Pattern](error-classification-pattern.md).
5. **When a fallback is intentional, document it.** A comment that says
   `// Fallback to SQLite when DATABASE_URL is unset — safe for local dev`
   tells a future reader the fallback is deliberate. Without the comment, the
   reader cannot distinguish a deliberate fallback from a bug.

## The Distinction: Fallback vs Swallow

A **fallback** is a deliberate choice to use an alternative when the primary
is unavailable. It is safe when the alternative is strictly less capable but
still correct (e.g., falling back from Postgres to SQLite for local dev).

A **swallow** is an accidental suppression of an error. It is never safe
because the system continues in a state that the code did not design for.

The test: if you can write a comment explaining why the fallback is safe, it
is a fallback. If you cannot, it is a swallow — throw instead.

## Anti-Patterns

- **The empty catch.** `catch (e) {}` — the error is gone, the system
  continues, and the symptom surfaces far from the cause.
- **The log-and-continue.** `catch (e) { log.error(e); }` — the error is
  logged but the system continues in an invalid state. Logging is not
  handling.
- **The silent permission broadening.** A capability check that fails and
  silently grants the capability anyway "to keep things working." This is a
  security hole disguised as resilience.
- **The null return on error.** A function that returns `null` on failure
  instead of throwing. Every caller must check; some will not; the null
  propagates and crashes far from the cause.
- **The fallback without a comment.** A `try { primary() } catch { fallback() }`
  with no comment explaining why the fallback is safe. A future reader cannot
  tell whether it is deliberate or a bug.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Engineering Principles state: "Prefer
  throwing early with a clear error for unsupported or unsafe states — never
  silently swallow errors; never silently broaden permissions or
  capabilities; document fallback behavior with a comment when a fallback is
  intentional and safe; otherwise throw." The output schema validation
  enforces this: a node that declares `output_format` but returns no
  schema-valid output **fails** rather than degrading silently. The database
  adapter throws on invalid config rather than silently falling back to a
  default that might lose data.
- **Rust `Result<T, E>`.** The language encodes fail-fast at the type level.
  A function that can fail returns `Result`, and the caller must handle the
  `Err` case — the compiler enforces it. There is no "silent swallow" because
  ignoring a `Result` is a compile error (with `#[must_use]`). This is
  fail-fast as a language feature.
- **Python `failfast` in `unittest`.** The test runner has a `--failfast`
  flag that stops on the first failure. Without it, a failure in test 1
  cascades into unrelated failures in tests 2–50, hiding the root cause. The
  flag encodes the principle: stop early, diagnose the first failure, do not
  let it cascade.

## See Also

- [Error Classification Pattern](error-classification-pattern.md) — how to
  map raw errors to user-friendly messages while preserving the raw error
  for debugging.
- [Process-Boundary State Ownership](process-boundary-state-ownership.md) —
  fail-fast applies to state mutation: do not silently mutate state owned by
  another process.
- [Reversibility + Rollback-First](reversibility-rollback-first.md) —
  fail-fast and rollback-first are complementary: fail early so the rollback
  path is simple.
- [Root-Cause First](root-cause-first.md) — a silent fallback is often a
  workaround that masks the root cause.
