---
type: Practice
title: Structured Logging with Tracing
description: Use the tracing crate over log, structure fields with file paths and rule IDs, configure tracing-subscriber with env-filter, honor RUST_LOG, and log IDE hook events as structured spans.
tags: [rust, tracing, logging, structured-logs, env-filter, tracing-subscriber]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Structured Logging with Tracing

## Failure Mode

The `log` crate emits unstructured strings that grep-only tools cannot slice.
Free-form messages hide rule IDs and file paths, so dashboards aggregate noise.
Missing env-filter means `--verbose` either floods or starves. Hook events
logged as plain text lose the correlation IDs agents need.

## Practice

### Tracing over Log

Use `tracing` for any tool that emits more than a handful of lines. Spans carry
context across await points; events attach to the active span. Reserve `log` for
leaf libraries with no structured needs.

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

### Structured Field Conventions

Pass fields as key/value pairs, not interpolated strings. Pick stable field
names so consumers can filter reliably.

- `file.path` — absolute path to the file under analysis
- `rule.id` — the lint rule identifier
- `severity` — `error`, `warn`, `info`
- `duration.ms` — elapsed time for a phase

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(self), fields(file.path = %path.display()))]
fn check_file(&self, path: &std::path::Path) {
    info!(rule.id = "no-bare-println", severity = "warn", "violation");
}
```

Never embed the path in the message. The subscriber formats fields consistently
across stdout, JSON, and TOON outputs.

### Subscriber Configuration

Initialize once at program start. Default to `WARN`, let `RUST_LOG` override,
and route logs to stderr so stdout stays result-only.

```rust
use tracing_subscriber::{EnvFilter, fmt};

fn init_logs() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn"));
    fmt().with_env_filter(filter).with_writer(std::io::stderr).init();
}
```

### RUST_LOG

Document the directive grammar in `--help`: `RUST_LOG=project_lint=debug,notify=warn`.
Treat the env var as the single source of truth for verbosity; do not add a
parallel `--verbose` flag that fights it.

### IDE Hook Events as Spans

Wrap each hook invocation in a span. Record the hook kind, session id, and
target agent so downstream tooling can reconstruct the timeline.

```rust
#[instrument(fields(hook.kind = %kind, session.id = %sid, agent = %agent))]
async fn run_hook(kind: HookKind, sid: &str, agent: &str) { /* ... */ }
```

Emit a single `info!` event on completion with `duration.ms`. Avoid per-line
spans for hot loops; they dwarf the actual work.

## Related Concepts

- [Async Patterns](async-patterns.md) — spans across await points
- [File Watcher Patterns](file-watcher-patterns.md) — logging watcher events
- [CLI Tool Standards](cli-tool-standards.md) — stderr-only log discipline
