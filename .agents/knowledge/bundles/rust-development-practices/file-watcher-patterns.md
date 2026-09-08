---
type: Practice
title: File Watcher Patterns
description: Filter and debounce notify events, respect per-platform watch limits, handle FS races on rename, recover from channel disconnection, and bridge to async runtimes via tokio mpsc.
tags: [rust, notify, file-watcher, debouncing, async, tokio, cross-platform]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# File Watcher Patterns

## Failure Mode

Raw notify events fire per byte written. Without debouncing, a single save
triggers dozens of re-checks. Linux hits the inotify watch limit and silently
drops events. A rename mid-scan produces a vanished-file panic. A disconnected
channel hangs the watcher thread. Blocking the runtime on a std channel stalls
every task.

## Practice

### Event Filtering and Debouncing

Use `notify-debouncer-mini` (or `notify-debouncer-full`) to coalesce bursts.
Tune the debounce window to the editor in play — 200 ms covers most save
dialogs. Filter to the extensions you lint before forwarding.

```rust
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};

let (tx, rx) = std::sync::mpsc::channel();
let mut deb = new_debouncer(Duration::from_millis(200), None, tx)?;
deb.watcher().watch(&root, Recursive)?;
for r in rx {
    for ev in r.map_err(|e| log::error!("watch err: {e:?}"))? {
        if is_lintable(&ev.path) { handle(ev.path); }
    }
}
```

### Cross-Platform Watch Limits

Each OS caps watchers differently. Document the limit and fail loudly when
exceeded rather than silently dropping.

- Linux inotify: 8192 watches per user (`fs.inotify.max_user_watches`)
- macOS FSEvents: no hard per-user limit, but stream scheduling adds latency
- Windows ReadDirectoryChangesW: limited by handle count and buffer size

On Linux, detect `notify::ErrorKind::MaxWatchersExceeded` and emit a structured
error with the sysctl to raise.

### FS Race Conditions

A file may be renamed or deleted between the event and the scan. Stat the path
before reading; treat `NotFound` as a no-op, not an error. For renames, match
the `from` and `to` events by debounce batch and re-scan only the new path.

```rust
match fs::read_to_string(&path) {
    Ok(src) => check(&path, &src),
    Err(e) if e.kind() == NotFound => tracing::warn!(file.path = %path.display(), "vanished"),
    Err(e) => return Err(e.into()),
}
```

### Watcher Error Recovery

The debouncer channel can disconnect if the watcher thread panics. Read in a
loop; on `RecvError`, log and rebuild the watcher from the known root set.

### Async Runtime Bridge

Forward std channel events into a tokio mpsc channel so the runtime owns
processing. Spawn a dedicated thread for the blocking watcher; never call
`watcher()` from an async context.

```rust
let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(64);
std::thread::spawn(move || {
    let (wtx, wrx) = std::sync::mpsc::channel();
    let mut deb = new_debouncer(Duration::from_millis(200), None, wtx).unwrap();
    deb.watcher().watch(&root, Recursive).unwrap();
    for ev in wrx { for e in ev.unwrap() { let _ = tx.blocking_send(e.path.clone()); } }
});
```

## Related Concepts

- [Async Patterns](async-patterns.md) — tokio mpsc bridge
- [Structured Logging with Tracing](structured-logging-tracing.md) — logging watcher events
- [Cross-Platform Path Handling](cross-platform-path-handling.md) — path normalization on events
