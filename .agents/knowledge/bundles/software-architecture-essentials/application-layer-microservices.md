---
type: Practice
title: Application Layer and Microservices
description: Separate the web layer from the application layer, decompose into microservices, use service discovery, and deploy with containers, Kubernetes, serverless, and service mesh.
tags: [architecture, application-layer, microservices, service-discovery, kubernetes, serverless, service-mesh, single-responsibility]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-24"
  last-used: "2026-07-24"
---

# Application Layer and Microservices

## Layer Separation

Separate the web layer (HTTP handling, TLS, routing) from the application layer
(business logic, workers). Scale each independently. Adding a new API adds
application servers without adding web servers.

## Microservices

Independently deployable, small services that communicate over lightweight
protocols (gRPC, HTTP/REST, message queues). Each owns a bounded context.

### When to use

- Teams can own services end-to-end.
- Services need independent deployment and scaling.
- Failure domains should be isolated.

### When not to use

- Team size is small.
- Deployment automation is immature.
- Domain boundaries are unclear.

A monolith with clean modular boundaries is often better than microservices
with confused boundaries.

## Service Discovery

Services need to find each other dynamically.

| Tool | Best for |
|------|----------|
| **Consul** | Multi-cloud service registry + health checks |
| **etcd** | Kubernetes-native key-value and discovery |
| **ZooKeeper** | Coordination (locks, leader election) |

Health checks (HTTP endpoints) ensure traffic only reaches healthy instances.

## Deployment Options

| Option | Best for | Tradeoff |
|--------|----------|----------|
| **Containers** | Reproducible deployment, local parity | Adds orchestration need |
| **Kubernetes** | Container orchestration at scale | Operational complexity |
| **Serverless (FaaS)** | Spiky, low-traffic, event-driven | Cold starts, time limits, vendor lock-in |
| **Service mesh** | Uniform mTLS, retries, observability | Adds latency and control plane complexity |

## Service Mesh

A service mesh (Istio, Linkerd, Consul Connect) provides uniform
service-to-service communication: mTLS, retries, circuit breaking, traffic
splitting, and observability. Use it when you have enough services that these
concerns become a burden in application code.

## Decision Checklist

1. Is the team large enough to own multiple services? If not, monolith.
2. Do services need independent scaling or failure isolation? If yes, consider
   microservices.
3. Is container orchestration already in place? Kubernetes is the default, but
   not the only choice.
4. Are communication concerns (mTLS, retries, tracing) duplicated in every
   service? If yes, use a service mesh.

## See Also

- [Communication Protocols](communication-protocols.md) — gRPC and REST for
  inter-service calls.
- [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) — service
  mesh handles east-west LB.
- [Asynchronism and Queues](asynchronism-and-queues.md) — application workers
  enable async processing.
- [Architecture Philosophy](philosophy.md) — modular decomposition at the
  codebase level.
- [Project Structure](project-structure.md) — domain-first packages map to
  service boundaries.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Application layer (microservices, service discovery).
