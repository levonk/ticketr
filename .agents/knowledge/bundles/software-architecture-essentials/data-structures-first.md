---
type: Practice
title: Data Structures First
description: Choose the right data structures and organize the data well, and the algorithms will almost always be self-evident. Data structures, not algorithms, are central to programming. Write stupid code that uses smart objects — the complexity lives in the data, not the control flow.
tags: [architecture, data-structures, algorithms, data-modeling, design, simplicity]
date:
  created: "2026-08-27"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
---

# Data Structures First

## The General Rule

Get the data structures right and the code writes itself. Get them wrong and
no amount of clever algorithmics will rescue the program — the code will be
fighting the data layout at every step. The data model is the load-bearing
part of a design; the algorithms that operate on it are usually a consequence
of the structures chosen.

- **Choose the right data structures and organize the data well.** A set
  where you need membership tests, a map where you need lookups, a queue
  where you need ordering, a tree where you need hierarchy. The wrong
  structure forces the algorithm to compensate — a list used for membership
  turns an O(1) check into an O(n) scan, and the code grows defensive
  `contains` loops that exist only to paper over the structural mistake.
- **The algorithms will almost always be self-evident once the structures are
  right.** When the data is shaped to the problem, the code that walks it is
  short and obvious. When the data is shaped against the problem, the code is
  long, clever, and fragile — full of index arithmetic, special cases, and
  workarounds for the mismatch.
- **Write stupid code that uses smart objects.** Put the intelligence in the
  data structures (and the types that enforce their invariants); keep the
  control flow dumb. A function that delegates to a well-typed structure's
  methods is easier to read and change than a function that re-implements the
  structure's invariants inline with clever branching.

## Why It Matters

Data is the stable part of a system; algorithms change more often. A
representation chosen well survives feature growth, refactoring, and
performance work largely intact. A representation chosen poorly forces every
later change to wrestle with it — each new feature must work around the
structural mismatch, and the workarounds accumulate into the very complexity
the original design was trying to avoid.

The principle inverts the instinct that algorithm design is the hard,
interesting part and data layout is a detail. In practice the reverse is
true: the bulk of a program's difficulty is in representing the data so that
the operations the program needs are cheap and natural. Once that
representation is found, the operations are usually a line or two each.

This is Brooks' observation from *The Mythical Man-Month* — "Representation
is the essence of programming" — and Pike's fifth rule: "Data dominates. If
you've chosen the right data structures and organized things well, the
algorithms will almost always be self-evident."

## How To Apply It

1. **Design the data representation before the functions.** Before writing
   any logic, list the operations the program needs (lookups, insertions,
   ordered iteration, range queries, adjacency walks) and pick the structure
   that makes the frequent ones cheap. The structure is chosen by the
   workload, not by what is convenient to declare.
2. **Make illegal states unrepresentable.** Use types and structure to make
   invalid data impossible to construct. A `PaymentStatus` enum with
   `Pending`/`Settled`/`Failed` makes a "settled then failed" transition a
   type error, not a runtime check. The invariant lives in the type, not in
   every function that touches the value.
3. **Prefer smart structures over smart code.** When a function is growing
   long and full of special cases, ask whether the data is mis-shaped. A
   40-line function with five branches often collapses to 10 lines when the
   input is restructured so the branches become unnecessary.
4. **Let the structure drive the iteration.** If you are maintaining a
   parallel index or a manual cross-reference, ask whether a different
   structure (a map, a graph, an inverted index) would make the relationship
   implicit. The right structure makes the relationship a property of the
   data; the wrong structure makes it a property the code must rebuild.
5. **When measurement justifies optimization, change the structure first.**
   A faster algorithm on the same structure yields a constant-factor gain; a
   structure better matched to the access pattern can yield an order of
   magnitude. See [Measure Before Optimizing](measure-before-optimizing.md)
   — but when you do optimize, the structure is the highest-leverage change.

## Anti-Patterns

- **The clever algorithm over a mis-shaped structure.** A binary search over
  a sorted list where a hash map would give O(1) lookup. The algorithm is
  "optimal" for the chosen structure, but the structure was the wrong choice
  — the cleverness is compensating for a structural error.
- **Smart code over dumb data.** A 60-line function that manually tracks
  state across a flat list, where a small state machine or a typed enum would
  make the transitions explicit and the function trivial. The complexity is
  in the code because it was not put in the data.
- **The parallel index.** Maintaining a side map from `id → record` by hand,
  re-syncing it on every mutation, when the records could be stored in a map
  keyed by `id` directly. The manual index is a workaround for a structure
  that does not match the access pattern.
- **Representing illegal states.** A `User` with `is_active: bool` and
  `deleted_at: Option<DateTime>` where `is_active == true && deleted_at.is_some()`
  is a nonsensical combination the code must guard against. A single status
  enum makes the nonsense unrepresentable.

## Concrete Instances

- **Rob Pike's 5 Rules of Programming (Rule 5).** "Data dominates. If you've
  chosen the right data structures and organized things well, the algorithms
  will almost always be self-evident. Data structures, not algorithms, are
  central to programming." Often shortened to "write stupid code that uses
  smart objects."
- **Fred Brooks, *The Mythical Man-Month*.** "Show me your flowcharts and
  conceal your tables, and I shall continue to be mystified. Show me your
  tables, and I won't usually need your flowcharts; they'll be obvious."
  Brooks states the principle a decade and a half before Pike: the data
  representation is the design.
- **Rust's `enum` + `match`.** A tagged enum makes the set of states
  explicit and the compiler enforces that every `match` covers all variants.
  Illegal states are unrepresentable; the "smart" lives in the type, the
  "stupid" (a flat `match`) lives in the code. This is data-structures-first
  at the language level.
- **Relational database design.** Normalization is data-structures-first at
  the storage level: get the schema right (no redundant, updateable data;
  keys where keys belong) and the queries are straightforward; get the
  schema wrong and the queries become contortions that re-derive the
  relationships the schema failed to capture.
- **Wirewiki autocomplete — two structures for two access patterns.** 240
  million domain names are split into a hot head (Tranco top 1 M, looked up
  first) and a cold tail (CZDS, consulted only on miss). The head lives in
  an in-memory character trie with the top 8 suggestions precomputed at
  every node — a lookup is a walk of a few pointers, O(prefix length). The
  tail is sorted, delta-compressed into fixed-size blocks on SSD with a
  27 MB in-memory directory; a lookup binary-searches the directory then
  linearly scans one 256-name block, O(prefix length × log N). Both inputs
  (query length, domain count) are bounded, so both worst cases are
  effectively O(1). The structure was chosen to fit a perception budget,
  not to be clever — see
  [Perceived-Latency-Driven Design](perceived-latency-driven-design.md).

## See Also

- [Measure Before Optimizing](measure-before-optimizing.md) — when
  optimization is justified, the structure is the highest-leverage change;
  measure first, then change the data layout.
- [KISS Principle](kiss-principle.md) — smart structures and stupid code is
  the KISS-compatible form: the complexity is concentrated in one place (the
  data) rather than spread across the control flow.
- [SRP + ISP](srp-isp.md) — a well-shaped structure has a single
  responsibility and exposes a narrow interface; the structure's invariants
  are its responsibility.
- [Architecture Philosophy](philosophy.md) — domain-based modular structure
  is data-structures-first at the system level: organize the code around the
  data's domain boundaries.
- [Database Scaling](database-scaling.md) — the storage-level instance of
  this principle: schema and data model choices dominate query and scaling
  complexity.
- [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) —
  the access pattern that drives the structure choice can be a perception
  budget, not just a workload profile; the trie + block-index split exists
  to fit inside a human typing cadence.

## Sources

- Rob Pike, "Notes on Programming in C" (1989) — Rule 5.
- Fred Brooks, *The Mythical Man-Month* (1975) — "Show me your tables..."
- [p99 0 ms* autocomplete for 240 million domain names](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  — Ruurtjan Pul, 2026-06-22. In-memory trie + mmap block index chosen so
  both worst cases are effectively O(1) under bounded inputs.
