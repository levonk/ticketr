---
type: Practice
title: Perceived-Latency-Driven Design
description: Set latency targets from human perception thresholds, not from an SLO picked in a vacuum. Measure the human (typing cadence, Nielsen's 0.1 s "instantaneous" bound), derive the budget the system must fit inside, then hide round-trips behind human motor latency with client-side prefetch. The budget is set by the body, not by the backend.
tags: [architecture, performance, latency, perceived-latency, prefetch, ux, nielsen, latency-budget, autocomplete, client-side-caching]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: ruurtjan-p99-0ms-autocomplete
    resource: "https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names"
    title: "p99 0 ms* autocomplete for 240 million domain names — Ruurtjan Pul (2026-06-22)"
  - id: nielsen-response-times
    resource: "https://www.nngroup.com/articles/response-times-3-important-limits/"
    title: "Nielsen Norman Group — Response Times: The 3 Important Limits (0.1 s, 1 s, 10 s)"
---

# Perceived-Latency-Driven Design

## The General Rule

The latency budget for an interactive system is set by what the human
perceives, not by what the server can deliver. Derive the budget from a
measured human action, then engineer the system to fit inside it — including
hiding round-trips the user never has to wait for. A backend that is "fast"
in absolute terms is still slow if the user perceives a gap; a backend that
is "slow" in absolute terms is instant if the response is ready before the
user looks for it.

- **Measure the human to set the budget.** The budget is not 100 ms because
  that sounds round; it is `keyPressDuration + gap + keyPressDuration` for a
  typing-driven interaction, or Nielsen's 0.1 s "instantaneous" threshold for
  a click. Measure the actual motor cadence of the target action on real
  users, take the p99 of that cadence, and that is the wall-clock budget the
  system must beat.
- **Hide round-trips behind motor latency.** The time the user spends
  pressing the next key is time the network can be working for free. Prefetch
  on the input event that starts the action (e.g. `keyDown`), render on the
  event that completes it (`keyUp`). If the response lands inside the motor
  window, the perceived latency is zero — the user never waited.
- **Stop optimizing the part that no longer dominates.** Once the backend
  fits inside the perception budget with headroom, further backend
  optimization has zero user-visible payoff. The next bottleneck is the
  network round-trip, then human motor speed itself. Optimization effort
  shifts to the new dominant part or stops. See
  [Measure Before Optimizing](measure-before-optimizing.md).

## Why It Matters

An SLO picked without reference to human perception is a number, not a
target. "p99 < 200 ms" is either far more than the user needs (wasted
engineering) or far less than the user notices (the user still perceives a
lag the SLO calls green). The only latency that matters is the one the user
perceives, and perception is gated by motor action, not by a stopwatch on
the server.

The technique inverts the usual request-response model. Instead of
request → wait → response, the system becomes prefetch → motor action →
response already waiting. The round-trip still happens; it is just hidden
inside time the user was going to spend anyway. This is the difference
between "the API is fast" and "the user never waits": the first is a backend
metric, the second is the product.

Nielsen's three limits give the coarse frame: 0.1 s is "instantaneous"
(cause and effect feel simultaneous), 1 s is "the user notices a delay but
stays in flow", 10 s is "the user loses attention". This page is about
hitting the 0.1 s band — the band where the interaction feels physical, not
digital.

## How To Apply It

1. **Identify the human action that gates perception.** For autocomplete it
   is the keystroke. For a click-to-open panel it is the click-to-render
   gap. For a drag it is the frame time during the drag. Name the motor
   event that frames the user's expectation.
2. **Measure the motor cadence at p99.** For typing, measure
   `keyUp[n] → keyUp[n+1]` across a realistic sample of fast typing and take
   the 99th percentile. That is the wall-clock budget — the response for
   keystroke `n+1` must be ready before the user releases key `n+1`. Do not
   use the mean; the tail is where the perception breaks.
3. **Split the action into a prefetch event and a render event.** Fire the
   request on the event that *starts* the action (`keyDown`), render on the
   event that *completes* it (`keyUp`). The gap between them is free
   network time. If the alphabet of likely-next inputs is bounded and small,
   prefetch all of them in one round-trip (see step 4).
4. **Bound the prefetch fan-out.** The response can carry not just the
   current results but the precomputed results for every likely next input.
   The fan-out is bounded by the input alphabet size (e.g. 38 valid domain
   characters → at most 39 result sets per request). A bounded alphabet
   makes the prefetch payload small and predictable — the cost is constant,
   not open-ended.
5. **Engineer the backend to fit the budget with headroom.** The backend
   must answer in a small fraction of the motor budget so the network
   round-trip still fits. Choose data structures whose worst case is
   effectively O(1) under bounded inputs (see
   [Data Structures First](data-structures-first.md)): an in-memory trie for
   the hot head, a memory-mapped block index for the cold tail.
6. **Measure end-to-end, not just the API.** The path is
   `Browser → CDN → origin → API` and back. The API being 2 ms is irrelevant
   if the CDN hop is 80 ms. Measure the full path at the percentiles that
   matter (p99, not p50).
7. **Stop when the backend is no longer the bottleneck.** Once the backend
   fits with headroom, the dominant cost is the network. Further backend
   tuning is wasted effort. The remaining levers are geographic distribution
   (geo load balancing, edge compute) and CDN caching of hot paths — or
   accepting that distant users exceed the budget at p99.

## Anti-Patterns

- **Picking the SLO from a round number.** "We will target 100 ms" with no
  reference to the motor action that gates perception. The number is either
  too loose (the user still perceives lag) or too tight (engineering spent
  on headroom the user cannot feel).
- **Optimizing the backend past the perception threshold.** Shaving the API
  from 2 ms to 1 ms when the network round-trip is 60 ms and the motor
  budget is 121 ms. The 1 ms is invisible; the effort was wasted. See
  [Measure Before Optimizing](measure-before-optimizing.md).
- **Request-response without prefetch.** Waiting for `keyUp` to fire the
  request, then making the user wait for the round-trip. The motor window
  was free network time and it was left on the table.
- **Unbounded prefetch fan-out.** Prefetching every possible continuation
  when the alphabet is unbounded, turning a latency optimization into a
  bandwidth and cost explosion. The technique relies on a bounded input
  alphabet; without that bound it is the wrong tool.
- **Reporting p50 and calling it instant.** The user perceives the tail, not
  the median. A p50 of 5 ms with a p99 of 300 ms is a laggy product with a
  flattering dashboard.
- **Ignoring geography.** A single-region origin hits the budget for nearby
  users and blows it for distant ones. The perception target is per-user;
  a global p99 is dominated by the farthest user.

## Concrete Instances

- **Wirewiki autocomplete (Ruurtjan Pul, 2026).** A domain-name autocomplete
  over 240 million names that achieves p99 0 ms *perceived* latency. The
  budget is derived from measured typing cadence: p99
  `keyUp[n] → keyUp[n+1]` = 121 ms. The request fires on `keyDown` and
  prefetches the result set for the typed prefix *plus* a precomputed set
  for every one of the 38 valid next characters (bounded alphabet → ≤ 312
  domains, ~2.5 kB on the wire after compression). The render fires on
  `keyUp`; if the response landed inside the motor window the user perceives
  zero latency. The backend uses an in-memory character trie for the Tranco
  top-1 M head (O(prefix length)) and an SSD-backed memory-mapped block
  index for the 240 M CZDS tail (O(prefix length × log N)), both effectively
  O(1) because both inputs are bounded. The API answers in ~2 ms at p50;
  the author stopped optimizing the API because the network round-trip
  dominates and further tuning has no user-visible payoff — the textbook
  [Measure Before Optimizing](measure-before-optimizing.md) stopping rule.
  The asterisk on "0 ms": a single European origin hits the budget for
  European users but exceeds it for USA users (+100–200 ms RTT); full p99 0
  ms globally would require geo load balancing, which the author judged
  disproportionate for a non-product.
- **Nielsen's 0.1 s "instantaneous" threshold.** The perception anchor: below
  ~100 ms, cause and effect feel simultaneous to the user. This is the band
  perceived-latency-driven design aims for; the motor-budget technique is
  how a system with a > 100 ms round-trip can still deliver a < 100 ms
  *perceived* latency by hiding the round-trip inside motor time.

## See Also

- [Measure Before Optimizing](measure-before-optimizing.md) — once the
  backend fits the perception budget with headroom, stop; the network is
  now the dominant part and further backend tuning is wasted.
- [Data Structures First](data-structures-first.md) — fitting the backend
  inside the budget is a data-structure problem: a trie for the head and a
  memory-mapped block index for the tail make the worst case effectively
  O(1) under bounded inputs.
- [Scalability Fundamentals](scalability-fundamentals.md) — the perception
  budget is the latency target; back-of-the-envelope capacity math confirms
  the backend can fit inside it. Latency vs throughput: this page is about
  the latency the user perceives, not the throughput the server delivers.
- [Caching Strategies](caching-strategies.md) — client-side prefetch is a
  cache-aside variant that populates the cache ahead of the request; CDN
  caching of hot prefix paths absorbs the prefetch fan-out at the edge.
- [CDN and DNS](cdn-and-dns.md) — edge caching of hot autocomplete paths
  absorbs frequent prefetch requests and cuts the origin round-trip for
  distant users.
- [Root-Cause First](root-cause-first.md) — "the autocomplete feels slow" is
  a perception symptom; the root cause may be a missing prefetch, a backend
  over the budget, or geographic distance — diagnose before tuning.

## Sources

- [p99 0 ms* autocomplete for 240 million domain names](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  — Ruurtjan Pul, 2026-06-22. The worked example: motor-budget derivation
  (121 ms p99 typing cadence), `keyDown` prefetch + `keyUp` render, bounded
  38-character alphabet, in-memory trie + mmap block index, and the
  decision to stop optimizing once the network dominated.
- [Response Times: The 3 Important Limits](https://www.nngroup.com/articles/response-times-3-important-limits/)
  — Nielsen Norman Group. The 0.1 s / 1 s / 10 s perception thresholds that
  frame the budget.
