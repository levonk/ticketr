---
type: Practice
title: Caching Strategies
description: Multi-layer caching (client, CDN, web server, database, application), cache update strategies, and cache invalidation decisions.
tags: [architecture, caching, cache-aside, write-through, write-behind, refresh-ahead, cache-invalidation, redis, memcached]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
---

# Caching Strategies

## Cache Layers

| Layer | Example | When to use |
|-------|---------|-------------|
| **Client** | Browser cache, `Cache-Control` | Static assets, idempotent GETs |
| **CDN** | CloudFront, Cloudflare | Global static content distribution |
| **Web server / reverse proxy** | Varnish, NGINX cache | Dynamic but cacheable pages |
| **Database** | Buffer pool, shared buffers | Managed by DB; tune for workload |
| **Application** | Redis, Memcached | Hot objects, sessions, query results |

## What to Cache

- User sessions.
- Fully rendered pages (with Vary header care).
- Activity streams and user graph data.
- Database objects (not raw queries — easier to invalidate).

Object-level caching is preferred over query-level caching: invalidate the
object when its data changes, rather than trying to find every query that might
include a changed cell.

## Cache Update Strategies

| Strategy | Write pattern | Read pattern | Best for |
|----------|---------------|--------------|----------|
| **Cache-aside** (lazy) | App writes to DB | App loads into cache on miss | Read-heavy, cache not always needed |
| **Write-through** | App writes to cache; cache writes to DB synchronously | Read from cache | Read-after-write consistency needed |
| **Write-behind** | App writes to cache; cache writes to DB asynchronously | Read from cache | High write throughput, can risk brief data loss |
| **Refresh-ahead** | Cache pre-fetches hot items before expiry | Read from cache | Predictable hot access patterns |

## Cache Invalidation

The hard problem. Strategies:

- **TTL** — simplest; tolerate staleness for the TTL window.
- **Event-driven invalidation** — write triggers cache delete.
- **Write-through** — cache is always consistent by definition.
- **Versioned keys** — append hash/version to key; old versions expire via TTL.

## Decision Checklist

1. Is the data read more than written? If not, a cache may not help.
2. Can stale data be served for the TTL? If not, use write-through or
   event-driven invalidation.
3. Is a cache miss expensive? If yes, use refresh-ahead or warm caches on
   deploy.
4. Is cache warm-up a problem? New nodes start empty — plan for cold-start.

## See Also

- [Database Scaling](database-scaling.md) — caches sit in front of databases.
- [CDN and DNS](cdn-and-dns.md) — CDN is the outermost cache.
- [Data Access Layer](data-access-layer.md) — centralize cache logic.
- [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) —
  client-side prefetch is a cache-aside variant that populates the cache
  ahead of the request, hiding the round-trip behind human motor latency;
  refresh-ahead is the server-side analog of the same idea.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Cache (layers, update strategies, invalidation).
- [p99 0 ms* autocomplete for 240 million domain names](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  — Ruurtjan Pul, 2026-06-22. Client-side prefetch on `keyDown` populates a
  per-next-character cache before the `keyUp` render; the bounded 38-character
  alphabet caps the prefetch payload at ~2.5 kB on the wire.
