---
type: Practice
title: Process-Boundary State Ownership
description: When a process cannot reliably distinguish "actively running elsewhere" from "orphaned by a crash" — because the work was started by a different process or input source (CLI, adapter, webhook, web UI, cron) — it must not autonomously mark that work as failed, cancelled, or abandoned based on a timer or staleness guess. Surface the ambiguous state to the user and provide a one-click action. Heuristics for recoverable operations (retry backoff, subprocess timeouts, hygiene cleanup of terminal-status data) remain appropriate; the rule is about destructive mutation of non-terminal state owned by an unknowable other party.
tags: [architecture, process-boundary, state-ownership, distributed-state, agent-runtime, safety, lifecycle, concurrency]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Process-Boundary State Ownership

## The General Rule

A process may only mutate state it owns. State is "owned" by a process when
that process started the work and can reliably observe its lifecycle. When
work is started by a different process or input source (a CLI invocation, a
webhook, a web UI action, a cron trigger, a platform adapter), the observing
process cannot reliably distinguish "actively running elsewhere" from
"orphaned by a crash." In that ambiguous state, the process must not
autonomously mark the work as failed, cancelled, or abandoned based on a
timer or staleness guess.

- **Do not autonomously mutate non-terminal state owned by an unknowable
  other party.** If a workflow run was started by the CLI and the server
  sees it as "stale," the server must not mark it as failed. The CLI may
  still be running it.
- **Surface the ambiguous state to the user and provide a one-click action.**
  Instead of autonomously deciding, present the state as "last seen N
  minutes ago, status uncertain" and offer a button: "Mark as abandoned."
  The user — who knows whether the CLI is still running — makes the decision.
- **Heuristics for recoverable operations remain appropriate.** Retry
  backoff, subprocess timeouts, and hygiene cleanup of terminal-status data
  (runs that are already `completed` or `failed`) are recoverable operations.
  The rule is about destructive mutation of non-terminal state — state that
  is still `running` or `paused`.

## Why It Matters

A timer-based cleanup that marks a running process as "failed" is a
destructive mutation. If the process is still running, it will eventually
write its result — but the database now says it failed, so the result is
either lost or written to a row that the system considers terminal. This
creates an inconsistent state that is hard to detect and harder to repair.

The problem is structural, not a bug: the observing process does not have the
information to make the decision. It cannot tell the difference between "the
CLI process crashed and the run is orphaned" and "the CLI process is still
running but the network is slow." Any timer-based guess will be wrong in one
of those cases.

## The Distinction: Recoverable vs Destructive

| Operation | Recoverable? | Autonomous? | Example |
|-----------|-------------|-------------|---------|
| Retry with backoff | Yes | Yes | A failed HTTP call retries after 1s, 2s, 4s |
| Subprocess timeout | Yes | Yes | A bash node that exceeds `timeout: 30000` is killed |
| Cleanup of terminal data | Yes | Yes | Deleting `completed` runs older than 7 days |
| Mark non-terminal run as failed | **No** | **No** | A timer marks a `running` run as `failed` after 30 min |
| Cancel a non-terminal run | **No** | **No** | A staleness check marks a `paused` run as `cancelled` |

The test: **can the operation be undone if the guess was wrong?** A retry can
be stopped. A timeout kills a subprocess that can be re-run. A cleanup of
terminal data loses nothing (the data is already terminal). But marking a
running process as failed cannot be undone — the process is still running,
and its eventual result will conflict with the terminal status.

## How To Apply

1. **Identify who owns the state.** Before mutating a status, ask: did this
   process start the work? If not, the state is owned by another party.
2. **If the state is non-terminal and owned by another party, do not
   autonomously mutate it.** Instead, surface the ambiguity to the user.
3. **Provide a one-click action for the user to decide.** The user knows
   whether the originating process is still running. Give them the
   information (last heartbeat, originating source, elapsed time) and a
   button to mark as abandoned.
4. **For terminal-status data, autonomous cleanup is safe.** A run that is
   already `completed` or `failed` can be cleaned up by a timer — the state
   is terminal, so the cleanup cannot conflict with a running process.
5. **For recoverable operations, autonomous action is safe.** Retry backoff
   and subprocess timeouts are recoverable — if the guess is wrong, the
   operation can be re-run.

## Anti-Patterns

- **The staleness sweeper.** A background job that marks any `running` work
  older than 30 minutes as `failed`. If the work was started by a long-running
  CLI process, the sweeper destroys the state while the process is still
  working.
- **The "I know you're done" guess.** A process that observes a heartbeat
  gap and marks the work as `cancelled`. The heartbeat gap may be a network
  issue, not a crash. The work may still be running.
- **The autonomous cancel on restart.** A server that, on startup, marks all
  `running` work from a previous session as `failed`. The work may have been
  started by a CLI that is still running independently of the server.
- **The cleanup that touches non-terminal state.** A cleanup job that is
  supposed to delete old `completed` runs but also deletes `running` runs
  that have no recent heartbeat. The `running` deletion is a destructive
  mutation disguised as hygiene.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Engineering Principles state: "When a
  process cannot reliably distinguish 'actively running elsewhere' from
  'orphaned by a crash' — typically because the work was started by a
  different process or input source (CLI, adapter, webhook, web UI, cron) —
  it must not autonomously mark that work as failed/cancelled/abandoned based
  on a timer or staleness guess. Surface the ambiguous state to the user and
  provide a one-click action." The CLI's orphan-cleanup precedent only
  cleans up terminal-status environments — it does not mark running work as
  abandoned.
- **GitHub Actions.** A workflow run started by a `push` event is owned by
  the Actions runner. If the runner loses contact, GitHub does not
  autonomously mark the run as `failed` — it waits for the runner's
  heartbeat timeout, then marks it as `cancelled` with a visible "runner
  lost contact" message. The user can re-run. The distinction: the timeout
  is long (hours), the state change is visible, and the user has a re-run
  action.
- **Kubernetes job controller.** A Job whose Pod is evicted is not
  autonomously marked as failed. The controller creates a new Pod (retry),
  and only after `backoffLimit` failures does it mark the Job as failed.
  The retry is recoverable; the final failure is after a proven sequence of
  attempts, not a staleness guess.

## See Also

- [Fail Fast + Explicit Errors](fail-fast-explicit-errors.md) — surface the
  ambiguous state explicitly; do not silently mutate it.
- [Reversibility + Rollback-First](reversibility-rollback-first.md) —
  autonomous mutation of non-terminal state is irreversible; the rollback
  path is unknowable.
- [Resilience Patterns](resilience-patterns.md) — retry backoff and
  subprocess timeouts are the recoverable operations that remain appropriate
  under this rule.
- [Error Classification Pattern](error-classification-pattern.md) — the
  ambiguous state is a classification problem: "stale" is not the same as
  "failed."
