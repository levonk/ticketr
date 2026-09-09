---
type: Practice
title: Dependency Injection Pattern
description: Inject dependencies (database, AI provider, config, platform) as narrow interface parameters — never import them inside the module. Define injection types that capture only what the module needs. Callers satisfy the interfaces structurally; no adapter wrappers needed when the concrete type already matches the interface shape. The composition root is the only place that knows which concrete implementations are wired.
tags: [architecture, dependency-injection, di, inversion-of-control, testability, coupling, composition-root]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Dependency Injection Pattern

## The General Rule

A module should not import its dependencies; it should receive them. The
module declares the interfaces it needs, and the caller (the composition
root) provides concrete implementations that satisfy those interfaces. This
inverts the dependency direction: the module depends on interfaces, not on
concrete modules.

- **Inject dependencies as narrow interface parameters.** A function that
  needs a database receives a `Database` interface, not a concrete
  `PostgresPool`. A function that needs an AI provider receives a
  `Provider` interface, not a concrete `ClaudeProvider`.
- **Define injection types that capture only what the module needs.** The
  injection type is a narrow interface — a subset of the full adapter
  interface. A module that only sends messages does not receive the full
  platform adapter; it receives a `MessageSender` interface with one method.
- **Callers satisfy the interfaces structurally.** If the concrete type
  already has the methods the interface requires, no wrapper is needed. The
  caller passes the concrete type directly; the module accepts it as the
  interface type. This is structural typing (duck typing at the type level).
- **The composition root is the only place that knows which concrete
  implementations are wired.** Every other module depends on interfaces. The
  composition root (the `main` function, the app initializer, the test
  setup) is where concrete adapters are constructed and injected.

## Why It Matters

Without DI, a module that needs a database imports the database module
directly. This creates a hard dependency: the module cannot be tested without
the database, cannot be reused with a different database, and cannot be
understood without reading the database module. The dependency graph is
rigid and the module is coupled to a concrete implementation.

With DI, the module declares "I need something that can do X" and the caller
provides it. The module is testable (inject a mock), reusable (inject a
different implementation), and understandable (the interface tells you what
the module needs, without reading the implementation).

## The Injection Type

The injection type is the narrow interface the module receives. It should
capture only what the module needs — no more. This is ISP applied to DI: a
module that only reads from the database receives a `DataReader` interface,
not a `Database` interface with read, write, and delete methods.

When the concrete type already satisfies the narrow interface, no wrapper is
needed. This is the power of structural typing: the module declares the
shape it needs, and any concrete type with that shape is accepted. In
nominally-typed languages (Java, C#), an adapter wrapper may be needed if
the concrete type does not explicitly implement the interface; in
structurally-typed languages (Go, TypeScript), the concrete type is accepted
if its shape matches.

## How To Apply

1. **List the module's dependencies.** What does the module need to do its
   job? Database access, AI provider, config, logging, platform messaging.
2. **Define a narrow interface for each dependency.** The interface has only
   the methods the module calls. If the module only reads, the interface has
   only read methods.
3. **Accept the interfaces as parameters.** The module's constructor or
   function signature takes the interfaces as parameters — it does not
   import the concrete implementations.
4. **At the composition root, construct concrete implementations and inject
   them.** The `main` function (or the test setup) creates the concrete
   database, provider, and platform, and passes them to the module.
5. **In tests, inject mocks that satisfy the interfaces.** The test
   constructs a mock database, mock provider, and mock platform, and passes
   them to the module. The module is tested in isolation.

## Anti-Patterns

- **The service locator.** A module that calls a global `getService('db')`
  to obtain its dependencies. This hides the dependencies in the function
  body, makes the module untestable without the service locator, and creates
  a hidden global dependency. Inject instead.
- **The import inside the module.** A module that `import { pool } from
  './db'` at the top of the file. The module is now coupled to the concrete
  database. Inject instead.
- **The fat injection type.** An injection type that includes methods the
  module does not call. The module receives a `Database` interface with 20
  methods but only calls 2. Define a narrow interface with only the 2
  methods — see [SRP + ISP](srp-isp.md).
- **The wrapper that adds nothing.** A wrapper class that wraps the concrete
  adapter and forwards every method to it, just to satisfy a nominally-typed
  interface. If the concrete type already has the right shape (structural
  typing), pass it directly.

## Concrete Instances

- **Archon (Bun + TypeScript).** The workflow engine defines `WorkflowDeps`
  — an injection type that captures only what the engine needs: a platform
  interface (`IWorkflowPlatform`, a subset of `IPlatformAdapter`), a workflow
  store (`IWorkflowStore`), and provider types. The engine does not import
  the concrete database, AI provider, or platform adapter. Callers in
  `@archon/core` satisfy the interfaces structurally — no adapter wrappers
  are needed. The engine is tested by injecting mock implementations.
- **Go `net/http` middleware.** A middleware function receives an
  `http.Handler` and returns an `http.Handler`. The middleware does not
  import the concrete handler; it receives it. This is DI at the function
  level — the middleware is a function that takes a dependency and returns a
  new handler with that dependency wired in.
- **Python FastAPI `Depends`.** A route function declares
  `db: Session = Depends(get_db)`. FastAPI injects the database session at
  call time. The function does not import the database; it declares the
  dependency and the framework provides it. Tests override `get_db` to
  inject a test session.

## See Also

- [Adapter + Provider Pattern](adapter-provider-pattern.md) — DI injects
  adapter interfaces; the two patterns are paired.
- [SRP + ISP](srp-isp.md) — the injection type must be narrow; a fat
  injection type violates ISP.
- [KISS Principle](kiss-principle.md) — DI with structural typing is simpler
  than DI with adapter wrappers; prefer the simpler form.
- [YAGNI Principle](yagni-principle.md) — do not inject a dependency the
  module does not use; the injection type should capture only what is needed.
