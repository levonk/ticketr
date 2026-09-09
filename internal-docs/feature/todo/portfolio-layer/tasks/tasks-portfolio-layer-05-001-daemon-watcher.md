---
story_id: "05-001"
story_title: "File watcher + daemon process management"
story_name: "daemon-watcher"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 5
parallel_id: 1
branch: "feature/current/portfolio-layer/story-05-001-daemon-watcher"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["03-004"]
parallel_safe: true
modules: ["daemon", "watcher"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend", "daemon"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Implement a daemon process that watches registered projects' `.tickets/` directories for file changes and automatically re-parses markdown tickets into the portfolio SQLite database. The daemon runs as a background process with PID file management, supporting `start`, `stop`, `status`, and `restart` subcommands. This is the live-sync backbone that keeps the SQLite index in sync with the markdown source of truth without requiring manual `tkr sync` invocations.

## Current State

- `src/web.rs` (lines 26-80) starts a warp web server but does not run as a background daemon — it blocks the foreground.
- `src/cli.rs` (lines 118-127) has a `Web` subcommand but no `Daemon` subcommand.
- `src/ticket.rs` (lines 289-330) has `list_tickets()` that scans `.tickets/` status directories and parses markdown frontmatter.
- Story 03-004 (dependency) implements the markdown-to-SQLite sync logic (`tkr sync`); this story wraps that logic in a file-watcher daemon.
- No `notify` crate dependency exists in `Cargo.toml` (line 14-34).
- No PID file management or background process spawning exists.
- `src/main.rs` (lines 1-37) uses `#[tokio::main]` and runs commands in the foreground.

## Scope

### In Scope

- Add `notify = "6"` dependency to `Cargo.toml`.
- Create `src/daemon.rs` module with:
  - File watcher using the `notify` crate that watches registered projects' `.tickets/` directories.
  - Debounced file change events (coalesce rapid writes within ~500ms).
  - On file change: re-parse the affected markdown file, update the SQLite `tasks` table (upsert by task ID).
  - On file delete: remove the task row from SQLite.
  - On file create: insert a new task row into SQLite.
- Daemon process management:
  - `tkr daemon start` — spawns daemon as a detached background process, writes PID to `~/.local/share/tkr/daemon.pid`.
  - `tkr daemon stop` — reads PID file, sends SIGTERM, waits for exit, removes PID file.
  - `tkr daemon status` — checks if PID in PID file is alive, reports daemon state, registered projects, and last sync time.
  - `tkr daemon restart` — stop + start.
- PID file location: `~/.local/share/tkr/daemon.pid` (use `directories` crate for XDG compliance, already a dependency).
- Daemon log file: `~/.local/share/tkr/daemon.log` for structured logging.
- Register `Daemon` subcommand in `src/cli.rs` with `Start`, `Stop`, `Status`, `Restart` variants.
- Wire `mod daemon;` into `src/main.rs`.
- The daemon should optionally start the web server (warp) in the same process so the API and watcher share state.

### Out of Scope

- GitHub bidirectional sync (Story 06-002).
- Web API endpoints (Story 05-002).
- Web UI / Kanban board (Story 06-001).
- Priority ordering persistence (Story 06-003).
- The actual SQLite schema and migration logic (Story 01-001).

## Sub-Tasks

1. **Add `notify` dependency** — Add `notify = "6"` to `Cargo.toml` `[dependencies]` section.
2. **Create `src/daemon.rs` module** — Define `Daemon` struct with `start()`, `stop()`, `status()`, `restart()` methods and a `run()` loop that initializes the file watcher and event loop.
3. **Implement file watcher** — Use `notify::Watcher` to watch all registered projects' `.tickets/` directories. Debounce events with a 500ms window. Map file events to sync actions (create/update/delete).
4. **Implement markdown re-parse on change** — On file change event, re-parse the markdown ticket (reuse `TicketManager::load_ticket`), compute `markdown_hash` (SHA-256 of file content), and upsert into the SQLite `tasks` table. Only write if hash changed (skip no-op writes).
5. **Implement PID file management** — Write PID to `~/.local/share/tkr/daemon.pid` on start. On stop, read PID, send SIGTERM via `nix` crate or `std::process`, wait up to 5s for exit, then SIGKILL if needed. Remove PID file on clean exit.
6. **Implement daemon logging** — Write structured logs to `~/.local/share/tkr/daemon.log` with timestamps. Log: daemon start/stop, file events, sync actions, errors.
7. **Add `Daemon` CLI subcommand** — Add `Daemon { action: DaemonAction }` to `Commands` enum in `src/cli.rs` with `DaemonAction` enum (`Start`, `Stop`, `Status`, `Restart`).
8. **Implement background process spawning** — On `tkr daemon start`, fork/spawn a detached child process that runs the daemon loop. The parent exits immediately after confirming the child started.
9. **Integrate web server into daemon** — The daemon process optionally starts the warp web server (from `src/web.rs`) in the same Tokio runtime, sharing the SQLite connection.
10. **Write integration tests** — Test daemon start/stop/status lifecycle, file change triggers SQLite update, PID file creation and cleanup.

## Relevant Files

| File | Role | Changes |
|------|------|---------|
| `Cargo.toml` | Dependencies | Add `notify = "6"` |
| `src/daemon.rs` | **NEW** — Daemon module | File watcher, process management, event loop |
| `src/main.rs` | Entry point | Add `mod daemon;` |
| `src/cli.rs` | CLI commands | Add `Daemon` subcommand with `Start`/`Stop`/`Status`/`Restart` variants |
| `src/web.rs` | Web server | Refactor `start_web_server` to be callable from daemon context |
| `src/ticket.rs` | Ticket parsing | Reuse `TicketManager::load_ticket` for re-parsing on file change |
| `tests/cli_tests.rs` | Integration tests | Add daemon lifecycle tests, file watcher sync tests |
| `~/.local/share/tkr/daemon.pid` | Runtime | PID file (created at runtime, not committed) |
| `~/.local/share/tkr/daemon.log` | Runtime | Log file (created at runtime, not committed) |

## Acceptance Criteria

- [ ] `notify = "6"` is added to `Cargo.toml` and `devbox run just build-internal` succeeds.
- [ ] `tkr daemon start` spawns a background daemon process and writes a PID file to `~/.local/share/tkr/daemon.pid`.
- [ ] `tkr daemon stop` reads the PID file, sends SIGTERM to the daemon, waits for exit, and removes the PID file.
- [ ] `tkr daemon status` reports whether the daemon is running, lists registered projects, and shows last sync time.
- [ ] `tkr daemon restart` stops and then starts the daemon.
- [ ] When a markdown ticket file is modified in a registered project's `.tickets/` directory, the daemon re-parses it and updates the SQLite `tasks` table within 2 seconds.
- [ ] When a markdown ticket file is created, the daemon inserts a new row into the SQLite `tasks` table.
- [ ] When a markdown ticket file is deleted, the daemon removes the corresponding row from the SQLite `tasks` table.
- [ ] Rapid file writes (multiple saves within 500ms) are debounced into a single sync action.
- [ ] The daemon writes structured logs to `~/.local/share/tkr/daemon.log`.
- [ ] If the daemon process dies unexpectedly, `tkr daemon status` detects the stale PID and reports "not running".
- [ ] All new daemon commands have integration tests in `tests/cli_tests.rs`.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.

## Examples

### Start the daemon

```bash
$ tkr daemon start
Daemon started (PID 12345)
Watching 3 projects:
  - tkr (/Users/micro/p/gh/levonk/tkr/.tickets)
  - dotfiles (/Users/micro/p/gh/levonk/dotfiles/.tickets)
  - infrahub (/Users/micro/p/gh/levonk/infrahub/.tickets)
Logs: ~/.local/share/tkr/daemon.log
```

### Check daemon status

```bash
$ tkr daemon status
Daemon: running (PID 12345)
Uptime: 2h 15m
Watching 3 projects
Last sync: 2026-09-08T14:30:00Z
Events processed: 42
```

### Stop the daemon

```bash
$ tkr daemon stop
Daemon stopped (PID 12345)
PID file removed.
```

### File change triggers sync (daemon.log)

```
[2026-09-08T14:30:00Z] INFO  File changed: /Users/micro/p/gh/levonk/tkr/.tickets/open/ja-6b9a0dc.md
[2026-09-08T14:30:00Z] INFO  Re-parsing ticket ja-6b9a0dc
[2026-09-08T14:30:00Z] INFO  SQLite upsert: ja-6b9a0dc (hash changed, state=open)
[2026-09-08T14:30:01Z] INFO  File deleted: /Users/micro/p/gh/levonk/tkr/.tickets/blocked/ja-old1234.md
[2026-09-08T14:30:01Z] INFO  SQLite delete: ja-old1234
```

## Test Plan

### Unit Tests

1. **PID file write/read** — Test that PID file is written correctly and can be read back.
2. **PID file stale detection** — Write a PID for a non-existent process, verify `status` reports "not running".
3. **Debounce logic** — Simulate rapid file events, verify only one sync action fires.
4. **Hash comparison** — Verify that identical file content (same hash) does not trigger a SQLite write.

### Integration Tests (in `tests/cli_tests.rs`)

1. **`test_daemon_start_creates_pid_file`** — Run `tkr daemon start`, assert PID file exists and contains a valid PID.
2. **`test_daemon_stop_removes_pid_file`** — Start daemon, run `tkr daemon stop`, assert PID file is removed.
3. **`test_daemon_status_running`** — Start daemon, run `tkr daemon status`, assert output contains "running".
4. **`test_daemon_status_not_running`** — Without starting, run `tkr daemon status`, assert output contains "not running".
5. **`test_daemon_restart`** — Start daemon, run `tkr daemon restart`, assert new PID differs from old PID.
6. **`test_daemon_file_change_syncs_to_sqlite`** — Start daemon, modify a markdown ticket file, assert SQLite `tasks` table reflects the change within 2 seconds.
7. **`test_daemon_file_create_inserts_row`** — Start daemon, create a new markdown ticket file, assert new row appears in SQLite.
8. **`test_daemon_file_delete_removes_row`** — Start daemon, delete a markdown ticket file, assert row is removed from SQLite.

### Test Isolation

- Use `tempfile::TempDir` for the tickets directory and override `XDG_DATA_HOME` to redirect PID/log files to a temp location.
- Each test starts and stops its own daemon instance to avoid interference.

## Observability

- **Log file**: `~/.local/share/tkr/daemon.log` with structured lines: `[timestamp] LEVEL message`.
- **Log levels**: `INFO` for sync actions, `WARN` for conflicts, `ERROR` for failures.
- **`tkr daemon status`** exposes: PID, uptime, project count, last sync timestamp, events processed count.
- **Metrics** (future): Expose Prometheus metrics endpoint if web server is running.

## Compliance

- PID file must be cleaned up on all exit paths (SIGTERM handler, panic, normal exit).
- Daemon must not hold exclusive locks on the SQLite database — use WAL mode for concurrent reads.
- File watcher must handle the case where a `.tickets/` directory does not exist yet (skip, don't crash).
- All file paths must be resolved via the `directories` crate for XDG compliance.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Daemon process leaks (orphaned after parent exits) | Medium | Medium | PID file + `tkr daemon stop` + stale PID detection in `status` |
| File watcher misses events on rapid saves | Low | Medium | Debounce with 500ms window; also run full rescan on `tkr sync` |
| SQLite lock contention between daemon and CLI | Medium | High | Use WAL mode (`PRAGMA journal_mode=WAL`); retry on `SQLITE_BUSY` with backoff |
| Cross-platform file watching differences (macOS FSEvents vs Linux inotify) | Low | Low | `notify` crate abstracts this; test on both platforms |
| Daemon crashes on malformed markdown | Medium | High | Wrap each file parse in `Result`; log error and continue, don't crash the watcher |

## Dependencies & Sequencing

- **Depends on**: `03-004` (Markdown to SQLite sync) — the daemon reuses the sync logic from this story.
- **Dependants**: `06-002` (GitHub sync) — the GitHub sync runs inside the daemon process and uses the same SQLite connection.
- **Parallel-safe**: Yes — this story only touches `src/daemon.rs`, `src/cli.rs` (Daemon subcommand), and `Cargo.toml`. It does not conflict with `05-002` (portfolio API) which extends `src/web.rs`.
- **External dependencies**: `notify = "6"` (file watcher crate).

## Definition of Done

- [ ] `notify = "6"` added to `Cargo.toml`.
- [ ] `src/daemon.rs` created with file watcher, process management, and event loop.
- [ ] `mod daemon;` added to `src/main.rs`.
- [ ] `Daemon` subcommand added to `src/cli.rs` with `Start`/`Stop`/`Status`/`Restart`.
- [ ] PID file management works (create on start, remove on stop, stale detection).
- [ ] File changes in `.tickets/` trigger SQLite updates within 2 seconds.
- [ ] Integration tests pass for all daemon lifecycle commands.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.
- [ ] No secrets or credentials committed.
- [ ] Conventional commit message used.

## STOP Conditions

- If the `notify` crate fails to build on the target platform, stop and investigate platform-specific issues before proceeding.
- If the daemon cannot spawn a detached background process on the current OS, stop and evaluate using `daemonize` crate or `systemd` user units as alternatives.
- If SQLite WAL mode causes issues with the existing `rusqlite` connection setup from Story 01-001, stop and coordinate with the 01-001 assignee.
- If file watcher events are unreliable on the development platform, stop and add a polling fallback.

## Maintenance Notes

- The daemon PID file path (`~/.local/share/tkr/daemon.pid`) follows XDG spec via the `directories` crate `ProjectDirs::data_dir()`.
- The daemon log file (`~/.local/share/tkr/daemon.log`) should be rotated if it grows large — consider adding log rotation in a future story.
- The debounce window (500ms) is a constant in `src/daemon.rs` and can be tuned via environment variable `TKR_DAEMON_DEBOUNCE_MS` in the future.
- The daemon should gracefully handle the addition/removal of registered projects at runtime (re-scan project list from SQLite on each watcher event or on a periodic timer).

## Commit Conventions

- Use conventional commits: `feat(daemon): add file watcher and process management`.
- Split commits by sub-task if the diff is large (e.g., one commit for `notify` dependency, one for `src/daemon.rs`, one for CLI subcommand, one for tests).
- Reference story ID in PR description: `Story: 05-001`.
- Branch: `feature/current/portfolio-layer/story-05-001-daemon-watcher`.
