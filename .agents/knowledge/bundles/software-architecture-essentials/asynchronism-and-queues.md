---
type: Practice
title: Asynchronism and Queues
description: Decouple slow work with message queues and task queues, manage overload with back pressure, and choose delivery semantics and patterns — Kafka, exactly-once, outbox, idempotency.
tags: [architecture, asynchronism, message-queue, task-queue, back-pressure, kafka, exactly-once, outbox-pattern, idempotency]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-24"
  last-used: "2026-07-24"
---

# Asynchronism and Queues

Asynchronous workflows reduce request latency by moving expensive or
unpredictable work out of the critical path.

## Message Queues vs Task Queues

| | Message Queue | Task Queue |
|---|---------------|------------|
| Purpose | Decouple producers and consumers | Execute background jobs |
| Unit | Message / event | Task + arguments |
| Scheduling | Usually FIFO / priority | Supports scheduling, retries |
| Example | RabbitMQ, SQS, Kafka, Pulsar | Celery, BullMQ |

## Delivery Semantics

| Semantic | Meaning | How to achieve |
|----------|---------|----------------|
| **At-most-once** | 0 or 1 delivery | Fire-and-forget, no retry |
| **At-least-once** | 1+ delivery; may duplicate | Ack + retry; consumers must be idempotent |
| **Exactly-once** | 1 delivery | At-least-once + idempotency (or broker transactions) |

True exactly-once is rare; design consumers to be idempotent. Use natural
keys, dedup IDs, or versioned writes.

## Back Pressure

When queues exceed memory, performance degrades. Back pressure limits queue
size; when full, reject with 503 and have clients retry with exponential
backoff. This protects the system and the budget.

## Kafka / Log-Based Streaming

Kafka is a distributed, partitioned, replicated commit log — not a simple
queue.

- **Partitions** — parallelism unit; each consumer group splits partitions.
- **Retention** — messages kept for N days; supports replay and multiple
  consumers.
- **Use when** — you need replay, multiple downstream consumers, or
  high-throughput event streaming.
- **Don't use when** — you need strict per-message deletion or simple
  point-to-point job dispatch (use RabbitMQ/SQS instead).

## Transactional Outbox

When a service must write to a DB and publish a message, doing both atomically
across systems is impossible. The outbox pattern:

1. Write the DB change and an outbox row in one DB transaction.
2. A separate process (or CDC like Debezium) reads the outbox and publishes.
3. Mark the outbox row as sent.

Guarantees: message is published if and only if the DB write commits.

## Decision Checklist

1. Is the work too slow for the request path? Yes → queue it.
2. Can the user see partial/intermediate state? Use a status notification.
3. Can messages be duplicated? If yes, make consumers idempotent.
4. Does order matter? Use a single partition or sequence numbers.
5. Is replay needed? Yes → Kafka/log. No → traditional queue.

## See Also

- [Communication Protocols](communication-protocols.md) — transport for
  queue consumers/producers.
- [CAP, Consistency, and Availability](cap-consistency-availability.md) —
  delivery semantics are consistency choices.
- [Caching Strategies](caching-strategies.md) — back pressure and cache
  invalidation both manage overload.
- [Cost-Aware System Design](cost-aware-system-design.md) — queues and
  cap-and-shed both limit unbounded work.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Asynchronism (message queues, task queues, back pressure).
