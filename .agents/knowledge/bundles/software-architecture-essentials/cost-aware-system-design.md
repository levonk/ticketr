---
type: Practice
title: Cost-Aware System Design
description: Treat cost as a first-class architectural constraint — per-seat vs per-call billing, fan-out cost amplification, cap-and-shed patterns, and flat-rate vs metered contracts.
tags: [architecture, cost, per-seat, per-call, fan-out, cap-and-shed, flat-rate, cost-allocation]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-24"
  last-used: "2026-07-24"
---

# Cost-Aware System Design

## Pricing Models

| Model | Pay for | Scales with | Example |
|-------|---------|-------------|---------|
| **Per-seat** | User | User count | Slack, GitHub |
| **Per-call** | Request | Usage volume | S3, Lambda, OpenAI |
| **Tiered** | Banded usage | Volume brackets | Stripe, SendGrid |
| **Flat-rate** | Fixed | Nothing | Enterprise contracts, unlimited plans |

## Per-Seat vs Per-Call Break-Even

```
break_even_calls_per_user = per_seat_price / per_call_price
```

Example: $20/month per-seat vs $0.001 per-call → 20,000 calls/user/month.
Below that, per-call wins; above, per-seat wins.

A system with high fan-out (one user action triggers many downstream calls)
can make per-call costs explode. Per-seat absorbs fan-out at no marginal cost.

## Fan-Out Cost Mitigation

- **Batching** — one batch call instead of N single calls.
- **Caching** — cache downstream results (see [Caching Strategies](caching-strategies.md)).
- **Aggregation service** — fan out once, serve many from cache.
- **Contract renegotiation** — move high-fan-out dependencies to per-seat or
  flat-rate when the math favors it.

## Cap-and-Shed

Set a hard budget for metered APIs. When the cap is hit, shed load — return
429/503, queue for later, or degrade. This is the cost equivalent of back
pressure.

```python
if calls_this_minute >= MAX_CALLS_PER_MINUTE:
    return Response(status=429, headers={"Retry-After": "60"})
```

## Flat-Rate vs Metered

| | Flat-rate | Metered |
|---|-----------|---------|
| Predictability | High | Low |
| Low usage | Expensive | Cheap |
| High usage | Cheap | Expensive |
| Best for | Steady, predictable, high usage | Variable, spiky, new integrations |

## Cost-Allocation Tagging

Tag every resource with owner, project, environment, and feature. Cost
 dashboards aggregate by tag. Enforce tags in IaC and block untagged
deployments. Without tags, you cannot optimize cost.

## Decision Checklist

1. Is usage per-user predictable? Per-seat may be cheaper.
2. Does one user action trigger many downstream calls? Beware per-call
   amplification.
3. Can you set a spending cap? If not, metered APIs are a financial risk.
4. Is this a steady high-volume dependency? Negotiate flat-rate.
5. Can you attribute cost to feature/team? Tag everything.

## See Also

- [Scalability Fundamentals](scalability-fundamentals.md) — capacity
  estimation feeds cost estimation.
- [Caching Strategies](caching-strategies.md) — caching reduces per-call costs.
- [Asynchronism and Queues](asynchronism-and-queues.md) — back pressure and
  cap-and-shed are load-management siblings.
- [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) — cost is
  one axis in the risk hierarchy.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — no cost-aware section; this page fills the gap.
