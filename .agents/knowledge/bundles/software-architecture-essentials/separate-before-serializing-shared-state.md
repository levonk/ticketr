---
type: Practice
title: Separate Before Serializing Shared State
description: When concurrent actors might write to the same file, branch, key, or object, eliminate the sharing first — then serialize. Serialization (locks, mutexes, transactions) is the fallback when sharing cannot be eliminated. Separation (partitioning, ownership, copy-on-write) removes the contention at the source. Serialize the unavoidable; separate the avoidable.
tags: [architecture, concurrency, separation, serialization, contention, partitioning, ownership]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: open-pstack-separate-before-serializing
    resource: "https://github.com/ericlitman/open-pstack"
    title: "open-pstack poteto-mode — Separate Before Serializing Shared State principle"
---

# Separate Before Serializing Shared State

## The General Rule

When concurrent actors might write to the same file, branch, key, or
object, the instinct is to add a lock — serialize access so only one
actor writes at a time. The rule says: **eliminate the sharing first,
then serialize.** Separation removes the contention at the source;
serialization manages the contention that remains.

### Separation Removes Contention

Separation means giving each actor its own copy of the state, so they
never contend:

- **Partitioning** — divide the state by key so each actor owns a
  partition. No two actors write to the same partition.
- **Ownership** — assign each piece of state to one actor. Only the
  owner writes; others read a copy.
- **Copy-on-write** — each actor works on its own copy and merges
  after. No contention during the work; contention only at merge.
- **Per-actor output paths** — each actor writes to its own file or
  directory. No file is written by two actors.

### Serialization Manages Remaining Contention

After separation, some sharing may remain (the merge step in
copy-on-write, a shared registry that assigns ownership). Serialize
this sharing — but it is now the exception, not the rule. The
serialization surface is small and well-defined.

### Why Separation First

Serialization has costs:

- **Contention** — serialized access creates a bottleneck. Actors wait
  for the lock instead of doing work.
- **Deadlock risk** — multiple locks introduce deadlock potential.
  Separation removes the need for multiple locks.
- **Complexity** — lock-based code is harder to reason about than
  separation-based code. The actor does not need to think about
  concurrency if it owns its state.

Separation eliminates these costs for the state that can be
separated. Serialization is reserved for the state that cannot.

## Concrete Instances

### Per-Branch Worktrees

Multiple agents working on the same repo: instead of serializing
access to one working directory (one agent waits while the other
commits), give each agent its own worktree on its own branch. No
contention during work; serialization only at merge time.

### Per-Task Output Files

Multiple tasks producing reports: instead of serializing access to
one report file, give each task its own output path
(`reports/{task-slug}.md`). No contention; serialization only if a
combined report is needed.

### Partitioned Queues

Multiple consumers processing messages: instead of one queue with a
lock, partition the queue by key so each consumer owns a partition.
No contention; the partition assignment is the only serialized step.

### Copy-on-Write for Shared Data

Multiple readers of a shared data structure: instead of locking the
structure for each read, copy the structure on write. Readers use the
old copy; the writer produces a new copy. No contention between
readers and the writer.

## Anti-Patterns

- **Lock first** — adding a lock before considering whether the
  sharing can be eliminated. The lock creates contention and
  complexity that separation would avoid.
- **Shared mutable state** — multiple actors writing to the same
  object with a lock. The lock manages the contention but does not
  eliminate it. Separate instead.
- **Coarse-grained locks** — one lock for a large section of state.
  The lock serializes everything, even independent operations.
  Partition and use finer-grained separation.
- **Merge without separation** — all actors write to the same branch
  and rely on merge conflict resolution. The conflicts are the
  serialization, applied late and expensively. Separate first (per-
  actor branches), merge the clean results.

## See Also

- [Process Boundary State Ownership](process-boundary-state-ownership.md)
  — ownership is a form of separation: each process owns its state.
- [Resilience Patterns](resilience-patterns.md) — the canonical
  invariant (no unbounded concurrency) is easier to enforce when
  state is separated; serialization is a choke point that can become
  a bottleneck.
