---
okf_version: "0.2"
---

# Software Architecture Essentials

A compounding knowledge base documenting architectural practices for modular,
maintainable software — from codebase-level structure (project layout, data
access, configuration, theming) to system-level distributed architecture
(scalability, CAP, load balancing, databases, caching, queues, protocols,
security, cost, microservices, and decentralized P2P). Each concept captures a
specific architectural concern with the practice that addresses it.

## Concepts

* [Overview](overview.md) - Synthesis of the architecture practice set and how the pieces fit together
* [Root-Cause First](root-cause-first.md) - Diagnose before fixing; workarounds are last resort; when changing a standard, update every document that restates the old one
* [Measure Before Optimizing](measure-before-optimizing.md) - Do not tune for speed until a measurement proves where the bottleneck is, and even then only when one part overwhelms the rest (Hoare/Pike rules 1-2)
* [Data Structures First](data-structures-first.md) - Choose the right data structures and the algorithms become self-evident; data dominates; write stupid code that uses smart objects (Brooks/Pike rule 5)
* [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) - Set latency targets from human perception thresholds (typing cadence, Nielsen's 0.1 s), not round-number SLOs; hide round-trips behind motor latency with client-side prefetch; stop optimizing the backend once the network dominates
* [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) - Ordered risk hierarchy for evaluating technology decisions (novel work > end-user impact > ... > new constant); dependency-update and functional-style axes
* [Technology Selection Pattern](technology-selection-pattern.md) - Convention for authoring technology-selection concept pages — state the requirement, list candidates, apply the risk hierarchy, recommend one, document why-not, state when to reconsider
* [AI + Human Timeline Estimates](ai-human-timeline-estimates.md) - Estimate as human review + AI execution pairs on four axes; pre-AI "human days" are no longer valid units
* [Architecture Philosophy](philosophy.md) - Domain-based modular architecture with clear separation of concerns and thin top-level entry points
* [Project Structure](project-structure.md) - Domain-first hierarchical package structure with vertical slicing for scalable monorepos
* [Data Access Layer](data-access-layer.md) - Centralize data fetching, caching, and mutation; single source of truth; security checkpoint
* [Configuration System](configuration-system.md) - Layered config precedence with schema validation, caching, and documented override rules
* [Distribution and Packaging](distribution.md) - Prefer single-binary or minimal-runtime distributions; track sizes; document install paths
* [Theme System](theme-system.md) - Single source of truth for palettes/variants; runtime switching; consistent semantic colors
* [Terminal State Management](terminal-state.md) - Minimal control sequences; prepare state for tools; centralize theme-aware helpers
* [Tool Detection Architecture](tool-detection.md) - PATH-first detection with version verification, caching, and clear errors
* [Adding New Tools](adding-tools.md) - Consistent procedure for wiring new tools into the CLI surface and service layer
* [Indexed AST Tool Selection](indexed-ast-tools.md) - Pick the right indexed AST tool (CodeGraph, Graphify, GitNexus) for the AST Search and AST Insights rows of the 6×2 matrix — by freshness, dispatch tracing, multi-repo support, content breadth, and license; they win orthogonal rounds and can run together
* [Authentication and Environment Management](auth-env.md) - Detect CI/SSH/Docker/Codespaces; prevent browser auth in headless; centralize helpers
* [Scalability Fundamentals](scalability-fundamentals.md) - Performance vs scalability, latency vs throughput, back-of-the-envelope calculations with corrected post-2024 latency figures
* [CAP, Consistency, and Availability](cap-consistency-availability.md) - CAP theorem, PACELC, consistency patterns (weak/eventual/strong), availability patterns (fail-over, replication, multi-region), availability-in-numbers, SLOs, chaos engineering
* [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) - L4/L7 load balancing, horizontal scaling, reverse proxy, service mesh, cloud-managed LBs, HTTP/3 QUIC, edge compute
* [CDN and DNS](cdn-and-dns.md) - Push vs pull CDNs, edge caching, DNS record types, TTL caching, DNS-based load balancing and fail-over
* [Database Scaling](database-scaling.md) - RDBMS replication/federation/sharding/denormalization/SQL tuning, NoSQL models (KV/document/wide-column/graph), distributed SQL, vector DBs, lakehouse, time-series
* [Caching Strategies](caching-strategies.md) - Multi-layer caching, cache-aside/write-through/write-behind/refresh-ahead, cache invalidation as the hard problem
* [Asynchronism and Queues](asynchronism-and-queues.md) - Message/task queues, back pressure, Kafka partitions/retention, delivery semantics, transactional outbox, idempotency
* [Communication Protocols](communication-protocols.md) - HTTP, TCP/UDP, RPC (gRPC), REST, GraphQL, WebSockets, SSE, WebTransport, WebRTC, HTTP/3 (QUIC) with tradeoffs, layering, browser support, NAT traversal, and a P2P branch
* [System Security Basics](system-security-basics.md) - Encrypt/sanitize/least-privilege foundations plus zero-trust, mTLS, OAuth2/OIDC, secrets management, observability (OpenTelemetry)
* [Application Layer and Microservices](application-layer-microservices.md) - Web/app layer separation, microservices, service discovery, containers, Kubernetes, serverless, service mesh
* [Resilience Patterns](resilience-patterns.md) - The canonical invariant (never allow unbounded concurrency or fan-out); the amplification cascade (April 2026 outage: unbounded goroutines to port exhaustion to GC collapse to OOM to restart loop); four classes of overload protection (concurrency limiters, connection poolers, circuit breakers, adaptive concurrency); Hystrix is necessary but not sufficient; tool-class feature matrix; choke-point principle; no universal library; service mesh + per-language library stack with cross-language matrix
* [Decentralized P2P Architecture](decentralized-p2p-architecture.md) - Peer-to-peer topology, subscriber-based replication, CRDT/commutative-monoid state, WASM-sandboxed contracts, censorship-resistance, browser-as-node limitations, homelab always-on-node deployment
* [Cost-Aware System Design](cost-aware-system-design.md) - Per-seat vs per-call billing, fan-out cost amplification, cap-and-shed, flat-rate vs metered contracts, cost-allocation tagging
* [Business Models Around Open Protocols](business-models-around-open-protocols.md) - Managed nodes, premium clients, curation, identity services, B2B censorship-resistant infra, consulting, seedboxing; why tolls and arbitrage fail on open networks
* [Plugin Scanner Registration](plugin-scanner-registration.md) - Trait-based plugin registration, dynamic loading vs compile-time, discovery, lifecycle, dependency injection
* [Rule Engine Design](rule-engine-design.md) - Rule evaluation order, activation profiles, conflict resolution, performance optimization, incremental evaluation
* [Trait-Based Extensibility](trait-based-extensibility.md) - Trait hierarchies for rules, default impls, trait object vs generic, combinators, trait versioning
* [Config-Driven Tool Design](config-driven-tool-design.md) - Cross-bundle synthesis: layered config precedence, profile-based activation, schema migration, hot-reload, cross-platform discovery
* [Event-Driven Linter Architecture](event-driven-linter-architecture.md) - Cross-bundle synthesis: IDE hook integration, event filtering/routing, async processing, backpressure
* [Multi-Language Scanner Coordination](multi-language-scanner-coordination.md) - Cross-bundle synthesis: language detection, scanner registration, shared interfaces, polyglot project analysis
