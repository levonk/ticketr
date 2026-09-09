---
story_id: "03-001"
story_title: "App CRUD commands + auto-create default app"
story_name: "app-crud"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 3
parallel_id: 1
branch: "feature/current/portfolio-layer/story-03-001-app-crud"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["02-002"]
parallel_safe: true
modules: ["cli", "app", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story adds App-level CRUD commands to the tkr CLI (`tkr app create`, `tkr app list`, `tkr app sunset`) and ensures that a `default` app is automatically created whenever a project is registered. Apps are deployable units within a project (e.g. `api`, `web`, `worker`); single-app projects get an implicit `default` app so that tasks, stories, and requirements always have an app context to attach to. App state follows the deployment vocabulary: `sketched`, `drafted`, `deployed`, `deprecated`, `sunset`.

## Current State

### Relevant files

- `src/cli.rs` — CLI argument definitions and command dispatch (lines 1–265). The `Commands` enum currently has no `App` variant. New subcommands are added as enum variants and dispatched in `Commands::execute`.
- `src/ticket.rs` — `TicketManager` struct and ticket CRUD logic. Not directly modified by this story but provides patterns for ID generation and file I/O.
- `src/main.rs` — Entry point; constructs `TicketManager` and calls `Commands::execute`.
- `Cargo.toml` — Dependencies; `rusqlite` is added in story 01-001.
- `tests/cli_tests.rs` — Integration tests using `assert_cmd`, `predicates`, `tempfile`. New tests follow the same pattern.
- `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md` — PRD defining the `apps` table schema (lines 205–214) and CLI commands (lines 469–471).

### Code excerpts

The `apps` table from the PRD Data Model:

```sql
CREATE TABLE apps (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,             -- "default" for single-app projects
    description TEXT,
    state TEXT DEFAULT 'drafted',   -- sketched, drafted, deployed, deprecated, sunset
    created TEXT NOT NULL,
    UNIQUE(project_id, name)
);
```

CLI commands from the PRD:

```bash
tkr app create "api" --project=<id>
tkr app list --project=<id>
tkr app sunset <id>
```

### Build / test / lint commands

```bash
devbox run just build-internal
devbox run just test-internal
devbox run just lint-internal
```

## Scope

### In scope

- Add `App` subcommand to `src/cli.rs` with `create`, `list`, and `sunset` sub-subcommands.
- Implement `AppManager` (or equivalent DB access layer) for CRUD operations against the `apps` SQLite table.
- Auto-create a `default` app when a project is registered (modify the project registration flow from story 02-002).
- Validate app state transitions: `sketched → drafted → deployed ⇄ deprecated → sunset`.
- `tkr app create` accepts `--project=<id>`, `--description=<text>`, and optional `--state=<state>` (defaults to `drafted`).
- `tkr app list` accepts `--project=<id>` filter and displays app id, name, state, and project.
- `tkr app sunset <id>` transitions an app to the `sunset` terminal state.
- Integration tests for all three commands and the auto-create-default behavior.

### Out of scope

- Web API endpoints for apps (story 05-002).
- Linking tasks to apps via markdown frontmatter (story 04-001).
- App deletion (apps are sunset, not deleted — preserves referential integrity).
- Web UI for app management (story 06-001).
- Bidirectional sync of app state (daemon story 05-001).

## Sub-Tasks

- [ ] Add `App` variant to `Commands` enum in `src/cli.rs` with `Create`, `List`, `Sunset` sub-subcommands
  - Verify: `devbox run just build-internal`
- [ ] Create `src/app.rs` module with `AppManager` struct wrapping a SQLite connection
  - Verify: `devbox run just build-internal`
- [ ] Implement `AppManager::create_app(project_id, name, description, state) -> Result<i64>` inserting into `apps` table
  - Verify: `devbox run just test-internal test_app_create`
- [ ] Implement `AppManager::list_apps(project_id: Option<i64>) -> Result<Vec<App>>` querying the `apps` table
  - Verify: `devbox run just test-internal test_app_list`
- [ ] Implement `AppManager::sunset_app(id) -> Result<()>` updating state to `sunset`
  - Verify: `devbox run just test-internal test_app_sunset`
- [ ] Implement `AppManager::ensure_default_app(project_id) -> Result<()>` that creates a `default` app if none exists for the project
  - Verify: `devbox run just test-internal test_app_auto_create_default`
- [ ] Modify project registration (story 02-002 code) to call `ensure_default_app` after inserting a project row
  - Verify: `devbox run just test-internal test_project_register_creates_default_app`
- [ ] Wire `App` subcommand dispatch in `Commands::execute`
  - Verify: `devbox run just build-internal`
- [ ] Add `mod app;` to `src/main.rs`
  - Verify: `devbox run just build-internal`
- [ ] Write integration tests in `tests/cli_tests.rs` for app create, list, sunset, and auto-create-default
  - Verify: `devbox run just test-internal test_app`
- [ ] Run full lint suite
  - Verify: `devbox run just lint-internal`

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `src/cli.rs` | Modify | Add `App` subcommand enum variant and dispatch |
| `src/app.rs` | Create | New module with `AppManager` for app CRUD |
| `src/main.rs` | Modify | Register `app` module |
| `tests/cli_tests.rs` | Modify | Add integration tests for app commands |
| `src/db.rs` or `src/portfolio_db.rs` | Modify | Add app-related query helpers if DB module exists from 01-001 |

## Acceptance Criteria

- [ ] `tkr app create "api" --project=1 --description="REST API"` creates an app row and prints the new app ID
- [ ] `tkr app create` with a duplicate `(project_id, name)` fails with a clear error message
- [ ] `tkr app list --project=1` lists all apps for project 1 with id, name, state, and description
- [ ] `tkr app list` without `--project` lists all apps across all projects
- [ ] `tkr app sunset <id>` transitions the app state to `sunset` and confirms the change
- [ ] Registering a new project via `tkr project register` automatically creates a `default` app for that project
- [ ] App state values are validated against the allowed set: `sketched`, `drafted`, `deployed`, `deprecated`, `sunset`
- [ ] `tkr app create --state=invalid` fails with a validation error
- [ ] All new tests pass: `devbox run just test-internal`
- [ ] Lint passes: `devbox run just lint-internal`

## Examples

### Example 1: Create an app

**Input:**
```bash
tkr app create "api" --project=1 --description="REST API server"
```

**Output:**
```
Created app "api" (id: 1) for project 1
```

### Example 2: List apps for a project

**Input:**
```bash
tkr app list --project=1
```

**Output:**
```
ID  Name     State     Description
1   default  drafted   Default app
2   api      drafted   REST API server
```

### Example 3: Sunset an app

**Input:**
```bash
tkr app sunset 2
```

**Output:**
```
App 2 ("api") transitioned to sunset
```

### Example 4: Auto-create default app on project registration

**Input:**
```bash
tkr project register /path/to/repo --portfolio=personal
```

**Output:**
```
Registered project "repo" (id: 3) in portfolio "personal"
Created default app (id: 5) for project 3
```

### Example 5: Duplicate app name fails

**Input:**
```bash
tkr app create "api" --project=1
```
(when "api" already exists for project 1)

**Output (stderr):**
```
Error: App "api" already exists for project 1
```

## Test Plan

All tests use `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. The binary name is `tkr`.

### Unit / integration tests to add in `tests/cli_tests.rs`

```rust
#[test]
fn test_app_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + project first (via CLI or direct DB insert)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app")
        .arg("create")
        .arg("api")
        .arg("--project=1")
        .arg("--description=REST API")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created app"));
}

#[test]
fn test_app_create_duplicate_fails() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + project + first "api" app
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1")
        .assert().success();
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_app_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + project + two apps
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("list").arg("--project=1")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("api"));
}

#[test]
fn test_app_sunset() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + project + app
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("sunset").arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("sunset"));
}

#[test]
fn test_app_create_invalid_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1").arg("--state=invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid state"));
}

#[test]
fn test_project_register_creates_default_app() {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir = temp_dir.path().join("myrepo");
    fs::create_dir_all(&repo_dir).unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project").arg("register").arg(&repo_dir).arg("--portfolio=personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"));
    // Verify default app exists
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("list").arg("--project=1")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"));
}
```

## Observability

- Each app command logs to stderr on error with context (project ID, app name, state transition).
- `tkr app list` output is tabular and human-readable; a future `--json` flag will support machine consumption.
- DB operations use `anyhow::Context` to chain errors with SQL context.
- Consider adding `RUST_LOG=debug` logging for SQL statements during app CRUD operations.

## Compliance

- App state values must match the PRD's state vocabulary exactly: `sketched`, `drafted`, `deployed`, `deprecated`, `sunset`. No ad-hoc states.
- The `UNIQUE(project_id, name)` constraint must be enforced at both the DB and application layer.
- Auto-created `default` apps must have `state='drafted'` (the table default).
- No secrets or credentials in app descriptions or logs.
- MIT license — no attribution boilerplate added to new files.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Auto-create default app fails silently during project registration | Medium | High | Wrap in transaction; if app creation fails, roll back project registration and report error |
| Duplicate app name race condition (two concurrent `tkr app create`) | Low | Medium | Rely on SQLite `UNIQUE` constraint; catch constraint violation and return friendly error |
| State transition validation too strict (blocks valid workflows) | Low | Medium | Allow any transition to `sunset` (terminal); validate other transitions loosely in v1 |
| DB schema not yet migrated (depends on 01-001) | Medium | High | Story 01-001 must be complete; verify `apps` table exists before running app commands |

## Dependencies & Sequencing

- **Depends on**: 02-002 (Project registration CLI) — apps link to projects via `project_id`; project registration must exist to auto-create the default app.
- **Depends on**: 01-001 (Portfolio DB foundation) — the `apps` SQLite table must exist.
- **Blocks**: 04-001 (Link tasks to stories) — tasks reference `app_id`.
- **Blocks**: 04-003 (Portfolio views) — views display apps per project.
- **Parallel-safe**: Yes — this story only touches `apps` table and `app` CLI; no overlap with requirement/story/sync stories.

## Definition of Done

- [ ] All sub-tasks completed and checked off
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` succeeds with all new tests passing
- [ ] `devbox run just lint-internal` succeeds with no warnings
- [ ] App state values validated against PRD vocabulary
- [ ] Auto-create default app tested on project registration
- [ ] Conventional commit messages used (`feat(app): ...`)
- [ ] No secrets or credentials committed
- [ ] PR describes the "why" not just the "what"

## STOP Conditions

- Stop if story 01-001 (DB foundation) or 02-002 (project registration) is not complete — the `apps` table and project registration flow must exist.
- Stop if the DB schema diverges from the PRD Data Model — coordinate with the schema owner.
- Stop if auto-create default app would break existing project registration tests — review with the author of 02-002.

## Maintenance Notes

- The `AppManager` struct should be designed for reuse by the web API (story 05-002) — keep DB access logic in the manager, not in CLI dispatch code.
- App state transitions may later need a state machine validator; for now, simple validation suffices.
- The `default` app convention is important: any code that looks up an app by name should handle the `default` case specially in future stories.
- When adding `--json` output support later, ensure the `App` struct derives `Serialize`.

## Commit Conventions

Use conventional commits with the `app` module scope:

```
feat(app): add app create/list/sunset CLI commands
feat(app): auto-create default app on project registration
feat(app): validate app state transitions against PRD vocabulary
test(app): add integration tests for app CRUD commands
```
