---
type: Practice
title: SRP + ISP — Single Responsibility + Interface Segregation
description: Keep each module and package focused on one concern. Extend behavior by implementing existing narrow interfaces. Avoid fat interfaces and god modules that mix policy, transport, and storage. Do not add unrelated methods to an existing interface — define a new one.
tags: [architecture, srp, isp, single-responsibility, interface-segregation, modularity, coupling]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# SRP + ISP — Single Responsibility + Interface Segregation

## The General Rule

**SRP** (Single Responsibility Principle) says a module should have one, and
only one, reason to change. **ISP** (Interface Segregation Principle) says
clients should not be forced to depend on methods they do not use. The two
are a pair: SRP governs the module, ISP governs the interface the module
exposes.

- **Keep each module and package focused on one concern.** A module that
  handles HTTP routing, business logic, and database access has three reasons
  to change. Split it into three modules, each with one reason.
- **Extend behavior by implementing existing narrow interfaces.** When a new
  platform, provider, or storage backend arrives, it should implement the
  existing interface that captures its concern — not force a new method onto
  an interface that was designed for a different concern.
- **Avoid fat interfaces and "god modules" that mix policy, transport, and
  storage.** A fat interface is one with methods for multiple concerns; a god
  module is one that imports from every layer. Both make the system
  untestable (you cannot mock one concern without mocking all) and
  unchangeable (a change to one concern ripples through all callers).
- **Do not add unrelated methods to an existing interface — define a new
  one.** If an interface currently has `send()` and you need `delete()`, ask
  whether `delete()` belongs on the same interface. If `send()` clients do
  not need `delete()`, define a separate `Deletable` interface.

## Why They Are Paired

SRP without ISP produces modules that are internally focused but externally
fat — one module, one concern, but an interface with twenty methods that
every client must implement. ISP without SRP produces many narrow interfaces
that are all implemented by one god module — the interfaces are segregated
but the module is not.

The correct application is: one module per concern (SRP), and each module
exposes one or more narrow interfaces, each capturing a single capability
(ISP). A client depends only on the interface for the capability it needs.

## How To Apply

1. **Identify the concern.** Before writing a module, name the one concern it
   owns. If you cannot name it in one phrase, the module is doing too much.
2. **Define the narrowest interface that captures that concern.** List the
   methods a client of that concern needs. If a method is not needed by any
   client of this concern, it does not belong on this interface.
3. **When a new capability is needed, decide: same concern or new concern?**
   If it is the same concern, add the method to the existing interface. If it
   is a different concern, define a new interface and let the module
   implement both.
4. **When a module grows beyond one concern, split it.** The split produces
   two modules, each with one concern and one or more narrow interfaces. The
   old module's callers are updated to depend on the new narrow interfaces.
5. **Prefer multiple narrow interfaces over one wide interface.** A module
   that implements `Readable`, `Writable`, and `Closable` is better than one
   that implements `Resource` with all three methods. A client that only
   reads depends only on `Readable`.

## Anti-Patterns

- **The god module.** A module that imports from the transport layer, the
  storage layer, and the business logic layer. It has every reason to change
  and cannot be tested in isolation.
- **The fat interface.** An interface with methods for multiple concerns
  (e.g., `send()`, `store()`, `configure()`). Every implementer must provide
  all three, even if it only does one.
- **The "just one more method" creep.** An interface that started with three
  methods and grew to fifteen because each new feature added "just one more
  method." The interface is now fat and every implementer has stub methods
  for capabilities it does not support.
- **The empty-interface abstraction.** An interface with one method and one
  implementer. This is a YAGNI violation (see [YAGNI](yagni-principle.md)) —
  the interface exists for a hypothetical second implementer that never
  arrived.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Engineering Principles state: "Keep each
  module and package focused on one concern; extend behavior by implementing
  existing narrow interfaces whenever possible; avoid fat interfaces and god
  modules that mix policy, transport, and storage; do not add unrelated
  methods to an existing interface — define a new one." The platform adapter
  interface (`IPlatformAdapter`) captures only message sending and auth — it
  does not include database access or workflow execution. The AI provider
  interface (`IAgentProvider`) captures only query sending and streaming —
  it does not include platform-specific concerns. The workflow store
  interface (`IWorkflowStore`) captures only persistence — it does not
  include execution. Each is narrow and has multiple implementers.
- **Rust `std::io`.** `Read`, `Write`, `BufRead`, and `Seek` are separate
  traits. A type that can read but not seek implements only `Read`. A client
  that only reads depends only on `Read`. This is ISP at the language level —
  the standard library does not have a single `Io` trait with every method.
- **Python `collections.abc`.** `Iterable`, `Iterator`, `Sequence`,
  `Mapping`, `Set` are separate abstract base classes. A type that is
  iterable but not indexable implements only `Iterable`. A function that only
  iterates accepts `Iterable` and does not require `Sequence`. Narrow
  interfaces, multiple implementers, clients depend on the minimum.

## See Also

- [Adapter + Provider Pattern](adapter-provider-pattern.md) — narrow
  interfaces are the foundation of the adapter pattern; each adapter
  implements one narrow interface.
- [Dependency Injection Pattern](dependency-injection-pattern.md) — injecting
  narrow interfaces (not concrete modules) is what makes DI testable.
- [KISS Principle](kiss-principle.md) — a focused module is simpler than a
  god module; SRP is the structural expression of simplicity.
- [YAGNI Principle](yagni-principle.md) — do not create an interface until
  there is a second implementer; an interface with one implementer is a guess.
