---
type: Practice
title: Load Balancing and Reverse Proxy
description: Distribute traffic with L4/L7 load balancers, horizontal scaling, reverse proxies for SSL termination and caching, plus service mesh, cloud-managed LBs, HTTP/3 QUIC, and edge compute.
tags: [architecture, load-balancing, reverse-proxy, horizontal-scaling, service-mesh, ssl-termination, cdn, edge-compute]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-24"
  last-used: "2026-07-24"
---

# Load Balancing and Reverse Proxy

## When to Use a Load Balancer

Use a load balancer when you have multiple backend instances and need:

- Health-checking (no traffic to failed nodes).
- Traffic distribution.
- SSL termination offloaded from backends.
- Session persistence.
- A single point of failure replaced by multiple LBs.

## Layer 4 vs Layer 7

| | Layer 4 | Layer 7 |
|---|---------|---------|
| Inspects | IP, port | Headers, cookies, message body |
| Forwarding | NAT / packet | Terminates connection, reads request |
| Flexibility | Low | High (route video to video servers, billing to hardened servers) |
| Latency | Lower | Slightly higher (minimal on modern hardware) |

**Decision**: Use L4 for high-throughput, simple TCP/UDP distribution. Use L7
when routing decisions need HTTP knowledge (host, path, headers, cookies).

## Routing Strategies

- **Round-robin** — simple, fair; bad if nodes have different capacity.
- **Weighted round-robin** — account for instance size.
- **Least connections / least loaded** — better for long-lived connections.
- **IP hash / session persistence** — same client to same backend.
- **Random with bounded loads** — simple and effective for homogeneous nodes.

## Reverse Proxy

A reverse proxy centralizes internal services behind a public interface.
Benefits even with one backend:

- SSL termination.
- Compression.
- Caching.
- Static content serving.
- Security (hide backend IPs, rate-limit, blacklist).

### Load balancer vs reverse proxy

Every load balancer is a reverse proxy. Not every reverse proxy load-balances.
Deploy a reverse proxy for SSL/security/caching; add load balancing when you
scale beyond one backend.

## Modern Options

- **Cloud-managed LBs** (ALB, NLB, GCP Load Balancer, Azure Front Door) —
  default for cloud deployments: auto-scaling, health checks, WAF integration.
- **Service mesh** (Istio, Linkerd, Consul Connect) — east-west
  service-to-service load balancing, mTLS, retries, circuit breaking.
- **HTTP/3 (QUIC)** — UDP-based, multiplexed, 0-RTT. Load balancers must route
  by QUIC connection ID, not 4-tuple.
- **Edge compute** (Cloudflare Workers, Lambda@Edge) — run logic at the edge,
  reducing origin load.

## Decision Checklist

1. Are you routing client-to-server (north-south) or service-to-service
   (east-west)? North-south: LB/gateway. East-west: service mesh.
2. Do routing decisions need HTTP knowledge? Yes → L7. No → L4.
3. Do sessions live on the server? Yes → session persistence or externalize
   sessions to a shared cache/DB.
4. Is the protocol QUIC? Yes → ensure LB supports connection-ID routing.

## See Also

- [CDN and DNS](cdn-and-dns.md) — edge caching and name resolution.
- [Caching Strategies](caching-strategies.md) — reverse proxies can cache.
- [System Security Basics](system-security-basics.md) — SSL termination and
  mTLS.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Load balancer, Reverse proxy.
