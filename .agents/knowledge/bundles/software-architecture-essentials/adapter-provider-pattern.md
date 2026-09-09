---
type: Practice
title: Adapter + Provider Interface Pattern
description: Define a narrow interface that captures one concern (platform, provider, storage), place it in a contract layer with zero dependencies on concrete implementations, and extend the system by implementing the interface — not by modifying the interface or the callers. The contract layer is the single source of truth; concrete adapters live behind it.
tags: [architecture, adapter-pattern, provider-pattern, interface-design, contract-layer, abstraction, extensibility, dependency-inversion]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Adapter + Provider Interface Pattern

## The General Rule

When a system must support multiple implementations of the same concern
(multiple platforms, multiple AI providers, multiple storage backends),
define a narrow interface that captures the concern, place it in a contract
layer with no dependencies on concrete implementations, and extend the system
by adding new adapters that implement the interface. The callers depend on
the interface, never on the concrete adapter.

- **Define a narrow interface per concern.** The interface captures only the
  methods the caller needs — no more. See
  [SRP + ISP](srp-isp.md). A platform adapter interface has `sendMessage` and
  auth; it does not have database access or workflow execution.
- **Place the interface in a contract layer with zero dependencies.** The
  contract layer (the file or package that defines the interface) must not
  import any concrete implementation, SDK, or runtime dependency. It is the
  single source of truth that both callers and implementers depend on.
- **Extend by implementing, not by modifying.** A new platform, provider, or
  storage backend is added by writing a new adapter that implements the
  interface. No existing caller changes. No existing adapter changes. The
  interface does not grow unless a new capability is genuinely needed by all
  implementers.
- **Callers depend on the interface, not the adapter.** A caller that needs
  to send a message receives the interface type, not the concrete Slack
  adapter. This makes the caller testable (inject a mock) and portable
  (swap the adapter without changing the caller).

## The Contract Layer

The contract layer is the key structural element. It is a package or module
that:

1. **Defines the interface(s).** The narrow methods that capture the concern.
2. **Has zero runtime dependencies.** No SDK imports, no database drivers, no
   platform-specific libraries. Only type definitions and pure functions.
3. **Is the single import target for both callers and implementers.** Callers
   import the interface from the contract layer. Implementers import the
   interface from the contract layer and provide the concrete implementation.

This prevents a circular dependency: the contract layer does not depend on
the adapters, so the adapters can depend on the contract layer without
creating a cycle.

## When To Apply

Apply the pattern when **the system has (or will have) more than one
implementation of the same concern.** If there is only one platform, one
provider, or one storage backend, and there is no concrete plan for a
second, do not extract the interface — see [YAGNI](yagni-principle.md). Wait
for the second implementation; then extract.

The pattern is not limited to external integrations. It applies to any
concern with multiple implementations: multiple cache backends (Redis,
Memcached, in-memory), multiple queue systems (SQS, RabbitMQ, in-process),
multiple logging destinations (stdout, file, network).

## How To Apply

1. **Identify the concern and its variations.** Name the concern ("sending
   messages to a chat platform") and list the variations (Slack, Telegram,
   Discord, Web).
2. **Define the narrow interface.** List the methods a caller needs. If a
   method is not needed by any caller, it does not belong on the interface.
3. **Create the contract layer.** A package or module with only the interface
   definition and any shared types the interface references. No imports of
   concrete implementations.
4. **Implement the first adapter.** The adapter imports the interface from
   the contract layer and provides the concrete implementation. The adapter
   owns the SDK dependency; the contract layer does not.
5. **Wire callers to the interface, not the adapter.** Callers receive the
   interface type (via dependency injection — see
   [Dependency Injection Pattern](dependency-injection-pattern.md)). The
   wiring (which concrete adapter to use) happens at the composition root,
   not in the caller.
6. **Add the second adapter.** Write a new adapter that implements the
   interface. No caller changes. No interface changes. This is the test of
   the pattern: if adding a second adapter requires changing callers, the
   interface is too coupled.

## Anti-Patterns

- **The interface with one implementer (premature abstraction).** An
  interface extracted for a single implementation. It adds indirection
  without extensibility. Wait for the second implementation — YAGNI.
- **The contract layer that imports SDKs.** The contract layer imports the
  Slack SDK "for convenience." Now every caller of the interface transitively
  depends on the Slack SDK, and adding a Telegram adapter requires the
  Telegram SDK to be in the contract layer too. The contract layer is no
  longer a contract — it is a dependency magnet.
- **The fat adapter interface.** The interface has methods for sending
  messages, managing threads, uploading files, and querying user presence.
  An adapter that only supports sending messages must stub the other methods.
  Split into narrow interfaces — see [SRP + ISP](srp-isp.md).
- **The caller that imports the concrete adapter.** A caller that does
  `import { SlackAdapter } from './slack'` instead of receiving the interface
  type. The caller is now coupled to Slack and cannot be tested without the
  Slack SDK.

## Concrete Instances

- **Archon (Bun + TypeScript).** The platform adapter interface
  (`IPlatformAdapter`) captures only message sending and auth. The AI
  provider interface (`IAgentProvider`) captures only query sending and
  streaming. The database interface (`IDatabase`) captures only persistence.
  The workflow store interface (`IWorkflowStore`) captures only workflow run
  persistence. Each lives in a contract layer with zero SDK dependencies —
  the provider types module has a hard rule: "This file must never import
  SDK packages or other @archon/* packages." Adapters (Slack, Telegram,
  GitHub, Discord, Claude, Codex) implement the interfaces and own their
  SDK dependencies.
- **Rust `tower::Service`.** The `Service` trait captures a single concern:
  "given a request, produce a response." Every HTTP client, server, and
  middleware in the Tower ecosystem implements `Service`. The trait lives in
  `tower-service` with no HTTP dependencies. Adding a new transport (QUIC,
  Unix sockets) means implementing `Service` — no caller changes.
- **Go `database/sql`.** The `database/sql` package defines the `Driver`
  interface. Concrete drivers (pgx, mysql, sqlite) implement `Driver` and
  register via `sql.Register`. Callers use `sql.DB` and never import the
  concrete driver. The contract layer (`database/sql`) has no driver
  dependencies. Adding a new database means writing a new driver — no caller
  changes.

## See Also

- [SRP + ISP](srp-isp.md) — narrow interfaces are the foundation of this
  pattern; a fat adapter interface violates ISP.
- [Dependency Injection Pattern](dependency-injection-pattern.md) — callers
  receive the interface via DI; the adapter pattern and DI are paired.
- [YAGNI Principle](yagni-principle.md) — do not extract the interface until
  the second implementation is concrete.
- [DRY + Rule of Three](dry-rule-of-three.md) — the decision to extract the
  interface follows the same logic as extracting a utility: wait for the
  pattern to stabilize.
