---
type: Practice
title: Scalability Fundamentals
description: Distinguish performance from scalability, latency from throughput, and use back-of-the-envelope calculations with current hardware latency figures to reason about capacity.
tags: [architecture, scalability, performance, latency, throughput, capacity, back-of-the-envelope]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
---

# Scalability Fundamentals

## Performance vs Scalability

- **Performance** problem — slow for a single user.
- **Scalability** problem — fast for one user, slow under load.
- A service is **scalable** when adding resources produces proportional
  performance gains.

Don't optimize single-request latency when the real risk is load saturation.
Conversely, a system that is slow for one user won't become scalable by adding
servers.

## Latency vs Throughput

| Metric | Definition | Tradeoff |
|--------|-----------|----------|
| **Latency** | Time to complete one action | Lower often means less batching |
| **Throughput** | Actions per unit time | Higher often means more batching or parallelism |

Aim for **maximal throughput with acceptable latency**. More throughput is not
always better — batching increases throughput but raises per-request latency.
Parallelism reduces latency only if the bottleneck is not already saturated.

## Back-of-the-Envelope Capacity Math

Use powers of two and latency tables for quick estimates. Use **current
hardware figures** — old tables (2012-era: L1 0.5 ns, SSD random read 150 µs,
1 Gbps network) are off by 10-100× for modern NVMe, 100 Gbps datacenter
networks, and multi-core caches.

### Latency reference (2024-2025 hardware)

```
Operation                                   Time
---------------------------------------------------
L1 cache reference                         ~1 ns
Branch mispredict                          ~5 ns
L2 cache reference                         ~7 ns
Mutex lock/unlock                         ~25 ns
Main memory reference                    ~100 ns
Compress 1K bytes (Zstd)                  ~3 µs
NVMe random 4K read                      ~20 µs
Round trip within datacenter (100 Gbps) ~100 µs
Read 1 MB sequentially from NVMe        ~200 µs
Read 1 MB sequentially from 10 Gbps      ~1 ms
HDD seek                                ~10 ms
Send packet CA -> Netherlands -> CA     ~150 ms
```

### Powers of two

```
Power   Approx         Bytes
7       128
10      1 thousand     1 KB
20      1 million      1 MB
30      1 billion      1 GB
40      1 trillion     1 TB
50      1 quadrillion  1 PB
```

### Decision check before scaling

1. Is the bottleneck CPU, memory, I/O, or network? Measure first.
2. Is the workload CPU-bound or I/O-bound?
3. Can a single node handle the peak with faster hardware (vertical) before
   adding nodes (horizontal)?
4. Does horizontal scaling require session state to move out of the app?

## See Also

- [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) —
  horizontal scaling mechanism.
- [Caching Strategies](caching-strategies.md) — absorb uneven loads without
  scaling compute.
- [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) — risk
  hierarchy for scaling decisions.
- [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) —
  the latency target an interactive system must hit is set by human
  perception, not by a round-number SLO; back-of-the-envelope math confirms
  the backend fits inside the perception budget.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Performance vs scalability, Latency vs throughput, Appendix.
- [Latency numbers every programmer should know](https://gist.github.com/jboner/2841832)
  — historical reference; use the corrected modern figures above.
- [p99 0 ms* autocomplete for 240 million domain names](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  — Ruurtjan Pul, 2026-06-22. A latency budget derived from measured typing
  cadence (121 ms p99) rather than a round number; the backend was engineered
  to fit inside it with headroom.
