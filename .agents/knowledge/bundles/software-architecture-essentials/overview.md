---
type: Synthesis
title: Software Architecture Essentials Overview
description: Synthesis of software architecture practices — philosophy, project structure, data access, configuration, distribution, system-level scalability, consistency, load balancing, caching, queues, protocols, security, cost, microservices, decentralized P2P, business models, theming, terminal state, tool detection, and auth/environment.
tags: [architecture, overview, synthesis, modular, separation-of-concerns]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
---

---
description: STE100-inspired Simplified Technical English guidelines for technical prose output — active voice, short sentences, one-word-one-meaning, imperative for instructions
---

### Simplified Technical English (STE100-Inspired)

This artifact produces technical English (instructions, procedures, descriptions,
reference documentation). Apply these STE100-inspired guidelines to all technical
prose output so the result is unambiguous, translatable, and easy to read for
non-native speakers and for AI agents that must execute the steps precisely.

These are **STE-inspired guidelines**, not the full ASD-STE100 vocabulary
restriction. Domain terms (Dockerfile, pnpm, devbox, Nx, etc.) are permitted
when they are the correct technical term — STE100's 1000-word approved
vocabulary is too narrow for this domain. The goal is the *clarity discipline*
of STE100, not its word list.

For the full writing rules, before/after examples, and the approved-words
reference, see the
[Simplified Technical English](https://github.com/levonk/skills-releases/blob/main/knowledge/simplified-technical-english/simplified-technical-english.md)
and
[Detailed Guide](https://github.com/levonk/skills-releases/blob/main/knowledge/simplified-technical-english/detailed-guide.md)
concept pages in the `simplified-technical-english` knowledge bundle. The
bundle is the canonical, publicly-reachable home for these guidelines; this
include is the build-time gist that gets inlined into skills and other
bundles.

#### Core Principles

1. **One word, one meaning.** Pick one term for each concept and use it
   everywhere. Do not alternate between "image" and "container image" and
   "docker image" for the same thing. Pick one, define it once, reuse it.

2. **Short sentences.** Keep procedural sentences under 20 words. Keep
   descriptive sentences under 25 words. Split long sentences into two.

3. **Active voice.** Write "The build copies the file" not "The file is copied
   by the build." The actor does the action. Passive voice hides who does what
   and is the single largest source of ambiguity in technical prose.

4. **Imperative mood for instructions.** Write "Run the tests" not "You should
   run the tests" or "The tests should be run." Instructions tell the reader
   (or agent) what to do, directly.

5. **One topic per sentence.** One idea per sentence. Do not chain unrelated
   clauses with "and" or "while." If a sentence has two ideas, split it.

6. **Consistent verb forms.** Use the same verb for the same action across the
   document. If you "run" a script in section 1, do not "execute" it in
   section 2. Pick one verb per action and keep it.

7. **Approved modifiers only.** Avoid decorative adjectives and adverbs
   ("very", "extremely", "simply", "just"). Keep modifiers that carry
   information ("non-root", "read-only", "idempotent"). Drop modifiers that
   carry only emphasis.

8. **Define every acronym on first use.** Write "Continuous Integration (CI)"
   on first use, then "CI" thereafter. Never assume the reader knows the
   acronym.

#### What Counts as Technical English

Apply these guidelines to:

- Procedural instructions ("Run `just build`", "Add the user to the group")
- Descriptions of failure modes, symptoms, and practices
- Reference documentation and concept pages
- Checklists and review guidance
- Synthesis and overview prose in knowledge bundles

Do **not** apply these guidelines to:

- Code, commands, and file paths (those have their own syntax)
- Frontmatter and structured data (YAML, JSON)
- Diagrams and their source syntax (Mermaid, PlantUML)
- Business communication, marketing copy, or creative content
- Log entries and change logs (those are append-only records)

#### Quick Self-Check

Before finishing a piece of technical prose, run this checklist:

- [ ] Is every sentence under 25 words? (Procedural: under 20.)
- [ ] Is every sentence active voice? (Or is the passive voice intentional and
      necessary?)
- [ ] Are instructions in imperative mood?
- [ ] Does each technical term have one and only one form in this document?
- [ ] Is every acronym defined on first use?
- [ ] Are decorative modifiers removed?
- [ ] Does each sentence carry one topic?

If any answer is "no," revise before publishing.


# Software Architecture Essentials Overview

This bundle documents architectural practices for building modular,
maintainable software. It spans two levels:

- **Codebase-level architecture** — how to structure, access data, configure,
  and extend a code base.
- **System-level architecture** — how to scale, distribute, secure, and cost
  a running system, including both the client-server / microservices axis and
  the decentralized P2P axis.

Each concept captures a specific architectural concern and the practice that
addresses it, with decision checklists for choosing among alternatives.

## The Architecture Landscape

```
philosophy → project-structure → data-access → configuration → distribution
                                ↓
       root-cause-first → measure-before-optimizing → data-structures-first → perceived-latency-driven-design → tech-decision-risk → ai-human-timelines → technology-selection-pattern
                                ↓
                  theme → terminal-state → tool-detection → adding-tools → indexed-ast-tools → auth-env
                                ↓
   scalability → cap/consistency → load-balancing → cdn/dns → database-scaling
                                ↓
       caching → asynchronism → protocols → security → microservices → resilience-patterns → cost
                                ↓
                decentralized-p2p → business-models-around-open-protocols
```

| Concern | Practice | Prevents |
|---------|----------|----------|
| Discipline | [Root-Cause First](root-cause-first.md) | Workaround stacks, contradictory documents, silent guards, band-aids that outlive the bug |
| Profiling | [Measure Before Optimizing](measure-before-optimizing.md) | Hunch-driven speed hacks, optimizing the wrong level, caching without measuring, speculative fast paths for hypothetical large n |
| Data Model | [Data Structures First](data-structures-first.md) | Clever algorithms over mis-shaped structures, smart code over dumb data, parallel indexes, representable illegal states |
| Latency Target | [Perceived-Latency-Driven Design](perceived-latency-driven-design.md) | Round-number SLOs detached from human perception, request-response without prefetch, optimizing the backend past the perception threshold, reporting p50 instead of p99 |
| Decisions | [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) | Gut-feel choices, single-axis arguments, treating a new constant and a new public API as equally scary |
| Selection | [Technology Selection Pattern](technology-selection-pattern.md) | Undocumented "we chose X" decisions, cargo-cult inheritance, re-litigation, stale decisions, missing why-not |
| Estimates | [AI + Human Timeline Estimates](ai-human-timeline-estimates.md) | Pre-AI "human days" estimates that overstate AI-cheap work and understate unbounded tail risk |
| Philosophy | [Architecture Philosophy](philosophy.md) | Cross-domain leakage, untestable modules, refactoring risk |
| Structure | [Project Structure](project-structure.md) | Flat package sprawl, scattered feature code, unclear ownership |
| Data | [Data Access Layer](data-access-layer.md) | Duplicated data logic, missing auth checkpoints, debugging pain |
| Config | [Configuration System](configuration-system.md) | Override confusion, invalid configs, silent failures |
| Distribution | [Distribution and Packaging](distribution.md) | Heavy runtimes, untracked sizes, undocumented install paths |
| Theming | [Theme System](theme-system.md) | Color drift, broken runtime switching, inconsistent state colors |
| Terminal | [Terminal State Management](terminal-state.md) | Buffer clearing, input races, corrupted terminal state |
| Detection | [Tool Detection Architecture](tool-detection.md) | Brittle PATH assumptions, missing tools, slow re-detection |
| Extensibility | [Adding New Tools](adding-tools.md) | Inconsistent CLI surfaces, scattered wiring, missing tests/docs |
| Intelligence | [Indexed AST Tool Selection](indexed-ast-tools.md) | Defaulting to one indexed AST tool for every workload; losing the ~58% tool-call reduction by picking the wrong sweet spot; adopting a PolyForm-NC tool for business use without a license |
| Auth/Env | [Authentication and Environment Management](auth-env.md) | Browser auth in headless, duplicated env detection, silent auth failures |
| Scalability | [Scalability Fundamentals](scalability-fundamentals.md) | Confusing performance with scalability, using outdated latency numbers, wrong capacity math |
| Consistency/Availability | [CAP, Consistency, and Availability](cap-consistency-availability.md) | Treating all data as strongly consistent, ignoring error budgets, untested fail-over |
| Traffic | [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) | SPOFs, wrong L4/L7 choice, missing service mesh for east-west traffic |
| Edge | [CDN and DNS](cdn-and-dns.md) | Serving static content from origin, stale DNS records, no geographic routing |
| Persistence | [Database Scaling](database-scaling.md) | Defaulting to one RDBMS for every workload, missing specialized stores |
| Cache | [Caching Strategies](caching-strategies.md) | Cache invalidation mistakes, wrong update strategy, cold starts |
| Async | [Asynchronism and Queues](asynchronism-and-queues.md) | Blocking on slow work, unbounded queues, duplicate processing |
| Protocols | [Communication Protocols](communication-protocols.md) | Wrong protocol for the client, latency, or P2P requirement; ignoring browser support and NAT traversal |
| Security | [System Security Basics](system-security-basics.md) | Plaintext secrets, implicit trust inside the cluster, untraceable breaches |
| Services | [Application Layer and Microservices](application-layer-microservices.md) | Monolith forced into microservices too early, missing service discovery |
| Resilience | [Resilience Patterns](resilience-patterns.md) | Unbounded fan-out (the canonical invariant violation), no choke point, static-only concurrency limits, retry-storms without budgets, relying on Hystrix or the mesh for internal fan-out, unbounded log emission on the failure path |
| Decentralization | [Decentralized P2P Architecture](decentralized-p2p-architecture.md) | Forcing client-server onto a censorship-resistance requirement, wrong state model for P2P, browser-as-node pitfalls |
| Cost | [Cost-Aware System Design](cost-aware-system-design.md) | Unbounded metered API bills, no cost attribution, wrong pricing model |
| Revenue | [Business Models Around Open Protocols](business-models-around-open-protocols.md) | Toll-based models on open protocols, middleman arbitrage, ads inside contracts |
| Plugins | [Plugin Scanner Registration](plugin-scanner-registration.md) | Hardcoded scanner lists, no plugin discovery, missing lifecycle management |
| Rules | [Rule Engine Design](rule-engine-design.md) | Unordered rule evaluation, rule conflicts, no incremental evaluation on file changes |
| Traits | [Trait-Based Extensibility](trait-based-extensibility.md) | Rigid rule hierarchies, no default impls, trait object vs generic confusion |
| Config Design | [Config-Driven Tool Design](config-driven-tool-design.md) | Cross-bundle: flat config, no profile activation, no schema migration, no hot-reload |
| Events | [Event-Driven Linter Architecture](event-driven-linter-architecture.md) | Cross-bundle: no IDE hook integration, unbounded event queues, no backpressure |
| Multi-Lang | [Multi-Language Scanner Coordination](multi-language-scanner-coordination.md) | Cross-bundle: hardcoded language dispatch, no shared scanner interface, no polyglot analysis |

## Scope

This bundle covers **software architecture practices** — the structural
decisions and subsystem patterns that keep a codebase and a running system
modular, maintainable, and extensible. It does **not** cover:

- Build orchestration (Nx, Turborepo) — see
  [typescript-monorepo-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/overview.md).
- Container build environments — see
  [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md).
- Cloud provider-specific implementation details — see
  [cloud-provider-essentials](https://github.com/levonk/skills-releases/blob/main/knowledge/cloud-provider-essentials/overview.md).
- Developer environment setup — see
  [dev-environment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/overview.md).

## Compounding

New lessons from future architecture work — new subsystem patterns, cross-cutting
concerns, scaling thresholds — should be filed as new concept pages. Append to
`log.md` when adding.

## Related Knowledge Bundles

- [dev-environment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/overview.md) —
  Environment setup that hosts the architecture.
- [typescript-monorepo-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/overview.md)
  — Monorepo structure conventions that implement project-structure.md.
- [cloud-provider-essentials](https://github.com/levonk/skills-releases/blob/main/knowledge/cloud-provider-essentials/overview.md) —
  Cloud infrastructure that the distribution practice deploys to.
- [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md) —
  Container packaging that implements the distribution practice.
- [api-auth-payment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/api-auth-payment-practices/overview.md) —
  Worked example: the auth-provider-selection decision applies the
  tech-decision-risk hierarchy and the AI + human timeline estimate format.

## Sources

- `src/current/rules/software-dev/general/architecture/*.md` — 10 architecture rule files migrated 2026-07-18.
- [The System Design Primer](https://github.com/donnemartin/system-design-primer) —
  ingested 2026-07-24 for system-level architecture concepts.
- Rob Pike, "Notes on Programming in C" (1989) — the 5 Rules of Programming,
  ingested 2026-08-27 for measure-before-optimizing (rules 1-2),
  data-structures-first (rule 5), and the KISS cross-reference (rules 3-4).
- Fred Brooks, *The Mythical Man-Month* (1975) — "Show me your tables...",
  ingested 2026-08-27 for data-structures-first.
- [p99 0 ms* autocomplete for 240 million domain names](https://ruurtjan.com/articles/p99-0ms-autocomplete-for-240-million-domain-names)
  — Ruurtjan Pul, 2026-06-22. Ingested 2026-08-30 for
  perceived-latency-driven-design (the new concept: latency budgets derived
  from measured typing cadence, `keyDown` prefetch + `keyUp` render, bounded
  alphabet fan-out) and for concrete instances on measure-before-optimizing
  (the stopping rule), data-structures-first (trie + mmap block index),
  scalability-fundamentals (budget from perception, not a round number),
  caching-strategies (client-side prefetch as cache-aside), and cdn-and-dns
  (edge caching of hot prefix paths).
- [Nielsen Norman Group — Response Times: The 3 Important Limits](https://www.nngroup.com/articles/response-times-3-important-limits/)
  — the 0.1 s / 1 s / 10 s perception thresholds that frame the
  perceived-latency-driven-design budget. Ingested 2026-08-30.

---

## Content Ordering

This artifact is optimized for machine consumption. Generic framework content
(shared includes, knowledge bundles) appears before the skill-specific body.
This ordering maximizes cross-skill prefix caching: skills that share the same
includes produce identical byte prefixes, so an LLM context cache warmed by one
skill serves all skills that share the same preamble.

This is sub-optimal for human reading — the skill-specific content starts deep
in the file, after the generic preamble. Human readers can jump to the
skill-specific body by searching for the first `# ` heading that follows the
generic sections. Each section is self-contained and documented with its own
heading hierarchy.

