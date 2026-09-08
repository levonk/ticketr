---
type: Practice
title: Error Classification Pattern
description: Map raw errors from external systems (git, network, filesystem, SDK) to user-friendly, actionable messages via a single classification function. Always log the raw error for debugging and send the classified message to the user. The classifier is the single source of truth for error-to-message mapping; it distinguishes known infrastructure failures (blocked, user-facing) from classifiable programming bugs (crash, not absorbed).
tags: [architecture, error-handling, error-classification, user-experience, logging, observability, agent-runtime]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Error Classification Pattern

## The General Rule

When a system interacts with external systems (git, network, filesystem,
SDKs, databases), the raw errors those systems produce are not suitable for
end users. A raw `EACCES` or `permission denied while trying to connect to
the docker daemon` is accurate but not actionable — the user needs "Start
Docker and retry, or run without `--container`." The classification pattern
maps raw errors to user-friendly messages via a single function that is the
source of truth for the mapping.

- **Map raw errors to user-friendly messages via a single classification
  function.** The classifier takes a raw error and returns a human-readable,
  actionable message. It is the only place in the codebase that performs
  this mapping.
- **Always log the raw error for debugging.** The raw error (with stack
  trace, error type, and context) goes to the log. The user never sees the
  raw error; the developer always can.
- **Send the classified message to the user.** The user sees the
  classified, actionable message — not the raw error, not a generic
  "something went wrong."
- **Distinguish known infrastructure failures from programming bugs.** A
  `known: true` error is a recognized infrastructure/config failure that
  produces a user-facing "blocked" message. A `known: false` error is
  classifiable (we have a helpful message) but still a programming or
  user-input bug that should crash, not be absorbed as a blocked state.

## Why It Matters

Without classification, the user sees either the raw error (unhelpful,
possibly leaking internal details) or a generic "an error occurred"
(unhelpful, not actionable). With classification, the user sees a message
that tells them what went wrong and what to do about it.

The classification function is also a maintainability win: when a new error
type appears (a new SDK version changes an error message), the classifier is
the only file that needs updating. Without a central classifier, error
handling is scattered across the codebase and each call site must be updated
individually.

## The Classifier Structure

The classifier is a function that:

1. **Takes a raw error** (an `Error`, an exception, a status code).
2. **Matches the error against a pattern table.** Each pattern is a substring
   or error code that identifies a known failure mode.
3. **Returns a classified result.** The result includes the user-friendly
   message and a flag indicating whether the error is a known infrastructure
   failure or a programming bug.
4. **Falls through to a generic message for unrecognized errors.** An
   unrecognized error is still classified (the user gets a message), but it
   is logged as "unrecognized" so the classifier can be updated.

The pattern table is ordered: more specific patterns are checked before more
general ones. A "docker daemon permission denied" pattern is checked before
a generic "permission denied" pattern, so the docker-specific message wins.

## How To Apply

1. **Inventory the external systems your code interacts with.** Git, Docker,
   network, filesystem, SDKs, databases.
2. **Collect the raw errors each system produces.** Look at logs, test
   failures, and real incidents. Each distinct error type gets a pattern.
3. **Write a user-friendly message for each.** The message says what went
   wrong and what to do. "Permission denied while creating workspace. Check
   file system permissions." is good. "EACCES" is not.
4. **Order the patterns from most specific to most general.** The classifier
   checks specific patterns first so they win over generic ones.
5. **Log the raw error, send the classified message.** Every catch block
   that uses the classifier does both: `log.error({ err })` then
   `platform.send(classify(err))`.
6. **Mark known infrastructure failures vs programming bugs.** A known
   infrastructure failure (`known: true`) is a blocked state — the user is
   notified and the operation stops. A programming bug (`known: false`) is
   classifiable but should crash, not be absorbed.

## Anti-Patterns

- **Scattered error messages.** Each catch block writes its own user-facing
  message. The same raw error produces different messages in different call
  sites. The classifier does not exist.
- **The generic "something went wrong."** The user sees a message that
  conveys no information. The raw error is logged but the user cannot act on
  it.
- **The raw error leaked to the user.** The user sees `EACCES` or a stack
  trace. This is not actionable and may leak internal details (file paths,
  internal module names).
- **The classifier that absorbs programming bugs.** A classifier that maps
  a `TypeError: cannot read property 'id' of undefined` to a user-friendly
  message and continues. This is a programming bug — it should crash, not be
  absorbed as a "blocked" state.
- **The unordered pattern table.** A generic "permission denied" pattern is
  checked before a specific "docker permission denied" pattern. The generic
  message wins and the user gets the less helpful message.

## Concrete Instances

- **Archon (Bun + TypeScript).** The isolation package has a
  `classifyIsolationError()` function that maps git and Docker errors to
  user-friendly messages. The pattern table is ordered: Docker-specific
  patterns ("cannot connect to the docker daemon") are checked before
  generic patterns ("permission denied"). Each pattern has a `known` flag:
  `known: true` means a recognized infrastructure failure (blocked,
  user-facing); `known: false` means a classifiable programming bug (crash,
  not absorbed). Every catch block logs the raw error and sends the
  classified message to the user.
- **Stripe error codes.** Stripe's API returns structured error objects with
  `type`, `code`, and `message`. The `code` (e.g.,
  `card_declined`, `expired_card`, `insufficient_funds`) is the
  classification key. The Stripe SDK maps each code to a user-friendly
  message. The raw error is in the API response; the user sees the
  classified message. This is the classification pattern at the API level.
- **Rust `thiserror` + `anyhow`.** Libraries define typed errors with
  `thiserror` (each error variant has a message). Applications use `anyhow`
  for ad-hoc error context. The typed errors are the classification: a
  caller matches on the variant to produce a user-friendly message. The raw
  error chain is preserved for logging via `{:?}`. This is the
  classification pattern at the language level.

## See Also

- [Fail Fast + Explicit Errors](fail-fast-explicit-errors.md) —
  classification is not swallowing; the raw error is always logged and
  programming bugs still crash.
- [Process-Boundary State Ownership](process-boundary-state-ownership.md) —
  the `known` flag distinguishes infrastructure failures (blocked) from
  programming bugs (crash); the same distinction applies to state mutation.
- [Structured Logging](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/structured-logging.md)
  — the raw error is logged with structured fields for debugging.
- [Resilience Patterns](resilience-patterns.md) — classified errors feed
  into circuit breaker and retry decisions.
