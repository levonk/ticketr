# Directory Update Log

## 2026-08-30

* **Ingest**: Added
  [Ruurtjan Pul — "p99 0 ms* autocomplete for 240 million domain names"](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  (2026-06-22) and the
  [Nielsen Norman Group response-time thresholds](https://www.nngroup.com/articles/response-times-3-important-limits/)
  to the discipline axis of the bundle. The source filled one gap and
  supplied concrete instances for five existing pages.
  - [perceived-latency-driven-design.md](perceived-latency-driven-design.md)
    — new concept: set latency targets from human perception thresholds
    (measured typing cadence, Nielsen's 0.1 s "instantaneous" bound), not
    from round-number SLOs; hide round-trips behind human motor latency
    with client-side prefetch (`keyDown` prefetch + `keyUp` render); bound
    the prefetch fan-out by the input alphabet (38 domain characters →
    ≤ 312 results, ~2.5 kB on the wire); stop optimizing the backend once
    the network dominates. Distinct from scalability-fundamentals (capacity
    math), measure-before-optimizing (profiling discipline), and
    caching-strategies (cache patterns) — this page is about *where the
    latency target comes from* and *how to hide round-trips inside motor
    time*.
* **Update**: [measure-before-optimizing.md](measure-before-optimizing.md)
  — added a Concrete Instance (the Wirewiki stopping rule: API at 2 ms,
  network at 60 ms, so further API tuning stopped), a See Also link to
  perceived-latency-driven-design, and the source. Updated knowledge-basis
  and last-used to 2026-08-30.
* **Update**: [data-structures-first.md](data-structures-first.md) — added a
  Concrete Instance (in-memory trie for the Tranco head + mmap block index
  for the 240 M CZDS tail, both effectively O(1) under bounded inputs), a
  See Also link to perceived-latency-driven-design, and the source. Updated
  knowledge-basis and last-used to 2026-08-30.
* **Update**: [scalability-fundamentals.md](scalability-fundamentals.md) —
  added a See Also link to perceived-latency-driven-design (the latency
  target is set by perception, not a round number; back-of-the-envelope
  math confirms the backend fits the budget) and the source. Updated
  knowledge-basis and last-used to 2026-08-30.
* **Update**: [caching-strategies.md](caching-strategies.md) — added a See
  Also link to perceived-latency-driven-design (client-side prefetch as a
  cache-aside variant; refresh-ahead is the server-side analog) and the
  source. Updated knowledge-basis and last-used to 2026-08-30.
* **Update**: [cdn-and-dns.md](cdn-and-dns.md) — added a See Also link to
  perceived-latency-driven-design (edge caching of hot prefix paths absorbs
  the prefetch fan-out; single-origin geography blows the budget for
  distant users) and the source. Updated knowledge-basis and last-used to
  2026-08-30.
* **Update**: [overview.md](overview.md) — extended the discipline axis of
  the architecture landscape with `perceived-latency-driven-design` between
  data-structures-first and tech-decision-risk; added a Latency Target row
  to the concern table; added the Ruurtjan and Nielsen sources. Updated
  knowledge-basis and last-used to 2026-08-30.
* **Update**: [index.md](index.md) — registered the new concept page on the
  discipline axis after data-structures-first.

## 2026-08-27

* **Ingest**: Added Rob Pike's "Notes on Programming in C" (the 5 Rules of
  Programming) and Fred Brooks' *The Mythical Man-Month* to the discipline
  axis of the bundle. Rules 3-4 (fancy algorithms are slow when n is small
  and buggier than simple ones; Ken Thompson's "when in doubt, use brute
  force") were already covered by
  [kiss-principle.md](kiss-principle.md) and are now cited there as a
  Concrete Instance with a See Also link to the new measure page. The two
  gaps the source filled:
  - [measure-before-optimizing.md](measure-before-optimizing.md) — Pike
    rules 1-2 / Hoare's "premature optimization is the root of all evil":
    profiler-first discipline, do not tune until measurement proves the
    bottleneck and one part overwhelms the rest. Distinct from
    scalability-fundamentals.md (back-of-envelope capacity math is the
    estimate step; this is the measure step that confirms it) and from
    kiss-principle.md (which only had a brief KISS-vs-performance note).
  - [data-structures-first.md](data-structures-first.md) — Pike rule 5 /
    Brooks' "show me your tables": data structures, not algorithms, are
    central; write stupid code that uses smart objects; make illegal states
    unrepresentable. Not previously captured anywhere in the bundle.
* **Update**: [kiss-principle.md](kiss-principle.md) — added a Concrete
  Instance citing Pike rules 3-4 and Thompson's "brute force" rephrasing,
  plus See Also links to measure-before-optimizing and data-structures-first.
* **Update**: [overview.md](overview.md) — extended the discipline axis of
  the architecture landscape with `measure-before-optimizing` and
  `data-structures-first` between root-cause-first and tech-decision-risk;
  added Profiling and Data Model rows to the concern table; added Pike and
  Brooks to Sources. Updated knowledge-basis and last-used to 2026-08-27.
* **Update**: [index.md](index.md) — registered both new concept pages on
  the discipline axis after root-cause-first.

## 2026-08-09

* **Update**: Added "Setup via dev-env-upsert" section to [indexed-ast-tools.md](indexed-ast-tools.md) — documents the dev-env-upsert delegation pattern for indexed AST tool setup (file-type-aware detection, folds into prime_impl, staleness check inside prime_impl, not blanket install). Added See Also links to async-prime-internal.md and index-staleness-check.md.

## 2026-08-05

* **Ingest**: Authored 6 new concept pages sourced from the project-lint
  audit (linter architecture gaps). Includes 3 cross-bundle synthesis pages
  (config-driven-tool-design, event-driven-linter-architecture,
  multi-language-scanner-coordination). All pages grounded against
  project-lint/Cargo.toml versions on 2026-08-05.
  - [plugin-scanner-registration.md](plugin-scanner-registration.md) —
    trait-based plugin registration, dynamic loading vs compile-time,
    discovery, lifecycle
  - [rule-engine-design.md](rule-engine-design.md) — rule evaluation order,
    activation profiles, conflict resolution, incremental evaluation
  - [trait-based-extensibility.md](trait-based-extensibility.md) — trait
    hierarchies, default impls, trait object vs generic, combinators
  - [config-driven-tool-design.md](config-driven-tool-design.md) —
    cross-bundle synthesis: layered config precedence, profile activation,
    schema migration, hot-reload
  - [event-driven-linter-architecture.md](event-driven-linter-architecture.md)
    — cross-bundle synthesis: IDE hook integration, event routing, async
    processing, backpressure
  - [multi-language-scanner-coordination.md](multi-language-scanner-coordination.md)
    — cross-bundle synthesis: language detection, scanner registration,
    polyglot project analysis
* **Update**: Updated [index.md](index.md) with 6 new concept entries.

## 2026-08-02

* **Ingest**: Significantly expanded
  [resilience-patterns.md](resilience-patterns.md) with material from the
  April 2026 outage post-mortem on unbounded goroutine fan-out. Added six new
  sections that reframe the page around the root-cause invariant:
  1. **The Canonical Invariant** — leads the page now: "Never allow unbounded
     concurrency or unbounded fan-out; every parallelizable operation must
     have an explicit, enforced limit." Includes the generalized form ("No
     component may create work faster than the slowest downstream dependency
     can safely absorb it") and the operational law ("Every system must be
     designed so that a single pathological request cannot multiply itself
     into a cluster-wide resource collapse").
  2. **The Amplification Cascade** — the April 2026 outage walkthrough:
     20k goroutines to 20k memcached dials to ephemeral port exhaustion to
     TIME_WAIT to memcached failures to millions of error logs to blocking
     write(2) to 10× OS threads to GC pressure to OOM to restart to
     instant re-failure. Includes a stage-by-stage table and notes the
     cascade is self-reinforcing (restart does not clear TIME_WAIT).
  3. **The Meta-Rule** — connects the invariant to N+1 queries, unbounded
     recursion, unbounded retries, unbounded queues, unbounded goroutines,
     unbounded log emission, and unbounded memory growth as one pattern.
  4. **Hystrix: Necessary but Not Sufficient** — Hystrix is a downstream
     isolation tool; it caps concurrency at the RPC boundary, not inside
     your handler. The outage was caused by internal unbounded concurrency,
     not downstream slowness. Hystrix would have caught the symptoms
     (memcached latency) but not the cause (20k goroutines). Notes that
     Netflix archived Hystrix and recommends Resilience4j or NCL for new
     work.
  5. **Tool-Class Feature Matrix** — Hystrix vs Envoy Circuit Breakers vs
     Netflix Concurrency Limits vs Go errgroup/SetLimit vs Go
     net/http Transport vs Service Mesh adaptive concurrency vs Retry
     Budgeting, scored on prevents-unbounded-fan-out,
     prevents-unbounded-connections, prevents-downstream-overload, and
     adaptive. Only ICL directly prevents the root cause.
  6. **Request Lifecycle Diagram** — Mermaid flowchart showing where each
     tool class applies (Client → Handler → Internal Fan Out → Connection
     Pool → RPC Calls → External Services) and what each limits via
     dashed edges. Annotates that the April 2026 cascade started at the
     Internal Fan Out node because no ICL was attached.
  Also added two new anti-patterns (unbounded fan-out as the root cause;
  unbounded log emission on the failure path), a new decision-checklist item
  (bound log throughput), a new See Also link to root-cause-first.md, and
  two new sources (Netflix Concurrency Limits, the April 2026 post-mortem).
  Updated the index.md and overview.md.tmpl descriptions to reflect the
  canonical invariant as the lead.

## 2026-07-29

* **Addition**: Authored [resilience-patterns.md](resilience-patterns.md)
  — four classes of overload protection (internal concurrency limiters,
  connection poolers, RPC circuit breakers, adaptive concurrency), the
  choke-point principle (all four require a central registry + metrics +
  choke point), the no-universal-library insight (concurrency models,
  networking stacks, and failure semantics differ across languages), and the
  stack you actually need: service mesh (Envoy/Istio/Linkerd) for
  cross-language control + per-language libraries for internal fan-out.
  Includes a per-language library matrix (Go, Java, Rust, Node, Python, .NET)
  and anti-patterns (no choke point, static-only limits, retry-storms,
  mesh-for-internal-fan-out). Cross-linked to load-balancing,
  microservices, asynchronism, CAP, cost-aware, security, and tech-decision-
  risk. Sourced from the System Design Primer, Envoy adaptive concurrency
  docs, Resilience4j, and Netflix Hystrix.
* **Addition**: Authored [technology-selection-pattern.md](technology-selection-pattern.md)
  — convention for authoring technology-selection concept pages. Required
  sections: Failure Mode, Requirement, Candidates, Risk Assessment (applies
  the tech-decision-risk hierarchy), Recommendation, Why Not the Others,
  When to Reconsider, See Also. Optional sections: Preference Ordering,
  Verification, Per-language matrix. Documents when to write a selection
  page (reusable, non-obvious winner, reversibility cost, risk hierarchy
  level 6+) and when not to. References two worked examples:
  auth-provider-selection (binary choice, risk level 2) and
  resilience-patterns (stack choice with per-language matrix, risk level 6
  vs 11-12). Cross-linked to tech-decision-risk, ai-human-timelines,
  root-cause-first, and both worked examples.
* **Update**: [overview.md](overview.md) — extended the architecture
  landscape diagram with `technology-selection-pattern` on the decisions axis
  and `resilience-patterns` on the system-level axis (between microservices
  and cost); added Selection and Resilience rows to the concern table.
  Updated knowledge-basis and last-used to 2026-07-29.
* **Update**: [index.md](index.md) — registered both new concept pages.

## 2026-07-26
* **Migration**: Migrated `## Citations` body sections to `sources` frontmatter with stable `id` attributes per OKF v0.2 §13.1.
* **Migration**: Migrated bundle from OKF v0.1 to OKF v0.2 — bumped `okf_version` in index.md. No `# Citations` sections or `timestamp` fields to migrate.

## 2026-07-25

* **Ingest**: Added the decentralized / P2P architecture axis and the
  business-model axis to the bundle, sourced from the
  [Freenet tutorial](https://freenet.org/build/manual/tutorial/) and the
  Freenet research captured in the 2ndbrain vault
  (`Computer/Medium/Freenet/Freenet.md` and
  `Computer/Medium/Web Communication Protocols Compared.md`). The bundle was
  previously client-server / microservices only; this ingest adds the
  structural opposite (P2P) and the revenue side of the cost-aware equation.
  New pages:
  - [Decentralized P2P Architecture](decentralized-p2p-architecture.md) — P2P
    topology, subscriber-based replication, CRDT/commutative-monoid state,
    WASM-sandboxed contracts, censorship-resistance threat model, browser-as-
    node limitations (sockets / always-on), homelab always-on-node deployment.
  - [Business Models Around Open Protocols](business-models-around-open-protocols.md)
    — managed nodes, premium clients, curation, identity services, B2B
    censorship-resistant infra, consulting, seedboxing; why per-transaction
    tolls, middleman arbitrage, and in-contract ads fail on open networks.
* **Update**: [communication-protocols.md](communication-protocols.md) — added
  WebTransport, WebRTC, and SSE; added the application-vs-transport layering
  note; added a browser support matrix and a NAT traversal section; added a P2P
  branch and a mobile/unstable-network branch to the decision checklist;
  cross-linked to the new Decentralized P2P Architecture page. Updated
  knowledge-basis and last-used to 2026-07-25.
* **Update**: [cap-consistency-availability.md](cap-consistency-availability.md)
  — promoted the one-line CRDT mention into a "CRDTs and Commutative Monoids"
  subsection covering G-Counter / PN-Counter / LWW-Register / OR-Set / RGA /
  Yjs / Automerge, and the equivalence between multi-region active-active
  replication and P2P subscriber-based replication. Added `crdt` and
  `commutative-monoid` tags. Cross-linked to Decentralized P2P Architecture.
  Updated knowledge-basis and last-used to 2026-07-25.
* **Update**: [overview.md](overview.md) — extended the architecture landscape
  diagram with a new `decentralized-p2p → business-models-around-open-protocols`
  axis; added Decentralization and Revenue rows to the concern table; updated
  the bundle description and scope to cover the P2P axis. Updated knowledge-basis
  and last-used to 2026-07-25.
* **Update**: [index.md](index.md) — registered both new concept pages and
  updated the Communication Protocols entry to reflect the added protocols and
  P2P branch. Updated bundle description.

## 2026-07-24

* **Ingest**: Added system-level architecture concepts from
  [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  to extend the bundle from codebase-level architecture to system-level
  architecture. Each new page includes decision checklists and tradeoffs.
  New pages:
  - [Scalability Fundamentals](scalability-fundamentals.md) — performance vs
    scalability, latency vs throughput, back-of-the-envelope with current
    hardware latency figures.
  - [CAP, Consistency, and Availability](cap-consistency-availability.md) —
    CAP theorem, PACELC, consistency patterns (weak/eventual/strong), fail-over,
    availability in numbers, multi-region active-active, SLOs, chaos engineering.
  - [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) — L4/L7,
    horizontal scaling, reverse proxy, service mesh, cloud-managed LBs, HTTP/3
    QUIC, edge compute.
  - [CDN and DNS](cdn-and-dns.md) — push vs pull CDNs, DNS record types, TTL,
    geo/health-based DNS routing.
  - [Database Scaling](database-scaling.md) — RDBMS replication/federation/
    sharding/denormalization/SQL tuning, NoSQL data models, distributed SQL,
    vector DBs, lakehouse, time-series.
  - [Caching Strategies](caching-strategies.md) — multi-layer caching,
    cache-aside/write-through/write-behind/refresh-ahead, invalidation.
  - [Asynchronism and Queues](asynchronism-and-queues.md) — message queues,
    task queues, back pressure, Kafka/partitions/retention, delivery semantics,
    transactional outbox, idempotency.
  - [Communication Protocols](communication-protocols.md) — HTTP, TCP/UDP,
    RPC/gRPC, REST, GraphQL, WebSockets, HTTP/3 (QUIC) with decision logic.
  - [System Security Basics](system-security-basics.md) — encrypt, sanitize,
    least privilege, zero-trust, mTLS, OAuth2/OIDC, secrets management,
    observability (OpenTelemetry).
  - [Application Layer and Microservices](application-layer-microservices.md) —
    web/app layer separation, microservices, service discovery, containers,
    Kubernetes, serverless, service mesh.
  - [Cost-Aware System Design](cost-aware-system-design.md) — per-seat vs
    per-call billing, fan-out cost amplification, cap-and-shed, flat-rate vs
    metered contracts, cost-allocation tagging.
* **Update**: [overview.md](overview.md) — expanded the architecture landscape
  and concern table to include the new system-level concepts; updated scope to
  cover system-level architecture; added System Design Primer source.
* **Update**: [index.md](index.md) — added all new concept entries and updated
  bundle description.
* **Update**: [data-access-layer.md](data-access-layer.md) — added See Also
  links to Database Scaling, Caching Strategies, and System Security Basics.
* **Update**: [distribution.md](distribution.md) — added See Also links to
  CDN and DNS, Load Balancing and Reverse Proxy, and Cost-Aware System Design.

## 2026-07-18

* **Ingest**: Authored [indexed-ast-tools.md](indexed-ast-tools.md)
  — indexed AST tool selection practice for the three-contender
  landscape (CodeGraph, Graphify, GitNexus) within the AST Search (§4) and
  AST Insights (§5) rows of the 6×2 matrix. Reframed from "code knowledge graph
  tools" (a separate category) to "indexed AST tools" (the indexed entries in
  existing AST rows) — the bonus capabilities (dynamic dispatch, multi-repo,
  multimodal) are extensions of AST insights, not a different modality.
  Captures the six-round head-to-head (each tool wins exactly two rounds),
  dynamic-dispatch coverage matrix, shared design patterns (tree-sitter,
  SHA-256 caching, MCP, confidence-tagged edges, index-once-query-many,
  .gitignore hygiene), limitations of all three, and a sub-decision tree.
  Sourced from ADR-20260520001 v3.0.0 and the WiseBuilder YouTube comparison
  (2026-07-06). Cross-linked to
  [tool-detection.md](tool-detection.md),
  [adding-tools.md](adding-tools.md), and
  [tech-decision-risk-assessment.md](tech-decision-risk-assessment.md)
  (the risk hierarchy that justifies running multiple MIT-licensed indexed AST
  tools vs. adopting GitNexus's PolyForm Noncommercial license for business
  use).
* **Addition**: Authored [tech-decision-risk-assessment.md](tech-decision-risk-assessment.md)
  — ordered risk hierarchy for evaluating technology decisions, from
  highest risk (novel work, end-user impact, public API impact) down to
  lowest risk (new constant). Includes the dependency-update orthogonal
  axis (major > minor; security > capability > drift-avoidance) and the
  functional-programming-style axis (pure functions > immutable > static-wide
  > local-only mutable > wide-scope mutable > read-only). Worked example:
  better-auth vs. Supabase Auth migration decision. Cross-linked to
  [ai-human-timeline-estimates.md](ai-human-timeline-estimates.md) and
  [api-auth-payment-practices/auth-provider-selection.md](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md).
* **Addition**: Authored [ai-human-timeline-estimates.md](ai-human-timeline-estimates.md)
  — timelines must be estimated as "human review + AI execution" pairs on
  four axes (AI execution, human review, verification, tail risk), never as
  pre-AI "human days" alone. Documents what collapses with AI
  (well-trodden patterns, boilerplate, iteration, test generation) and what
  does not (security verification, migration risk on live systems, novel
  work, compliance/audit). Worked example: the better-auth middleware
  estimate correction. Cross-linked to
  [tech-decision-risk-assessment.md](tech-decision-risk-assessment.md) and
  [api-auth-payment-practices/auth-provider-selection.md](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/auth-provider-selection.md).
* **Addition**: Authored [root-cause-first.md](root-cause-first.md) —
  root-cause-first discipline: diagnose before fixing, workarounds are last
  resort, and when changing a standard update every document that restates the
  old one (never leave the other side in a broken state). Sourced from the
  dotfiles `.devin/rules/testing.md` Root-Cause First Policy, the infrahub
  Ansible AGENTS.md "Root Cause First - No Workarounds" section, and the
  dotfiles AGENTS.md "CRITICAL: Root Cause Analysis Required" section.
* **Initialization**: Created the `software-architecture-essentials` knowledge bundle to consolidate architectural practices from the `src/current/rules/software-dev/general/architecture/` rule files.
* **Creation**: Authored 10 concept pages covering philosophy, project structure, data access, configuration, distribution, theming, terminal state, tool detection, extensibility, and auth/environment.
  - [philosophy.md](philosophy.md) — domain-based modular architecture
  - [project-structure.md](project-structure.md) — domain-first hierarchical package structure
  - [data-access-layer.md](data-access-layer.md) — centralized data access with security checkpoint
  - [configuration-system.md](configuration-system.md) — layered config precedence with validation
  - [distribution.md](distribution.md) — single-binary/minimal-runtime distributions
  - [theme-system.md](theme-system.md) — single source of truth for palettes/variants
  - [terminal-state.md](terminal-state.md) — minimal terminal control sequences
  - [tool-detection.md](tool-detection.md) — PATH-first detection with caching
  - [adding-tools.md](adding-tools.md) — consistent procedure for wiring new tools
  - [auth-env.md](auth-env.md) — CI/SSH/Docker/Codespaces detection and headless auth
* **Creation**: Established [overview.md](overview.md) synthesis and [index.md](index.md) directory listing.
* **Note**: Concepts migrated from `src/current/rules/software-dev/general/architecture/*.md` (10 files).
