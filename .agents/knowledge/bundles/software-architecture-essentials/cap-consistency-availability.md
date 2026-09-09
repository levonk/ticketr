---
type: Practice
title: CAP, Consistency, and Availability
description: CAP theorem tradeoffs, consistency patterns, availability patterns, availability-in-numbers math, and modern extensions — PACELC, SLOs, chaos engineering.
tags: [architecture, cap-theorem, consistency, availability, replication, failover, slo, chaos-engineering, crdt, commutative-monoid]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-25"
  last-used: "2026-07-25"
---

# CAP, Consistency, and Availability

## CAP Theorem

In a distributed system, you can only guarantee two of:

- **Consistency** — every read receives the most recent write or an error.
- **Availability** — every request receives a response (may be stale).
- **Partition Tolerance** — the system continues to operate despite network
  partitions.

Networks are unreliable, so partition tolerance is effectively mandatory. The
real design decision is **CP** (consistent, may fail during a partition) vs
**AP** (always available, may serve stale data).

## PACELC

CAP only covers the partition case. **PACELC** extends it: **If there is a
Partition, choose Availability or Consistency; Else (no partition), choose
Latency or Consistency.**

| System | P? | E? | Typical choice |
|--------|----|----|----------------|
| Spanner / CockroachDB | CP | EC | Strongly consistent even without partition |
| DynamoDB / Cassandra | AP | EL | Available, low latency |

## Consistency Patterns

| Pattern | Read guarantee | Replication | Best for |
|---------|---------------|-------------|----------|
| **Weak** | May not see a write | Best-effort | VoIP, real-time games |
| **Eventual** | Will see it eventually (ms) | Asynchronous | DNS, email, feeds |
| **Strong** | Will see it immediately | Synchronous | Transactions, ledgers |

## Availability Patterns

### Fail-over

- **Active-passive** (primary-standby): passive takes over on heartbeat loss.
  Downtime depends on hot vs cold standby.
- **Active-active** (multi-primary): all nodes handle traffic; requires
  conflict resolution.
- **Multi-region active-active**: deploy across 3+ regions. Requires CRDTs,
  last-writer-wins, or application-level conflict resolution.

### CRDTs and Commutative Monoids

**Conflict-free Replicated Data Types (CRDTs)** are the data structures that
make AP/eventual consistency mathematically sound. A CRDT's state forms a
**commutative monoid** — updates can be applied in any order and still produce
the same final state, which is exactly what lets replicas converge without
coordination.

This is the same property that powers
[Decentralized P2P Architecture](decentralized-p2p-architecture.md): P2P
contract state must form a commutative monoid so peers can exchange
**summaries** (compact "what I have" hashes) and **deltas** (minimal updates)
and converge regardless of message order. The CRDT toolset is identical whether
the replicas are multi-region cloud regions or P2P subscribers on laptops.

Common CRDT families: G-Counter / PN-Counter (counters), LWW-Register /
OR-Set (registers and sets), RGA / Yjs / Automerge (collaborative text). See
[crdt.tech](https://crdt.tech/) for the catalog.

### Availability in numbers

| Availability | Per year | Per month | Per week |
|--------------|----------|-----------|----------|
| 99.9% (three 9s) | 8h 46m | 43m 50s | 10m 5s |
| 99.99% (four 9s) | 52m 36s | 4m 23s | 1m 5s |
| 99.999% (five 9s) | 5m 15s | 26s | 6s |

### Serial vs parallel availability

- **In sequence**: `A_total = A_foo × A_bar` → availability drops.
- **In parallel**: `A_total = 1 − (1 − A_foo) × (1 − A_bar)` → availability rises.

A chain of 99.9% components degrades quickly; redundant paths multiply
availability.

## Modern Reliability Practices

- **SLO-based design** — define error budgets. A 99.9% SLO gives 43m
  downtime/month; burn-rate alerts warn before the budget is gone.
- **Chaos engineering** — inject failures (kill nodes, partition network, add
  latency) to prove fail-over actually works.
- **Graceful degradation** — return partial results or cached responses when
  dependencies fail, with circuit breakers.

## Decision Checklist

1. Is the workload read-heavy or write-heavy? Read-heavy favors replicas;
  write-heavy favors partitioning or multi-primary.
2. Can the business tolerate stale reads? If yes, AP/eventual. If no, CP/strong.
3. Is cross-region availability required? If yes, plan conflict resolution.
4. Is the SLA defined in nines? Convert to error budget and alert on burn rate.

## See Also

- [Database Scaling](database-scaling.md) — replication and sharding
  implementations.
- [Caching Strategies](caching-strategies.md) — caches are AP; invalidation is
  consistency.
- [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) — fail-over
  at the LB layer.
- [Decentralized P2P Architecture](decentralized-p2p-architecture.md) — CRDTs
  and commutative monoids are the same consistency tool used for P2P
  subscriber-based replication.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Availability vs consistency, Consistency patterns, Availability patterns.
- [PACELC theorem](https://en.wikipedia.org/wiki/PACELC_theorem)
- [CAP FAQ](https://github.com/henryr/cap-faq)
