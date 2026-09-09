---
story_id: "01-001"
story_title: "Add rusqlite dependency + portfolio DB module with schema migrations"
story_name: "portfolio-db-foundation"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 1
parallel_id: 1
branch: "feature/current/portfolio-layer/story-01-001-portfolio-db-foundation"
status: "todo"
assignee: ""
reviewer: ""
dependencies: []
parallel_safe: true
modules: ["db", "schema"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story introduces the SQLite foundation for the Portfolio Layer by adding the `rusqlite` crate (v0.40 with the `bundled` feature) to `Cargo.toml` and creating a new `src/db.rs` module that initializes and migrates the portfolio database at `~/.local/share/tkr/portfolio.db`. The module creates the full 10-table schema defined in the PRD (portfolios, projects, apps, requirements, stories, tasks, ai_tasks, tags, task_tags, priority_order, sync_state) with proper foreign keys, defaults, and unique constraints. A schema versioning mechanism using the `sync_state` table ensures forward-compatible migrations. This is the foundational data layer that all subsequent portfolio stories (02-001, 02-002, and beyond) depend on.

## Current State

### Relevant files

- **`Cargo.toml`** (lines 14-34) — Current dependencies include `clap`, `anyhow`, `serde`, `tokio`, `warp`, `chrono`, `uuid`, etc. No SQLite dependency exists yet.
- **`src/main.rs`** (lines 1-37) — Declares modules: `cli`, `ticket`, `utils`, `web`, `tui`. No `db` module is declared.
- **`src/ticket.rs`** (lines 1-555) — The `Ticket` struct and `TicketManager` handle per-project markdown tickets. No SQLite integration.
- **`src/cli.rs`** (lines 1-265) — CLI commands for ticket CRUD. No portfolio/project subcommands.
- **`src/web.rs`** (lines 1-230) — Warp web server serving ticket API. No portfolio API endpoints.
- **`src/utils.rs`** (lines 1-57) — `find_tickets_dir` and `get_repo_root` helpers. No DB path resolution.

### Existing dependency block (Cargo.toml lines 14-34)

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
anyhow = "1.0"
thiserror = "1.0"
directories = "5.0"
ctrlc = "3.5"
indicatif = "0.17"
glob = "0.3"
is-terminal = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.12"
shell-escape = "0.1"
uuid = { version = "1.0", features = ["v4"] }
tokio = { version = "1.0", features = ["full"] }
warp = "0.3.7"
url = "2.0"
ratatui = "0.24"
crossterm = "0.27"

[dev-dependencies]
tempfile = "3.12"
assert_cmd = "2.1"
predicates = "3.0"
```

### Existing module declarations (src/main.rs lines 1-5)

```rust
mod cli;
mod ticket;
mod utils;
mod web;
mod tui;
```

### Build / test / lint commands

```bash
devbox run just build-internal    # cargo build --release
devbox run just test-internal      # cargo test
devbox run just lint-internal      # cargo clippy
```

## Scope

### In scope

- Add `rusqlite = { version = "0.40", features = ["bundled"] }` to `Cargo.toml` `[dependencies]`
- Create `src/db.rs` module with:
  - `PortfolioDb` struct wrapping a `rusqlite::Connection`
  - `PortfolioDb::open(path)` — opens (or creates) the SQLite database, enabling WAL mode and foreign keys
  - `PortfolioDb::init_schema()` — creates all 10 tables from the PRD schema if they don't exist
  - `PortfolioDb::migrate()` — reads schema version from `sync_state` and applies incremental migrations
  - `PortfolioDb::db_path()` — resolves `~/.local/share/tkr/portfolio.db` using the `directories` crate (already a dependency)
  - Constants for all table DDL statements
  - `SCHEMA_VERSION` constant (set to `1`)
- Register `mod db;` in `src/main.rs`
- Unit tests for schema creation, idempotent re-runs, and migration version tracking
- Integration test verifying the DB file is created at the expected path

### Out of scope

- Portfolio CRUD CLI commands (story 02-001)
- Project registration CLI commands (story 02-002)
- Any CLI subcommand changes
- Any web API endpoint changes
- Markdown-to-DB sync logic (story 03-004)
- Daemon / file watcher (story 05-001)
- GitHub sync (story 06-002)

## Sub-Tasks

- [ ] Add `rusqlite = { version = "0.40", features = ["bundled"] }` to `Cargo.toml` `[dependencies]`
  - Verify: `devbox run just build-internal` compiles with rusqlite
- [ ] Create `src/db.rs` with `PortfolioDb` struct and `open()` constructor
  - Verify: `devbox run just build-internal` compiles the new module
- [ ] Implement `db_path()` using the `directories` crate's `ProjectDirs` or `BaseDirs` to resolve `~/.local/share/tkr/portfolio.db`
  - Verify: unit test confirms path ends with `tkr/portfolio.db`
- [ ] Implement `init_schema()` with all 10 CREATE TABLE statements from the PRD
  - Verify: unit test opens DB, calls `init_schema()`, queries `sqlite_master` for all 10 table names
- [ ] Implement `migrate()` with schema version tracking via `sync_state` table
  - Verify: unit test calls `migrate()` twice and confirms version is unchanged on second call
- [ ] Enable WAL mode (`PRAGMA journal_mode=WAL`) and foreign keys (`PRAGMA foreign_keys=ON`) on connection open
  - Verify: unit test queries `PRAGMA foreign_keys` and confirms it returns `1`
- [ ] Register `mod db;` in `src/main.rs`
  - Verify: `devbox run just build-internal` compiles with module registered
- [ ] Write unit tests in `src/db.rs` (inline `#[cfg(test)]` module)
  - Verify: `devbox run just test-internal` passes all new tests
- [ ] Run clippy on the new module
  - Verify: `devbox run just lint-internal` passes with no new warnings

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Edit | Add `rusqlite` dependency |
| `src/db.rs` | Create | New module with `PortfolioDb` struct, schema DDL, migrations |
| `src/main.rs` | Edit | Add `mod db;` declaration |
| `tests/cli_tests.rs` | Edit (optional) | Add integration test for DB path resolution |

## Acceptance Criteria

- [ ] `Cargo.toml` contains `rusqlite = { version = "0.40", features = ["bundled"] }`
- [ ] `src/db.rs` exists and exports a `PortfolioDb` struct
- [ ] `PortfolioDb::open(path)` creates the database file if it doesn't exist
- [ ] `PortfolioDb::init_schema()` creates all 10 tables: `portfolios`, `projects`, `apps`, `requirements`, `stories`, `tasks`, `ai_tasks`, `tags`, `task_tags`, `priority_order`, `sync_state`
- [ ] All tables have the columns, types, defaults, and constraints defined in the PRD schema
- [ ] `PortfolioDb::migrate()` is idempotent — calling it multiple times does not error
- [ ] Schema version is stored in the `sync_state` table with key `schema_version`
- [ ] Foreign keys are enabled (`PRAGMA foreign_keys=ON`)
- [ ] WAL journal mode is enabled
- [ ] `src/main.rs` declares `mod db;`
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` succeeds
- [ ] `devbox run just lint-internal` succeeds with no new warnings

## Examples

### Example 1: Opening a new database

**Input:**
```rust
let db = PortfolioDb::open("/tmp/test-portfolio.db")?;
```

**Output:**
- File `/tmp/test-portfolio.db` is created on disk
- `db` is a `PortfolioDb` instance with an open `rusqlite::Connection`
- All 10 tables exist in the database

### Example 2: Idempotent schema initialization

**Input:**
```rust
let db = PortfolioDb::open("/tmp/test-portfolio.db")?;
db.init_schema()?;
db.init_schema()?; // second call
```

**Output:**
- No error on the second call
- Querying `SELECT count(*) FROM sqlite_master WHERE type='table'` returns `11` (10 tables + `sync_state`)

### Example 3: Migration version tracking

**Input:**
```rust
let db = PortfolioDb::open("/tmp/test-portfolio.db")?;
db.migrate()?;
let version: String = db.conn.query_row(
    "SELECT value FROM sync_state WHERE key = 'schema_version'",
    [],
    |row| row.get(0),
)?;
```

**Output:**
- `version` is `"1"`

### Example 4: Default path resolution

**Input:**
```rust
let path = PortfolioDb::db_path()?;
```

**Output:**
- On Linux: `/home/<user>/.local/share/tkr/portfolio.db`
- On macOS: `/Users/<user>/Library/Application Support/tkr/portfolio.db`

## Test Plan

All tests use `tempfile::TempDir` for isolated database paths. Tests are defined as inline `#[cfg(test)]` modules in `src/db.rs` and as integration tests in `tests/cli_tests.rs`.

### Unit tests (src/db.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_creates_database_file() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("portfolio.db");
        assert!(!db_path.exists());
        let _db = PortfolioDb::open(&db_path).unwrap();
        assert!(db_path.exists());
    }

    #[test]
    fn test_init_schema_creates_all_tables() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.init_schema().unwrap();

        let table_names = [
            "portfolios", "projects", "apps", "requirements",
            "stories", "tasks", "ai_tasks", "tags",
            "task_tags", "priority_order", "sync_state",
        ];
        for name in &table_names {
            let count: i32 = db.conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    rusqlite::params![name],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table '{}' was not created", name);
        }
    }

    #[test]
    fn test_init_schema_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.init_schema().unwrap();
        db.init_schema().unwrap(); // should not error

        let count: i32 = db.conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 11); // 10 tables + sync_state
    }

    #[test]
    fn test_migrate_sets_schema_version() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();

        let version: String = db.conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "1");
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        db.migrate().unwrap(); // should not error
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        let fk: i32 = db.conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn test_wal_mode_enabled() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        let mode: String = db.conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn test_portfolio_default_state() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();

        // Insert a portfolio without specifying state
        db.conn.execute(
            "INSERT INTO portfolios (id, name, created) VALUES (?1, ?2, ?3)",
            rusqlite::params!["test-portfolio", "Test", "2026-09-08T00:00:00Z"],
        ).unwrap();

        let state: String = db.conn
            .query_row(
                "SELECT state FROM portfolios WHERE id = 'test-portfolio'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "curated");
    }

    #[test]
    fn test_project_default_state() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();

        // Insert a project without specifying state
        db.conn.execute(
            "INSERT INTO projects (portfolio_id, name, repo_path, tickets_dir, registered_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["test-portfolio", "tkr", "/tmp/tkr", "/tmp/tkr/.tickets", "2026-09-08T00:00:00Z"],
        ).unwrap();

        let state: String = db.conn
            .query_row(
                "SELECT state FROM projects WHERE name = 'tkr'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "seeded");
    }

    #[test]
    fn test_db_path_ends_with_portfolio_db() {
        let path = PortfolioDb::db_path().unwrap();
        assert!(path.to_string_lossy().ends_with("portfolio.db"));
        assert!(path.to_string_lossy().contains("tkr"));
    }
}
```

### Integration tests (tests/cli_tests.rs)

```rust
#[test]
fn test_db_module_does_not_break_existing_commands() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("list")
        .assert()
        .success();
}
```

## Observability

- All schema operations log to `stderr` at info level (e.g. "Initialized portfolio DB at /path/to/portfolio.db")
- Migration version is queryable via `SELECT value FROM sync_state WHERE key = 'schema_version'`
- WAL mode enables better concurrent read performance and is observable via `PRAGMA journal_mode`

## Compliance

- MIT license — `rusqlite` is MIT licensed; the `bundled` feature bundles SQLite which is in the public domain
- No new PII is stored; the DB contains only project metadata and ticket references
- The DB path follows XDG Base Directory specification on Linux (`~/.local/share/tkr/`)
- No network calls are made in this story

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| `rusqlite` `bundled` feature increases compile time | High | Low | Acceptable for a CLI tool; `bundled` avoids system SQLite version mismatches |
| Schema migration conflicts with future stories | Medium | High | Use `sync_state` versioning from day one; all schema changes go through `migrate()` |
| DB path resolution differs across platforms | Medium | Medium | Use the `directories` crate (already a dependency) for cross-platform path resolution |
| Foreign key constraints block test data setup | Low | Medium | Tests insert parent records before children; use `CREATE TABLE IF NOT EXISTS` for idempotency |
| WAL mode leaves `-wal` and `-shm` sidecar files | Low | Low | These are normal SQLite artifacts; they are cleaned up on checkpoint |

## Dependencies & Sequencing

- **No dependencies** — this is the first story in the portfolio layer
- **Dependants**: 02-001 (Portfolio CRUD), 02-002 (Project registration), and all subsequent stories that interact with the DB
- **External dependency**: `rusqlite` v0.40 (released Aug 2026 per PRD references)
- **Existing dependency used**: `directories` v5.0 (already in `Cargo.toml`) for path resolution

## Definition of Done

- [ ] `Cargo.toml` includes `rusqlite = { version = "0.40", features = ["bundled"] }`
- [ ] `src/db.rs` exists with `PortfolioDb` struct, `open()`, `init_schema()`, `migrate()`, `db_path()`
- [ ] All 10 tables from the PRD schema are created with correct columns, types, defaults, and constraints
- [ ] `sync_state` table tracks `schema_version`
- [ ] `src/main.rs` declares `mod db;`
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` passes all tests (existing + new)
- [ ] `devbox run just lint-internal` passes with no new warnings
- [ ] Code reviewed and merged to the story branch

## STOP Conditions

- Stop and escalate if `rusqlite` v0.40 with `bundled` feature fails to compile in the devbox environment
- Stop and escalate if the `directories` crate API has changed in v5.0 and `ProjectDirs`/`BaseDirs` is unavailable
- Stop and escalate if existing tests break after adding the `db` module (indicates a module conflict)
- Stop if schema DDL from the PRD contains syntax errors that SQLite rejects — escalate to PRD owner for clarification

## Maintenance Notes

- The `SCHEMA_VERSION` constant must be incremented every time `migrate()` adds a new migration step
- New tables should be added in `migrate()` with `CREATE TABLE IF NOT EXISTS` to support upgrades
- The `bundled` feature means SQLite is compiled from source — no system SQLite dependency is required
- WAL mode is persistent (stored in the DB header) — it only needs to be set once, but setting it on every `open()` is harmless
- The `PortfolioDb` struct holds a single `rusqlite::Connection` — for concurrent access in the daemon (story 05-001), a connection pool or mutex will be needed

## Commit Conventions

Use conventional commit messages with the `db` module scope:

```
feat(db): add rusqlite dependency for portfolio SQLite storage
feat(db): create PortfolioDb module with schema initialization
feat(db): add schema migration with version tracking via sync_state
feat(db): enable WAL mode and foreign keys on connection open
test(db): add unit tests for schema creation and migration idempotency
```
