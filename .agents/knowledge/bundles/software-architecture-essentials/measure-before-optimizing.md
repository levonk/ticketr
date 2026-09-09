---
type: Practice
title: Measure Before Optimizing
description: Do not tune for speed until a measurement has proven where the bottleneck is, and even then only when one part of the code overwhelms the rest. Bottlenecks occur in surprising places; intuition about where time is spent is reliably wrong. The discipline is profiler-first, not hunch-first.
tags: [architecture, performance, profiling, optimization, measurement, premature-optimization]
date:
  created: "2026-08-27"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
---

# Measure Before Optimizing

## The General Rule

Optimization is a measured activity, not a designed one. Before changing code
for speed, a profiler or benchmark must have shown that the target is actually
the bottleneck. After the change, the same measurement must show the
improvement. Optimization without before-and-after measurement is guesswork
that adds complexity for unproven benefit.

- **Do not tune for speed until you have measured.** Intuition about where a
  program spends its time is reliably wrong. The slow function is rarely the
  one that *looks* expensive; it is often a small, innocuous call executed far
  more often than expected, or a hidden allocation on a hot path.
- **Even after measuring, only optimize when one part of the code overwhelms
  the rest.** A 20% speedup on a function that accounts for 5% of runtime
  yields a 1% overall gain — usually not worth the complexity it introduces.
  Concentrate on the part the profiler says dominates.
- **Bottlenecks occur in surprising places, so do not second-guess and insert
  a speed hack until you have proven that is where the bottleneck is.** A
  "fast" data structure added to a path that is not the bottleneck makes the
  code harder to read and slower to change, for no measurable gain. The speed
  hack is justified only by a measurement that names its target.

## Why It Matters

Unmeasured optimization has a negative expected value. The complexity it adds
is certain — more code, harder-to-read data structures, tighter coupling to a
performance hypothesis. The gain is speculative, and the hypothesis is usually
wrong: real bottlenecks cluster in places that are hard to predict from reading
the code (a logging call on a hot loop, a string concatenation in a formatter,
a lock held across I/O, a reflection lookup in a serializer).

The cost is not only the wasted optimization. Code shaped around a wrong
performance hypothesis becomes harder to fix when the real bottleneck is
found, because the speculative "fast" path is now load-bearing and must be
understood before it can be removed. Premature optimization is the root of
much evil precisely because it converts a hypothetical problem into a real
maintenance burden.

This is the operational form of Hoare's maxim — "premature optimization is the
root of all evil" — and Pike's first two rules. Hoare states the principle;
Pike states the discipline: measure, then optimize only the dominant part.

## How To Apply It

1. **Before optimizing, profile.** Run the workload under a profiler
   (CPU, allocation, lock contention) and read the flat and cumulative
   reports. The top of the flat report is where time is actually spent; the
   cumulative report shows which call paths feed it.
2. **Identify the dominant part.** Optimization pays off when one function or
   one call path accounts for a large fraction of total time. If time is
   spread evenly across many small contributors, no single optimization will
   move the needle — the system is balanced, and the right move is usually a
   different algorithm or data layout, not micro-tuning.
3. **Form a hypothesis, then test it.** "Function X is slow because it
   allocates per iteration" is a hypothesis. Test it by removing the
   allocation and re-measuring. If the profile does not improve, the
   hypothesis was wrong — revert and look again.
4. **Measure the improvement, not just the target.** A 10× speedup on the
   target function that accounts for 2% of runtime is a 0.2% overall gain.
   Report the overall number, not the local one.
5. **Stop when the dominant part is no longer dominant.** After the top
   bottleneck is fixed, re-profile. The next bottleneck is usually elsewhere.
   Do not chain speculative optimizations on the same code — each one should
   be justified by a fresh measurement.
6. **Prefer algorithmic and data-layout changes over micro-tuning.** A better
   algorithm or a more cache-friendly layout often yields order-of-magnitude
   gains with less complexity than a stack of micro-optimizations. See
   [Data Structures First](data-structures-first.md).

## When Simplicity Conflicts With Speed

A straightforward O(n) loop is simpler than a clever O(log n) structure —
until n grows. Optimize only when measurement shows the simple path is the
bottleneck; premature optimization adds complexity without proven benefit.
This is the same boundary stated in [KISS Principle](kiss-principle.md) — the
two principles agree: keep it simple until a measurement justifies the
complexity. See also [YAGNI](yagni-principle.md): a "fast" path built for a
hypothetical large n that never arrives is speculative code with negative
expected value.

## Anti-Patterns

- **The hunch-driven speed hack.** "This function *looks* slow, so I rewrote
  it with bit tricks." No profile was run. The function accounted for 1% of
  runtime; the rewrite made it unreadable and saved nothing measurable.
- **Optimizing the wrong level.** Micro-tuning a parser's inner loop while
  the real cost is a synchronous disk read per request. The profiler would
  have pointed at the I/O, not the loop.
- **Chasing the local number.** Reporting "I made function X 5× faster"
  without checking whether overall latency moved. The local win is real but
  irrelevant if X was not dominant.
- **Caching without measuring.** Adding a cache "because reads are slow" —
  but the reads were served from the OS page cache and were already
  sub-millisecond. The new cache adds invalidation complexity for no gain.
- **Optimizing for a workload that does not exist.** Tuning for a
  hypothetical large n when the measured n is small. See
  [KISS Principle](kiss-principle.md): fancy algorithms have big constants
  and are slow when n is small, and n is usually small.

## Concrete Instances

- **Rob Pike's 5 Rules of Programming (Rules 1-2).** "You can't tell where a
  program is going to spend its time. Bottlenecks occur in surprising places,
  so don't try to second guess and put in a speed hack until you've proven
  that's where the bottleneck is. Measure. Don't tune for speed until you've
  measured, and even then don't unless one part of the code overwhelms the
  rest." The rules are the operational restatement of Hoare's "premature
  optimization is the root of all evil."
- **Go `pprof`.** The Go toolchain ships a profiler that drives this
  discipline: `go test -cpuprofile`, `go tool pprof`, and the top/flat/list
  commands make the dominant function visible before any change is made. The
  standard library's design assumes you will profile before optimizing —
  there is no "fast path" added speculatively.
- **Rust `cargo bench` + `perf`.** Criterion benchmarks give before/after
  numbers; `perf record`/`perf report` show the hot function. The Rust
  performance book's first instruction is "measure before optimizing," and
  `clippy` warns against micro-optimizations that obscure intent without a
  measured justification.
- **Wirewiki autocomplete — the stopping rule in practice.** The API answers
  in ~2 ms at p50; the author stopped optimizing it there because the
  network round-trip (Browser → Cloudflare → origin) dominates the
  end-to-end latency. Shaving the API from 2 ms to 1 ms would be invisible
  behind a 60 ms CDN hop — the textbook case of optimizing a part that no
  longer overwhelms the rest. See
  [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) for
  the full case study.

## See Also

- [KISS Principle](kiss-principle.md) — keep it simple until a measurement
  justifies the complexity; fancy algorithms are slow when n is small.
- [YAGNI Principle](yagni-principle.md) — a "fast" path built for a
  hypothetical large n is speculative code with negative expected value.
- [Data Structures First](data-structures-first.md) — when measurement does
  justify optimization, prefer data-structure and layout changes over
  micro-tuning; the right structure makes the algorithm self-evident.
- [Scalability Fundamentals](scalability-fundamentals.md) — back-of-the-
  envelope capacity math is the *estimate* step; this page is the *measure*
  step that confirms or refutes the estimate before optimizing.
- [Root-Cause First](root-cause-first.md) — a "slow" symptom often has its
  root cause elsewhere; profile to find the cause, do not patch the symptom.
- [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) —
  once the backend fits the perception budget with headroom, the network
  becomes the dominant part; this page is the stopping rule that tells you
  when to shift effort.

## Sources

- Rob Pike, "Notes on Programming in C" (1989) — Rules 1 and 2.
- C.A.R. Hoare, quoted by Donald Knuth, "Structured Programming with go to
  Statements" (1974) — "premature optimization is the root of all evil."
- [p99 0 ms* autocomplete for 240 million domain names](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  — Ruurtjan Pul, 2026-06-22. The stopping rule in practice: the API was
  2 ms and the network was 60 ms, so further API tuning stopped.
