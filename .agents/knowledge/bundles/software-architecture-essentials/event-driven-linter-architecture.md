---
type: Practice
title: Event-Driven Linter Architecture
description: Drive a linter from an IDE event stream — JSON event protocol, trigger-based routing, async tokio channels with bounded backpressure, and compact binary serialization for internal events — synthesized across the architecture and Rust bundles.
tags: [architecture, rust, linter, event-driven, ide, tokio, async]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Event-Driven Linter Architecture

## Failure Mode

A linter is built as a batch CLI only: it scans the whole repo, prints a
report, and exits. An IDE that wants live diagnostics has to spawn the whole
binary on every keystroke, so feedback arrives seconds after the edit. The
linter has no notion of "this file changed", so it re-reads the entire tree
each time. Internal events are passed as `String` keys through a central
`match`, and a typo in one arm silently drops events.

## Practice

### IDE hook integration

The linter exposes a `--watch` mode that reads a newline-delimited JSON
event stream from stdin and writes newline-delimited JSON diagnostics to
stdout. This is the lingua franca for editor interop — every editor can
speak stdin/stdout JSON without a custom plugin.

```rust
#[derive(serde::Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Open { path: PathBuf },
    Edit { path: PathBuf, content: String },
    Save { path: PathBuf },
    Close { path: PathBuf },
}
```

### Event filtering and routing

Each rule declares a `triggers` field naming the event types it cares about
(`["edit", "save"]`). The dispatcher routes an event only to rules whose
triggers include the event type. A rule that lists no triggers is never
called in watch mode — it is a batch-only rule, and that is a valid choice.

```rust
fn dispatch(event: &Event, rules: &[Box<dyn Rule>], ctx: &ScanContext) {
    let tag = event.tag();
    for rule in rules.iter().filter(|r| r.triggers().contains(&tag)) {
        let findings = rule.evaluate(&event.as_scan_file(), ctx);
        emit(findings);
    }
}
```

### Async event processing

Use a bounded `tokio::sync::mpsc` channel between the reader and the
evaluator. The reader parses stdin into `Event`s and pushes them; the
evaluator pulls, runs rules, and writes diagnostics. This decouples I/O
latency from rule evaluation and lets multiple files be evaluated
concurrently with `tokio::spawn` per event.

```rust
let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(64);
tokio::spawn(async move { read_events(stdin, tx).await });
while let Some(event) = rx.recv().await {
    dispatch(&event, &rules, &ctx);
}
```

### Event serialization formats

- **JSON for IDE interop.** The stdin/stdout boundary is JSON because
  editors can produce and consume it trivially. Use `serde_json` with
  `#[serde(tag = "type")]` for a compact, self-describing wire format.
- **Compact binary for internal queues.** When events flow between internal
  tasks (e.g., a debouncer and the evaluator), serialize with `bincode` to
  cut allocation and bandwidth. The boundary is internal, so the format need
  not be human-readable.

Never hand-roll a text protocol for the editor boundary — JSON is already
the minimum-complexity choice.

### Backpressure

The channel is **bounded**. When the evaluator falls behind, the reader
must slow down, not buffer unboundedly. Two policies cover the realistic
cases:

- **Block the reader.** Default. `tx.send(...).await` awaits when the
  channel is full, applying natural backpressure to stdin. Best when events
  must not be lost.
- **Drop stale events.** For `Edit` events on the same path, drop older
  pending events for that path when a newer one arrives. Edits are
  supersets; an old edit's findings are stale by the time they would be
  computed. Use a per-path latest-event map before enqueue.

Never use an unbounded channel for editor events — a slow evaluator under a
fast typist will exhaust memory, and the failure is silent.

## Related Concepts

- [Plugin Scanner Registration](plugin-scanner-registration.md) — scanners
  host the rules that the event dispatcher routes to.
- [Rule Engine Design](rule-engine-design.md) — the `triggers` field and
  incremental evaluation that make per-event dispatch cheap.
