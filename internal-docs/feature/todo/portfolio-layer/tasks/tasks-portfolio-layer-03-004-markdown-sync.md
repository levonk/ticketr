---
story_id: "03-004"
story_title: "Markdown to SQLite sync (tkr sync)"
story_name: "markdown-sync"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 3
parallel_id: 1
branch: "feature/current/portfolio-layer/story-03-004-markdown-sync"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["02-002"]
parallel_safe: true
modules: ["sync", "db", "cli"]
priority: "MUST"
risk_level: "high"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story adds the `tkr sync` command that reads markdown ticket files from registered projects' `.tickets/` directories and indexes them into the SQLite `tasks` table. The sync is one-directional (markdown → SQLite); bidirectional sync comes in a later phase. Each markdown file's YAML frontmatter is parsed, and a `markdown_hash` is computed for change detection — only changed files are re-indexed. This is the foundation that makes the portfolio DB a rebuildable index of the durable markdown source of truth.

## Current State

### Relevant files

- `src/cli.rs` — CLI argument definitions and command dispatch (lines 1–265). The `Commands` enum currently has no `Sync` variant.
- `src/ticket.rs` — `TicketManager` with `list_tickets()` (lines 289–330) and `load_ticket()` (lines 172–197) methods that parse YAML frontmatter from markdown files. These patterns are directly reusable for the sync parser.
- `src/ticket.rs` — `Ticket` struct (lines 7–36) defines the YAML frontmatter schema. The sync must map these fields to the `tasks` table columns.
- `src/main.rs` — Entry point; module registration and `TicketManager` construction.
- `Cargo.toml` — Dependencies; `rusqlite` added in story 01-001, `serde_yaml` already present for parsing.
- `tests/cli_tests.rs` — Integration tests using `assert_cmd`, `predicates`, `tempfile`.
- `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md` — PRD defining the `tasks` table schema (lines 242–258), the no-corner principle (lines 158–165), and the `tkr sync` command (lines 505–507).

### Code excerpts

The `tasks` table from the PRD Data Model:

```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,             -- ticket ID (e.g. "ja-6b9a0dc")
    project_id INTEGER NOT NULL REFERENCES projects(id),
    app_id INTEGER REFERENCES apps(id),
    story_id TEXT REFERENCES stories(id),
    title TEXT NOT NULL,
    state TEXT NOT NULL,             -- logged, open, in_progress, blocked, ready, closed
    priority INTEGER DEFAULT 2,
    issue_type TEXT DEFAULT 'task',
    markdown_path TEXT NOT NULL,
    markdown_hash TEXT,
    github_issue_number INTEGER,
    github_issue_url TEXT,
    github_synced_at TEXT,
    github_etag TEXT,
    synced_at TEXT NOT NULL
);
```

YAML frontmatter parsing from `src/ticket.rs` (lines 172–197):

```rust
pub fn load_ticket(&self, id: &str) -> Result<Ticket> {
    let path = self.ticket_path(id)?;
    let content = fs::read_to_string(&path)?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid ticket format - expected frontmatter with --- separators");
    }
    let yaml_content = parts[1].trim();
    let ticket: Ticket = serde_yaml::from_str(yaml_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML for ticket {}: {}", id, e))?;
    Ok(ticket)
}
```

Ticket listing from `src/ticket.rs` (lines 289–330) — scans status directories (`open`, `in_progress`, `closed`, `blocked`, `ready`, `icebox`, `archive`) for `.md` files.

The no-corner principle from the PRD (lines 158–165):

```
- The SQLite DB is always rebuildable from markdown + GitHub API
- Markdown files are the durable, git-friendly source of truth
- GitHub Issues are an optional projection, not a dependency
- Each layer is independently durable
```

### Build / test / lint commands

```bash
devbox run just build-internal
devbox run just test-internal
devbox run just lint-internal
```

## Scope

### In scope

- Add `Sync` subcommand to `src/cli.rs` (top-level `tkr sync` with optional `--github` and `--status` flags; `--github` is a no-op stub for now, `--status` shows sync state).
- Create `src/sync.rs` module with `SyncManager` for markdown-to-SQLite indexing.
- Read all registered projects from the `projects` table.
- For each project, scan its `tickets_dir` for `.md` files in all status subdirectories (`open`, `in_progress`, `closed`, `blocked`, `ready`, `icebox`, `archive`).
- Parse YAML frontmatter from each markdown file using `serde_yaml` (reuse `Ticket` struct deserialization).
- Compute `markdown_hash` (SHA-256 of file content) for change detection.
- Upsert into the `tasks` table: if `markdown_hash` matches existing, skip; otherwise update all fields.
- Map `Ticket` fields to `tasks` columns: `id → id`, `title → title`, `status → state`, `priority → priority`, `issue_type → issue_type`, `project_id` from the registered project.
- Set `synced_at` to current UTC timestamp on each upsert.
- `tkr sync --status` prints last sync time, number of projects synced, and number of tasks indexed.
- Handle deleted markdown files: mark tasks as stale (set `state='closed'` or remove row — decision: remove row, since DB is rebuildable).
- Integration tests for sync with temp directories and markdown files.

### Out of scope

- Bidirectional sync (SQLite → markdown) — comes in a later phase with the daemon.
- GitHub sync (`--github` flag is a stub; actual GitHub sync is story 06-002).
- File watcher / daemon (story 05-001) — this story is a manual one-shot sync.
- Syncing tags, stories, or requirements from markdown — only tasks are synced in this story.
- Web API for sync status (story 05-002).
- Conflict resolution (no conflicts in one-directional sync).

## Sub-Tasks

- [ ] Add `Sync` variant to `Commands` enum in `src/cli.rs` with `--github` and `--status` flags
  - Verify: `devbox run just build-internal`
- [ ] Create `src/sync.rs` module with `SyncManager` struct wrapping a SQLite connection
  - Verify: `devbox run just build-internal`
- [ ] Implement `SyncManager::compute_hash(content: &str) -> String` using SHA-256
  - Verify: `devbox run just test-internal test_sync_hash`
- [ ] Implement `SyncManager::list_registered_projects() -> Result<Vec<Project>>` querying the `projects` table
  - Verify: `devbox run just test-internal test_sync_list_projects`
- [ ] Implement `SyncManager::scan_markdown_files(tickets_dir: &Path) -> Result<Vec<PathBuf>>` scanning all status subdirectories
  - Verify: `devbox run just test-internal test_sync_scan_files`
- [ ] Implement `SyncManager::parse_ticket_file(path: &Path) -> Result<(Ticket, String)>` returning parsed ticket + content hash
  - Verify: `devbox run just test-internal test_sync_parse_ticket`
- [ ] Implement `SyncManager::upsert_task(project_id, ticket, markdown_path, hash) -> Result<()>` inserting or updating the `tasks` row
  - Verify: `devbox run just test-internal test_sync_upsert`
- [ ] Implement `SyncManager::remove_stale_tasks(project_id, current_ids: &[String]) -> Result<()>` removing tasks whose markdown files no longer exist
  - Verify: `devbox run just test-internal test_sync_remove_stale`
- [ ] Implement `SyncManager::sync_all() -> Result<SyncReport>` orchestrating the full sync
  - Verify: `devbox run just test-internal test_sync_all`
- [ ] Implement `SyncManager::sync_status() -> Result<SyncState>` reading last sync info from `sync_state` table
  - Verify: `devbox run just test-internal test_sync_status`
- [ ] Wire `Sync` subcommand dispatch in `Commands::execute`
  - Verify: `devbox run just build-internal`
- [ ] Add `mod sync;` to `src/main.rs`
  - Verify: `devbox run just build-internal`
- [ ] Add `sha2` dependency to `Cargo.toml` if not already present
  - Verify: `devbox run just build-internal`
- [ ] Write integration tests in `tests/cli_tests.rs` for sync, sync --status, change detection, and stale removal
  - Verify: `devbox run just test-internal test_sync`
- [ ] Run full lint suite
  - Verify: `devbox run just lint-internal`

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `src/cli.rs` | Modify | Add `Sync` subcommand enum variant and dispatch |
| `src/sync.rs` | Create | New module with `SyncManager` for markdown-to-SQLite indexing |
| `src/main.rs` | Modify | Register `sync` module |
| `Cargo.toml` | Modify | Add `sha2` dependency for hashing |
| `tests/cli_tests.rs` | Modify | Add integration tests for sync commands |
| `src/ticket.rs` | Reference | Reuse `Ticket` struct and frontmatter parsing patterns |
| `src/db.rs` or `src/portfolio_db.rs` | Modify | Add task-related query helpers if DB module exists from 01-001 |

## Acceptance Criteria

- [ ] `tkr sync` scans all registered projects' `.tickets/` directories and indexes markdown files into the `tasks` table
- [ ] `tkr sync` prints a summary: number of projects synced, tasks indexed, tasks updated, tasks removed
- [ ] Running `tkr sync` twice with no file changes results in 0 updates (hash-based skip)
- [ ] Modifying a markdown file and running `tkr sync` updates the corresponding `tasks` row
- [ ] Deleting a markdown file and running `tkr sync` removes the corresponding `tasks` row
- [ ] `tkr sync --status` prints last sync timestamp, project count, and task count
- [ ] `tkr sync --github` prints "GitHub sync not yet implemented" (stub)
- [ ] Markdown files with invalid YAML frontmatter are skipped with a warning, not a hard failure
- [ ] The `markdown_hash` column is populated with SHA-256 hash of file content
- [ ] The `synced_at` column is updated on each upsert
- [ ] The `markdown_path` column stores the absolute path to the markdown file
- [ ] The `project_id` column is set from the registered project's DB ID
- [ ] All new tests pass: `devbox run just test-internal`
- [ ] Lint passes: `devbox run just lint-internal`

## Examples

### Example 1: First sync

**Input:**
```bash
tkr sync
```

**Output:**
```
Syncing 2 projects...
  Project "tkr" (id: 1): 15 tasks indexed, 0 updated, 0 removed
  Project "dotfiles" (id: 2): 4 tasks indexed, 0 updated, 0 removed
Sync complete: 19 tasks indexed, 0 updated, 0 removed
```

### Example 2: Second sync (no changes)

**Input:**
```bash
tkr sync
```

**Output:**
```
Syncing 2 projects...
  Project "tkr" (id: 1): 0 tasks indexed, 0 updated, 0 removed
  Project "dotfiles" (id: 2): 0 tasks indexed, 0 updated, 0 removed
Sync complete: 0 tasks indexed, 0 updated, 0 removed
```

### Example 3: Sync after file modification

**Input:**
```bash
# (user edits a ticket markdown file)
tkr sync
```

**Output:**
```
Syncing 2 projects...
  Project "tkr" (id: 1): 0 tasks indexed, 1 updated, 0 removed
  Project "dotfiles" (id: 2): 0 tasks indexed, 0 updated, 0 removed
Sync complete: 0 tasks indexed, 1 updated, 0 removed
```

### Example 4: Sync after file deletion

**Input:**
```bash
# (user deletes a ticket markdown file)
tkr sync
```

**Output:**
```
Syncing 2 projects...
  Project "tkr" (id: 1): 0 tasks indexed, 0 updated, 1 removed
  Project "dotfiles" (id: 2): 0 tasks indexed, 0 updated, 0 removed
Sync complete: 0 tasks indexed, 0 updated, 1 removed
```

### Example 5: Sync status

**Input:**
```bash
tkr sync --status
```

**Output:**
```
Last sync: 2026-09-08T11:45:00Z
Projects: 2
Tasks:    19
```

### Example 6: Sync with invalid markdown (graceful skip)

**Input:**
```bash
tkr sync
```
(where one markdown file has broken YAML)

**Output:**
```
Syncing 1 project...
  Warning: Failed to parse .tickets/open/broken.md: invalid YAML at line 3
  Project "tkr" (id: 1): 14 tasks indexed, 0 updated, 0 removed
Sync complete: 14 tasks indexed, 0 updated, 0 removed (1 file skipped)
```

### Example 7: GitHub sync stub

**Input:**
```bash
tkr sync --github
```

**Output:**
```
GitHub sync not yet implemented
```

## Test Plan

All tests use `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. The binary name is `tkr`.

### Integration tests to add in `tests/cli_tests.rs`

```rust
use sha2::{Digest, Sha256};

#[test]
fn test_sync_basic() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let repo_dir = temp_dir.path().join("myrepo");
    let tickets_dir = repo_dir.join(".tickets").join("open");
    fs::create_dir_all(&tickets_dir).unwrap();

    // Create a markdown ticket file
    let ticket_content = "---\nid: ja-test001\ntitle: Test Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Test Ticket\n\nDescription\n";
    fs::write(tickets_dir.join("ja-test001.md"), ticket_content).unwrap();

    // Setup: create portfolio + register project
    // (via CLI or direct DB setup)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sync complete"))
        .stdout(predicate::str::contains("1 task"));
}

#[test]
fn test_sync_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let repo_dir = temp_dir.path().join("myrepo");
    let tickets_dir = repo_dir.join(".tickets").join("open");
    fs::create_dir_all(&tickets_dir).unwrap();

    let ticket_content = "---\nid: ja-test001\ntitle: Test Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Test Ticket\n";
    fs::write(tickets_dir.join("ja-test001.md"), ticket_content).unwrap();

    // Setup: create portfolio + register project

    // First sync
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Second sync — should report 0 indexed
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("0 tasks indexed"));
}

#[test]
fn test_sync_detects_changes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let repo_dir = temp_dir.path().join("myrepo");
    let tickets_dir = repo_dir.join(".tickets").join("open");
    fs::create_dir_all(&tickets_dir).unwrap();

    let ticket_path = tickets_dir.join("ja-test001.md");
    let original = "---\nid: ja-test001\ntitle: Original Title\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Original Title\n";
    fs::write(&ticket_path, original).unwrap();

    // Setup: create portfolio + register project

    // First sync
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Modify the file
    let modified = "---\nid: ja-test001\ntitle: Updated Title\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Updated Title\n";
    fs::write(&ticket_path, modified).unwrap();

    // Second sync — should report 1 updated
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 updated"));
}

#[test]
fn test_sync_removes_deleted() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let repo_dir = temp_dir.path().join("myrepo");
    let tickets_dir = repo_dir.join(".tickets").join("open");
    fs::create_dir_all(&tickets_dir).unwrap();

    let ticket_path = tickets_dir.join("ja-test001.md");
    let content = "---\nid: ja-test001\ntitle: Test Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Test Ticket\n";
    fs::write(&ticket_path, content).unwrap();

    // Setup: create portfolio + register project

    // First sync
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Delete the file
    fs::remove_file(&ticket_path).unwrap();

    // Second sync — should report 1 removed
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 removed"));
}

#[test]
fn test_sync_status() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + register project + run sync once

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync").arg("--status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Last sync"))
        .stdout(predicate::str::contains("Projects:"))
        .stdout(predicate::str::contains("Tasks:"));
}

#[test]
fn test_sync_github_stub() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync").arg("--github")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_sync_invalid_markdown_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let repo_dir = temp_dir.path().join("myrepo");
    let tickets_dir = repo_dir.join(".tickets").join("open");
    fs::create_dir_all(&tickets_dir).unwrap();

    // Valid ticket
    let valid = "---\nid: ja-valid01\ntitle: Valid\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Valid\n";
    fs::write(tickets_dir.join("ja-valid01.md"), valid).unwrap();

    // Invalid ticket (no frontmatter)
    fs::write(tickets_dir.join("broken.md"), "This is not valid YAML\n").unwrap();

    // Setup: create portfolio + register project

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 task"))
        .stdout(predicate::str::contains("skipped"));
}

#[test]
fn test_sync_hash_computation() {
    let content = "---\nid: ja-test\ntitle: Test\n---\n";
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    let hash_hex = format!("{:x}", hash);
    assert_eq!(hash_hex.len(), 64); // SHA-256 produces 64 hex chars
}
```

## Observability

- `tkr sync` prints a per-project summary (tasks indexed, updated, removed) and a total summary.
- Invalid markdown files are skipped with a warning that includes the file path and parse error.
- `tkr sync --status` shows last sync timestamp, project count, and task count from the `sync_state` table.
- The `sync_state` table stores `last_sync_at` and `total_tasks` keys for quick status queries.
- DB operations use `anyhow::Context` to chain errors with SQL and file path context.
- Consider adding `RUST_LOG=debug` logging for per-file sync decisions (skip vs update).

## Compliance

- The no-corner principle must be upheld: the SQLite DB is always rebuildable from markdown. `tkr sync` must be idempotent and produce the same result whether run once or many times.
- The `markdown_hash` must use SHA-256 for consistency and collision resistance.
- The `synced_at` timestamp must be UTC (ISO 8601 format).
- Deleted markdown files must result in removed `tasks` rows (not orphaned rows with stale data).
- Invalid markdown files must not cause a hard failure — skip with warning and continue syncing other files.
- No secrets or credentials in markdown content should be logged or exposed.
- MIT license — no attribution boilerplate added to new files.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Large number of markdown files causes slow sync | Medium | Medium | Hash-based skip: only re-parse and upsert changed files. Benchmark with 1000+ files. |
| Markdown file moved between status directories causes duplicate tasks | Medium | High | Use ticket `id` as PK (not file path); upsert by ID, so moving directories just updates `markdown_path` |
| Invalid YAML crashes sync for all projects | Medium | High | Per-file error handling: catch parse errors, warn, and continue. Never let one bad file stop the sync. |
| DB schema not yet migrated (depends on 01-001) | Medium | High | Story 01-001 must be complete; verify `tasks` table exists |
| Project's `tickets_dir` doesn't exist or is inaccessible | Low | Medium | Check directory existence; skip project with warning if `tickets_dir` is missing |
| Race condition: file modified during sync | Low | Low | Acceptable for manual one-shot sync; daemon (05-001) will handle real-time with file watcher |
| `sha2` crate not in Cargo.toml | Low | Low | Add `sha2 = "0.10"` to Cargo.toml; verify build succeeds |

## Dependencies & Sequencing

- **Depends on**: 02-002 (Project registration) — sync reads registered projects from the `projects` table to know which `.tickets/` directories to scan.
- **Depends on**: 01-001 (Portfolio DB foundation) — the `tasks` and `sync_state` SQLite tables must exist.
- **Blocks**: 04-001 (Link tasks to stories) — sync populates the `tasks` table that task-story linking reads from.
- **Blocks**: 04-002 (Tags) — sync indexes tasks that tags attach to.
- **Blocks**: 05-001 (Daemon + file watcher) — the daemon's real-time sync reuses the `SyncManager` from this story.
- **Parallel-safe**: Yes — this story only touches `tasks` table and `sync` CLI; no overlap with app/requirement/story stories.

## Definition of Done

- [ ] All sub-tasks completed and checked off
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` succeeds with all new tests passing
- [ ] `devbox run just lint-internal` succeeds with no warnings
- [ ] `tkr sync` indexes markdown files into the `tasks` table
- [ ] Hash-based change detection works (second sync with no changes reports 0 updates)
- [ ] Deleted markdown files result in removed `tasks` rows
- [ ] Invalid markdown files are skipped with a warning
- [ ] `tkr sync --status` displays sync state
- [ ] `tkr sync --github` is a stub that prints "not yet implemented"
- [ ] `sha2` dependency added to Cargo.toml
- [ ] Conventional commit messages used (`feat(sync): ...`)
- [ ] No secrets or credentials committed
- [ ] PR describes the "why" not just the "what"

## STOP Conditions

- Stop if story 01-001 (DB foundation) or 02-002 (Project registration) is not complete — the `tasks` table and project registration must exist.
- Stop if the DB schema diverges from the PRD Data Model — coordinate with the schema owner.
- Stop if the `Ticket` struct in `src/ticket.rs` changes incompatibly — the sync parser depends on the frontmatter schema.
- Stop if `sha2` crate cannot be added to Cargo.toml (network/registry issues) — coordinate with environment setup.

## Maintenance Notes

- The `SyncManager` struct should be designed for reuse by the daemon (story 05-001) — the daemon's file watcher will call `SyncManager::upsert_task` for individual files, not `sync_all`.
- The `sync_all` method should be transactional per-project (not per-file) to avoid partial syncs on error.
- The `markdown_hash` comparison is the core optimization — ensure it's a string comparison, not a re-hash of the DB value.
- When adding bidirectional sync later, the `markdown_hash` will be used to detect conflicts (both changed = conflict).
- The `sync_state` table is a simple key-value store; extend it with new keys as needed (e.g. `last_github_sync_at`).
- The `--github` stub should be clearly marked as unimplemented to avoid user confusion.

## Commit Conventions

Use conventional commits with the `sync` module scope:

```
feat(sync): add tkr sync command for markdown-to-SQLite indexing
feat(sync): implement SHA-256 hash-based change detection
feat(sync): handle deleted markdown files by removing stale tasks
feat(sync): add --status flag for sync state reporting
feat(sync): stub --github flag for future GitHub sync
test(sync): add integration tests for sync, change detection, and stale removal
```
