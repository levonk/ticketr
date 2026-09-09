---
story_id: "03-002"
story_title: "Requirement CRUD commands (create, list, show, supersede)"
story_name: "requirement-crud"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 3
parallel_id: 1
branch: "feature/current/portfolio-layer/story-03-002-requirement-crud"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["02-001"]
parallel_safe: true
modules: ["cli", "requirement", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story adds Requirement-level CRUD commands to the tkr CLI (`tkr requirement create`, `tkr requirement list`, `tkr requirement show`, `tkr requirement supersede`). Requirements are durable constraints at the strategic level of the hierarchy — they live under a portfolio and group stories. Requirement state follows the specification vocabulary: `surfaced`, `proposed`, `planned`, `current`, `superseded`. Superseding a requirement marks it as replaced, preventing new stories from being attached to it.

## Current State

### Relevant files

- `src/cli.rs` — CLI argument definitions and command dispatch (lines 1–265). The `Commands` enum currently has no `Requirement` variant.
- `src/ticket.rs` — `TicketManager` struct; provides patterns for ID generation using timestamps and UUIDs (lines 108–140).
- `src/main.rs` — Entry point; module registration and `TicketManager` construction.
- `Cargo.toml` — Dependencies; `rusqlite` added in story 01-001, `uuid` already present.
- `tests/cli_tests.rs` — Integration tests using `assert_cmd`, `predicates`, `tempfile`.
- `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md` — PRD defining the `requirements` table schema (lines 217–227) and CLI commands (lines 474–477).

### Code excerpts

The `requirements` table from the PRD Data Model:

```sql
CREATE TABLE requirements (
    id TEXT PRIMARY KEY,             -- e.g. "req-abc123"
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id),
    title TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'proposed',  -- surfaced, proposed, planned, current, superseded
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0
);
```

CLI commands from the PRD:

```bash
tkr requirement create "Title" --portfolio=<id> --description="..."
tkr requirement list
tkr requirement show <id>
tkr requirement supersede <id>
```

ID generation pattern from `src/ticket.rs` (lines 108–140):

```rust
pub fn generate_id(&self) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let uuid = uuid::Uuid::new_v4();
    let uuid_str = uuid.as_simple().to_string();
    let hash = format!("{:x}{}", timestamp % 10000, &uuid_str[..4]);
    Ok(format!("{}-{}", prefix, hash))
}
```

### Build / test / lint commands

```bash
devbox run just build-internal
devbox run just test-internal
devbox run just lint-internal
```

## Scope

### In scope

- Add `Requirement` subcommand to `src/cli.rs` with `Create`, `List`, `Show`, and `Supersede` sub-subcommands.
- Create `src/requirement.rs` module with `RequirementManager` for CRUD operations against the `requirements` SQLite table.
- Generate requirement IDs with `req-` prefix (e.g. `req-abc123`).
- `tkr requirement create` accepts `--portfolio=<id>`, `--description=<text>`, `--target-date=<YYYY-MM-DD>`, and optional `--state=<state>` (defaults to `proposed`).
- `tkr requirement list` optionally accepts `--portfolio=<id>` filter and displays id, title, state, portfolio, and target date.
- `tkr requirement show <id>` displays full requirement details including description, state, target date, and linked stories (if any exist).
- `tkr requirement supersede <id>` transitions state to `superseded` (terminal) and warns if stories are still attached.
- Validate requirement state values: `surfaced`, `proposed`, `planned`, `current`, `superseded`.
- Integration tests for all four commands.

### Out of scope

- Web API endpoints for requirements (story 05-002).
- Portfolio views by-requirement (story 04-003).
- Requirement deletion (requirements are superseded, not deleted — preserves referential integrity for stories).
- Automatic state transitions (e.g. `planned → current` when first story ships) — manual for now.
- Web UI for requirement management (story 06-001).

## Sub-Tasks

- [ ] Add `Requirement` variant to `Commands` enum in `src/cli.rs` with `Create`, `List`, `Show`, `Supersede` sub-subcommands
  - Verify: `devbox run just build-internal`
- [ ] Create `src/requirement.rs` module with `RequirementManager` struct wrapping a SQLite connection
  - Verify: `devbox run just build-internal`
- [ ] Implement `RequirementManager::generate_id() -> Result<String>` producing `req-<hash>` IDs
  - Verify: `devbox run just test-internal test_requirement_id_format`
- [ ] Implement `RequirementManager::create_requirement(portfolio_id, title, description, state, target_date) -> Result<String>`
  - Verify: `devbox run just test-internal test_requirement_create`
- [ ] Implement `RequirementManager::list_requirements(portfolio_id: Option<String>) -> Result<Vec<Requirement>>`
  - Verify: `devbox run just test-internal test_requirement_list`
- [ ] Implement `RequirementManager::show_requirement(id) -> Result<Requirement>` with linked story count
  - Verify: `devbox run just test-internal test_requirement_show`
- [ ] Implement `RequirementManager::supersede_requirement(id) -> Result<()>` updating state to `superseded`
  - Verify: `devbox run just test-internal test_requirement_supersede`
- [ ] Add state validation helper: `validate_requirement_state(state: &str) -> Result<()>`
  - Verify: `devbox run just test-internal test_requirement_invalid_state`
- [ ] Wire `Requirement` subcommand dispatch in `Commands::execute`
  - Verify: `devbox run just build-internal`
- [ ] Add `mod requirement;` to `src/main.rs`
  - Verify: `devbox run just build-internal`
- [ ] Write integration tests in `tests/cli_tests.rs` for requirement create, list, show, supersede
  - Verify: `devbox run just test-internal test_requirement`
- [ ] Run full lint suite
  - Verify: `devbox run just lint-internal`

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `src/cli.rs` | Modify | Add `Requirement` subcommand enum variant and dispatch |
| `src/requirement.rs` | Create | New module with `RequirementManager` for requirement CRUD |
| `src/main.rs` | Modify | Register `requirement` module |
| `tests/cli_tests.rs` | Modify | Add integration tests for requirement commands |
| `src/db.rs` or `src/portfolio_db.rs` | Modify | Add requirement-related query helpers if DB module exists from 01-001 |

## Acceptance Criteria

- [ ] `tkr requirement create "Multi-account Sync" --portfolio=personal --description="..."` creates a requirement and prints the new `req-` ID
- [ ] `tkr requirement create` with a non-existent portfolio ID fails with a clear FK violation error
- [ ] `tkr requirement list` lists all requirements with id, title, state, and portfolio
- [ ] `tkr requirement list --portfolio=personal` filters by portfolio
- [ ] `tkr requirement show <id>` displays full details including description, state, target date, and story count
- [ ] `tkr requirement show <invalid-id>` fails with "Requirement not found"
- [ ] `tkr requirement supersede <id>` transitions state to `superseded` and confirms
- [ ] `tkr requirement supersede <id>` warns if stories are still attached to the requirement
- [ ] Requirement state values are validated: `surfaced`, `proposed`, `planned`, `current`, `superseded`
- [ ] `tkr requirement create --state=invalid` fails with a validation error
- [ ] Requirement IDs follow the `req-<hash>` format
- [ ] All new tests pass: `devbox run just test-internal`
- [ ] Lint passes: `devbox run just lint-internal`

## Examples

### Example 1: Create a requirement

**Input:**
```bash
tkr requirement create "Multi-account Sync" --portfolio=personal --description="Must support multi-account GitHub sync"
```

**Output:**
```
Created requirement "Multi-account Sync" (id: req-a1b2c3) in portfolio "personal"
```

### Example 2: Create with target date and custom state

**Input:**
```bash
tkr requirement create "Offline-first" --portfolio=personal --state=planned --target-date=2026-12-01
```

**Output:**
```
Created requirement "Offline-first" (id: req-d4e5f6) in portfolio "personal"
```

### Example 3: List all requirements

**Input:**
```bash
tkr requirement list
```

**Output:**
```
ID            Title              State      Portfolio  Target Date
req-a1b2c3    Multi-account Sync current    personal   —
req-d4e5f6    Offline-first      planned    personal   2026-12-01
```

### Example 4: Show a requirement

**Input:**
```bash
tkr requirement show req-a1b2c3
```

**Output:**
```
Requirement: req-a1b2c3
Title:       Multi-account Sync
State:       current
Portfolio:   personal
Target Date: —
Stories:     2

Description:
Must support multi-account GitHub sync across lrepo52 and levonk accounts.
```

### Example 5: Supersede a requirement

**Input:**
```bash
tkr requirement supersede req-a1b2c3
```

**Output:**
```
Requirement req-a1b2c3 ("Multi-account Sync") transitioned to superseded
Warning: 2 stories are still attached to this requirement
```

### Example 6: Invalid state fails

**Input:**
```bash
tkr requirement create "Test" --portfolio=personal --state=invalid
```

**Output (stderr):**
```
Error: Invalid requirement state "invalid". Allowed: surfaced, proposed, planned, current, superseded
```

## Test Plan

All tests use `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. The binary name is `tkr`.

### Integration tests to add in `tests/cli_tests.rs`

```rust
#[test]
fn test_requirement_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement")
        .arg("create")
        .arg("Multi-account Sync")
        .arg("--portfolio=personal")
        .arg("--description=Must support multi-account sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created requirement"))
        .stdout(predicate::str::contains("req-"));
}

#[test]
fn test_requirement_create_invalid_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement")
        .arg("create")
        .arg("Test")
        .arg("--portfolio=personal")
        .arg("--state=invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid requirement state"));
}

#[test]
fn test_requirement_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + two requirements
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync"))
        .stdout(predicate::str::contains("Offline-first"));
}

#[test]
fn test_requirement_list_by_portfolio() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create two portfolios with requirements
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("list").arg("--portfolio=personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync"));
}

#[test]
fn test_requirement_show() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("show").arg("req-a1b2c3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync"))
        .stdout(predicate::str::contains("current"));
}

#[test]
fn test_requirement_show_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("show").arg("req-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_requirement_supersede() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("supersede").arg("req-a1b2c3")
        .assert()
        .success()
        .stdout(predicate::str::contains("superseded"));
}

#[test]
fn test_requirement_id_format() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("create").arg("Test").arg("--portfolio=personal")
        .assert()
        .success()
        .get_output()
        .stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    // Extract the ID and verify it starts with "req-"
    assert!(stdout.contains("req-"));
}
```

## Observability

- Each requirement command logs to stderr on error with context (requirement ID, portfolio ID, state transition).
- `tkr requirement list` output is tabular and human-readable; a future `--json` flag will support machine consumption.
- `tkr requirement show` includes a story count so users can see if a requirement has active work.
- Superseding a requirement with attached stories produces a warning on stderr.
- DB operations use `anyhow::Context` to chain errors with SQL context.

## Compliance

- Requirement state values must match the PRD's state vocabulary exactly: `surfaced`, `proposed`, `planned`, `current`, `superseded`. No ad-hoc states.
- The `portfolio_id` FK constraint must be enforced — creating a requirement for a non-existent portfolio must fail.
- Requirement IDs must follow the `req-<hash>` format for consistency.
- Superseded requirements are not deleted; they remain in the DB for historical reference and referential integrity.
- No secrets or credentials in requirement descriptions or logs.
- MIT license — no attribution boilerplate added to new files.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Superseding a requirement with active stories orphans them | Medium | High | Warn on supersede if stories are attached; do not block (user may be intentionally archiving) |
| FK violation on non-existent portfolio not caught gracefully | Low | Medium | Catch SQLite FK error and return user-friendly message |
| Requirement ID collision (unlikely with UUID) | Very Low | Low | Use `uuid::Uuid::new_v4()` for entropy; PK constraint catches collisions |
| DB schema not yet migrated (depends on 01-001) | Medium | High | Story 01-001 must be complete; verify `requirements` table exists |
| State validation too strict (blocks valid workflows) | Low | Medium | Allow any transition to `superseded` (terminal); validate other transitions loosely in v1 |

## Dependencies & Sequencing

- **Depends on**: 02-001 (Portfolio CRUD) — requirements link to portfolios via `portfolio_id`; portfolios must exist.
- **Depends on**: 01-001 (Portfolio DB foundation) — the `requirements` SQLite table must exist.
- **Blocks**: 04-003 (Portfolio views) — by-requirement view needs requirements to exist.
- **Blocks**: 03-003 (Story CRUD) indirectly — stories link to requirements, but 03-003 can proceed in parallel with a nullable FK.
- **Parallel-safe**: Yes — this story only touches `requirements` table and `requirement` CLI; no overlap with app/story/sync stories.

## Definition of Done

- [ ] All sub-tasks completed and checked off
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` succeeds with all new tests passing
- [ ] `devbox run just lint-internal` succeeds with no warnings
- [ ] Requirement state values validated against PRD vocabulary
- [ ] Requirement IDs follow `req-<hash>` format
- [ ] Supersede warns when stories are attached
- [ ] Conventional commit messages used (`feat(requirement): ...`)
- [ ] No secrets or credentials committed
- [ ] PR describes the "why" not just the "what"

## STOP Conditions

- Stop if story 01-001 (DB foundation) or 02-001 (Portfolio CRUD) is not complete — the `requirements` table and portfolio CLI must exist.
- Stop if the DB schema diverges from the PRD Data Model — coordinate with the schema owner.
- Stop if requirement state vocabulary conflicts with the PRD — the 5 states must match exactly.

## Maintenance Notes

- The `RequirementManager` struct should be designed for reuse by the web API (story 05-002) — keep DB access logic in the manager, not in CLI dispatch code.
- The `show` command's story count query should be efficient — use a `COUNT(*)` join, not loading all stories.
- When adding `--json` output support later, ensure the `Requirement` struct derives `Serialize`.
- The supersede warning for attached stories may later become a blocking error if the product direction changes — keep the logic isolated.

## Commit Conventions

Use conventional commits with the `requirement` module scope:

```
feat(requirement): add requirement create/list/show/supersede CLI commands
feat(requirement): validate requirement state transitions against PRD vocabulary
feat(requirement): generate req- prefixed IDs for requirements
test(requirement): add integration tests for requirement CRUD commands
```
