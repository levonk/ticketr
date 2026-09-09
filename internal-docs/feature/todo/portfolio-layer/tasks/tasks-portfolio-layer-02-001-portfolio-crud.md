---
story_id: "02-001"
story_title: "Portfolio CRUD commands (create, list, show, dissolve)"
story_name: "portfolio-crud"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 2
parallel_id: 1
branch: "feature/current/portfolio-layer/story-02-001-portfolio-crud"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["01-001"]
parallel_safe: true
modules: ["cli", "portfolio", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story adds the `tkr portfolio` CLI subcommand with four operations — `create`, `list`, `show`, and `dissolve` — that manage portfolio records in the SQLite database. Portfolios are the top level of the 7-level hierarchy (Portfolio → Project → App → Requirement → Story → Task → AI Task) and use the curation-themed state vocabulary: `conceived`, `curated` (default), `engaged`, `paused`, and `dissolved`. The `create` command inserts a new portfolio row with a user-provided ID, name, and optional description. The `list` command displays all portfolios in a tabular format. The `show` command displays full details for a single portfolio. The `dissolve` command sets the portfolio's state to `dissolved` (soft delete — the row is retained for historical reference). All operations use the `PortfolioDb` module from story 01-001.

## Current State

### Relevant files

- **`src/cli.rs`** (lines 1-265) — The `Cli` struct uses `clap` derive macros. The `Commands` enum has variants for ticket operations (`Create`, `Start`, `Close`, `List`, `Show`, etc.), `Web`, `Tui`, `Version`. No `Portfolio` subcommand exists. The `execute()` method dispatches to `TicketManager` methods.
- **`src/main.rs`** (lines 1-37) — Parses CLI, creates `TicketManager`, calls `cli.command.execute(&mut manager).await`. No `PortfolioDb` is instantiated.
- **`src/db.rs`** (created in story 01-001) — Contains `PortfolioDb` struct with `open()`, `init_schema()`, `migrate()`, `db_path()`. No CRUD methods for portfolios yet.
- **`src/ticket.rs`** (lines 1-555) — `Ticket` struct and `TicketManager`. Unrelated to portfolio CRUD but shows the pattern for data structures.
- **`tests/cli_tests.rs`** (lines 1-705) — Integration tests using `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. Tests set `TICKETS_DIR` env var for isolation. No portfolio tests exist.

### Existing CLI structure (src/cli.rs lines 24-127)

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new ticket
    Create { ... },
    /// Set ticket status to in_progress
    Start { id: String },
    /// Set ticket status to closed
    Close { id: String },
    // ... other ticket commands ...
    /// Start web server with kanban board
    Web { host: String, port: u16 },
    /// Start terminal user interface (TUI)
    Tui,
}
```

### Existing execute dispatch (src/cli.rs lines 129-265)

```rust
impl Commands {
    pub async fn execute(self, manager: &mut TicketManager) -> anyhow::Result<()> {
        match self {
            Commands::Create { .. } => { ... },
            // ...
            Commands::Web { host, port } => {
                crate::web::start_web_server(manager, host, port).await?;
            },
            Commands::Tui => {
                crate::tui::run_tui(manager).await?;
            },
        }
        Ok(())
    }
}
```

### Portfolio schema (from PRD, created in story 01-001)

```sql
CREATE TABLE portfolios (
    id TEXT PRIMARY KEY,             -- e.g. "personal", "business-a"
    name TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'curated',    -- conceived, curated, engaged, paused, dissolved
    created TEXT NOT NULL,
    position INTEGER DEFAULT 0
);
```

### Portfolio state vocabulary (from PRD)

```
Portfolio:  conceived → curated → engaged ⇄ paused → dissolved
```

| Pre-draft | Draft | Ready | Active | Blocked | Done | Archived |
|-----------|-------|-------|--------|---------|------|----------|
| `conceived` | `curated` | `engaged` | `engaged` | `paused` | — | `dissolved` |

### Build / test / lint commands

```bash
devbox run just build-internal    # cargo build --release
devbox run just test-internal      # cargo test
devbox run just lint-internal      # cargo clippy
```

## Scope

### In scope

- Add `Portfolio` subcommand to `Commands` enum in `src/cli.rs` with nested subcommands:
  - `tkr portfolio create <id> <name> [--description=<desc>]` — creates a portfolio with state `curated`
  - `tkr portfolio list` — lists all non-dissolved portfolios
  - `tkr portfolio show <id>` — shows details of a single portfolio
  - `tkr portfolio dissolve <id>` — sets portfolio state to `dissolved`
- Add CRUD methods to `PortfolioDb` in `src/db.rs`:
  - `create_portfolio(id, name, description) -> Result<()>`
  - `list_portfolios(include_dissolved: bool) -> Result<Vec<Portfolio>>`
  - `get_portfolio(id) -> Result<Portfolio>`
  - `dissolve_portfolio(id) -> Result<()>`
- Add `Portfolio` struct in `src/db.rs` (serializable, with `id`, `name`, `description`, `state`, `created`, `position`)
- Wire `PortfolioDb` instantiation in `src/main.rs` (or lazily in the `Portfolio` command handler)
- Handle the case where the DB doesn't exist yet — auto-create and migrate
- Integration tests using `assert_cmd` with a custom `TKR_DB_PATH` env var for test isolation

### Out of scope

- Project registration CLI (story 02-002)
- App, requirement, story, task, AI task CRUD (stories 03-001 through 04-004)
- Portfolio state transitions beyond `dissolve` (e.g. `engage`, `pause` — future enhancement)
- Portfolio views (by-requirement, by-story, by-tag, by-project — story 04-003)
- Web API endpoints for portfolios (story 05-002)
- Markdown sync (story 03-004)
- Daemon (story 05-001)

## Sub-Tasks

- [ ] Add `Portfolio` struct to `src/db.rs` with `id`, `name`, `description`, `state`, `created`, `position` fields
  - Verify: `devbox run just build-internal` compiles the struct
- [ ] Add `create_portfolio()` method to `PortfolioDb` — inserts a row with state `curated`, created timestamp = now
  - Verify: unit test inserts a portfolio and queries it back
- [ ] Add `list_portfolios(include_dissolved)` method to `PortfolioDb` — returns all portfolios, optionally excluding dissolved
  - Verify: unit test inserts 2 portfolios, dissolves 1, lists with and without `include_dissolved`
- [ ] Add `get_portfolio(id)` method to `PortfolioDb` — returns a single portfolio by ID
  - Verify: unit test inserts a portfolio and retrieves it by ID
- [ ] Add `dissolve_portfolio(id)` method to `PortfolioDb` — sets state to `dissolved`
  - Verify: unit test dissolves a portfolio and confirms state is `dissolved`
- [ ] Add `Portfolio` subcommand variant to `Commands` enum in `src/cli.rs` with nested `PortfolioSubcommand` enum
  - Verify: `devbox run just build-internal` compiles with new subcommand
- [ ] Implement `portfolio create` handler — calls `PortfolioDb::create_portfolio()`
  - Verify: `tkr portfolio create personal "Personal" --description="My work"` succeeds
- [ ] Implement `portfolio list` handler — calls `PortfolioDb::list_portfolios(false)`, prints table
  - Verify: `tkr portfolio list` shows created portfolios
- [ ] Implement `portfolio show` handler — calls `PortfolioDb::get_portfolio()`, prints details
  - Verify: `tkr portfolio show personal` shows portfolio details
- [ ] Implement `portfolio dissolve` handler — calls `PortfolioDb::dissolve_portfolio()`
  - Verify: `tkr portfolio dissolve personal` succeeds and portfolio no longer appears in `list`
- [ ] Add `TKR_DB_PATH` env var support to `PortfolioDb::db_path()` for test isolation
  - Verify: integration test uses `TKR_DB_PATH` to point to a temp dir
- [ ] Wire `PortfolioDb` instantiation in the `Portfolio` command handler (open + migrate on demand)
  - Verify: `tkr portfolio create` works even when no DB file exists yet
- [ ] Write integration tests in `tests/cli_tests.rs` using `assert_cmd`
  - Verify: `devbox run just test-internal` passes all new tests
- [ ] Run clippy
  - Verify: `devbox run just lint-internal` passes with no new warnings

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `src/db.rs` | Edit | Add `Portfolio` struct, `create_portfolio()`, `list_portfolios()`, `get_portfolio()`, `dissolve_portfolio()` methods |
| `src/cli.rs` | Edit | Add `Portfolio` variant to `Commands` enum with nested `PortfolioSubcommand` enum; implement handlers |
| `src/main.rs` | Edit | Pass DB path or `PortfolioDb` instance to command execution (if needed) |
| `tests/cli_tests.rs` | Edit | Add integration tests for `portfolio create/list/show/dissolve` |

## Acceptance Criteria

- [ ] `tkr portfolio create <id> <name>` creates a portfolio with state `curated`
- [ ] `tkr portfolio create <id> <name> --description="..."` stores the description
- [ ] `tkr portfolio create` with a duplicate ID returns an error message
- [ ] `tkr portfolio list` displays all non-dissolved portfolios in a readable format
- [ ] `tkr portfolio list` does not show dissolved portfolios by default
- [ ] `tkr portfolio show <id>` displays the portfolio's ID, name, description, state, and creation date
- [ ] `tkr portfolio show <id>` with a non-existent ID returns an error
- [ ] `tkr portfolio dissolve <id>` sets the portfolio state to `dissolved`
- [ ] `tkr portfolio dissolve <id>` with a non-existent ID returns an error
- [ ] Dissolving a portfolio that is already dissolved is a no-op (or returns a message)
- [ ] The DB is auto-created and migrated if it doesn't exist when any `portfolio` command runs
- [ ] `TKR_DB_PATH` env var overrides the default DB path (for testing)
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` passes all tests (existing + new)
- [ ] `devbox run just lint-internal` passes with no new warnings

## Examples

### Example 1: Create a portfolio

**Input:**
```bash
tkr portfolio create personal "Personal" --description="Personal projects and work"
```

**Output:**
```
Created portfolio: personal (Personal)
State: curated
```

### Example 2: Create a portfolio without description

**Input:**
```bash
tkr portfolio create business-a "Business A"
```

**Output:**
```
Created portfolio: business-a (Business A)
State: curated
```

### Example 3: List portfolios

**Input:**
```bash
tkr portfolio list
```

**Output:**
```
ID            Name           State     Created
personal      Personal       curated   2026-09-08T11:45:00Z
business-a    Business A     curated   2026-09-08T11:46:00Z
```

### Example 4: Show a portfolio

**Input:**
```bash
tkr portfolio show personal
```

**Output:**
```
Portfolio: personal
Name: Personal
Description: Personal projects and work
State: curated
Created: 2026-09-08T11:45:00Z
Position: 0
```

### Example 5: Dissolve a portfolio

**Input:**
```bash
tkr portfolio dissolve personal
```

**Output:**
```
Dissolved portfolio: personal (Personal)
```

### Example 6: List after dissolving (dissolved portfolio is hidden)

**Input:**
```bash
tkr portfolio list
```

**Output:**
```
ID            Name           State     Created
business-a    Business A     curated   2026-09-08T11:46:00Z
```

### Example 7: Show a non-existent portfolio

**Input:**
```bash
tkr portfolio show nonexistent
```

**Output:**
```
Error: Portfolio not found: nonexistent
```
Exit code: 1

### Example 8: Duplicate portfolio ID

**Input:**
```bash
tkr portfolio create personal "Personal"
```

**Output:**
```
Error: Portfolio already exists: personal
```
Exit code: 1

## Test Plan

All integration tests use `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. Tests set `TKR_DB_PATH` to a temp directory path for DB isolation.

### Integration tests (tests/cli_tests.rs)

```rust
#[test]
fn test_portfolio_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .arg("--description")
        .arg("My personal work")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created portfolio: personal"))
        .stdout(predicate::str::contains("curated"));

    assert!(db_path.exists());
}

#[test]
fn test_portfolio_create_without_description() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("oss")
        .arg("Open Source")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created portfolio: oss"));
}

#[test]
fn test_portfolio_create_duplicate_id() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create first
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    // Create duplicate
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_portfolio_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create two portfolios
    for (id, name) in [("personal", "Personal"), ("business", "Business")] {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TKR_DB_PATH", &db_path)
            .arg("portfolio")
            .arg("create")
            .arg(id)
            .arg(name)
            .assert()
            .success();
    }

    // List
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("Personal"))
        .stdout(predicate::str::contains("business"))
        .stdout(predicate::str::contains("Business"));
}

#[test]
fn test_portfolio_list_excludes_dissolved() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create two portfolios
    for (id, name) in [("personal", "Personal"), ("business", "Business")] {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TKR_DB_PATH", &db_path)
            .arg("portfolio")
            .arg("create")
            .arg(id)
            .arg(name)
            .assert()
            .success();
    }

    // Dissolve one
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success();

    // List should not show dissolved
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("business"))
        .stdout(predicate::str::contains("Business"))
        .stdout(predicate::str::contains("personal").not());
}

#[test]
fn test_portfolio_show() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .arg("--description")
        .arg("My personal work")
        .assert()
        .success();

    // Show
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("show")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("Personal"))
        .stdout(predicate::str::contains("My personal work"))
        .stdout(predicate::str::contains("curated"));
}

#[test]
fn test_portfolio_show_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("show")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_portfolio_dissolve() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    // Dissolve
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dissolved"));
}

#[test]
fn test_portfolio_dissolve_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_portfolio_dissolve_already_dissolved() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create and dissolve
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success();

    // Dissolve again — should be idempotent or warn
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success();
}

#[test]
fn test_portfolio_auto_creates_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    assert!(!db_path.exists());

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    assert!(db_path.exists());
}
```

## Observability

- Each `portfolio` command prints a human-readable confirmation message to stdout
- Errors are printed to stderr with a clear message and exit code 1
- The DB path is logged on first creation (when the DB file doesn't exist)
- Future: portfolio state changes should be logged for audit (daemon story 05-001)

## Compliance

- MIT license — no new external dependencies
- No PII stored — portfolio records contain only user-defined IDs, names, and descriptions
- `dissolve` is a soft delete — data is retained for historical reference (no-corner principle: DB is rebuildable)
- No network calls in this story

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| DB path not writable | Low | High | Check parent directory exists; create it if needed; return clear error if not writable |
| Concurrent CLI invocations corrupt DB | Low | Medium | SQLite handles concurrent reads; writes are serialized by WAL mode; acceptable for CLI use |
| `TKR_DB_PATH` env var leaks into production | Low | Low | Only used for testing; default path is always used when env var is unset |
| Duplicate ID on create | Medium | Low | Check for existing ID before insert; return clear error message |
| Dissolving a portfolio with active projects | Medium | Medium | For now, allow it (soft delete only); future story may add validation |

## Dependencies & Sequencing

- **Dependencies**: 01-001 (Portfolio DB foundation) — requires `PortfolioDb`, `open()`, `migrate()`, and the `portfolios` table
- **Dependants**: 03-002 (Requirement CRUD — requirements link to portfolios), 03-003 (Story CRUD — stories may reference portfolio context), 04-003 (Portfolio views — reads portfolio data)
- **Parallel with**: 02-002 (Project registration) — both depend on 01-001 and can be developed in parallel
- **External dependencies**: None new (uses `clap`, `rusqlite`, `anyhow`, `chrono` — all already available)

## Definition of Done

- [ ] `tkr portfolio create <id> <name> [--description=<desc>]` works
- [ ] `tkr portfolio list` displays non-dissolved portfolios
- [ ] `tkr portfolio show <id>` displays portfolio details
- [ ] `tkr portfolio dissolve <id>` sets state to `dissolved`
- [ ] Duplicate ID detection works
- [ ] Non-existent ID returns an error
- [ ] DB auto-creates and migrates on first use
- [ ] `TKR_DB_PATH` env var works for test isolation
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` passes all tests (existing + new)
- [ ] `devbox run just lint-internal` passes with no new warnings
- [ ] Code reviewed and merged to the story branch

## STOP Conditions

- Stop and escalate if story 01-001 is not complete (the `PortfolioDb` module and `portfolios` table must exist)
- Stop and escalate if `clap` derive macros don't support nested subcommands in the expected way (they do, but verify the pattern compiles)
- Stop if the `TKR_DB_PATH` env var approach conflicts with existing env var conventions in the project
- Stop and escalate if existing ticket tests break after adding the `Portfolio` subcommand (indicates a CLI parsing conflict)

## Maintenance Notes

- The `Portfolio` subcommand uses clap's nested subcommand pattern — future portfolio-related commands (e.g. `tkr portfolio view`) will be added to the same `PortfolioSubcommand` enum
- The `dissolve` operation is a soft delete — the row remains in the DB with state `dissolved`. This is intentional per the no-corner principle
- The `list` command excludes dissolved portfolios by default. A future `--all` flag may be added to include them
- The `Portfolio` struct should be kept in sync with the `portfolios` table schema — any schema changes must update both
- The `TKR_DB_PATH` env var is a test-only override; document this in the module so it's not confused with a user-facing feature

## Commit Conventions

Use conventional commit messages with the `portfolio` or `db` module scope:

```
feat(db): add Portfolio struct and CRUD methods to PortfolioDb
feat(cli): add portfolio subcommand with create, list, show, dissolve
feat(cli): wire PortfolioDb auto-creation on portfolio commands
feat(cli): add TKR_DB_PATH env var for DB path override
test(cli): add integration tests for portfolio CRUD commands
```
