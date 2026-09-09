---
story_id: "03-003"
story_title: "Story CRUD commands (create, list, show, ship)"
story_name: "story-crud"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 3
parallel_id: 1
branch: "feature/current/portfolio-layer/story-03-003-story-crud"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["02-001"]
parallel_safe: true
modules: ["cli", "story", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story adds Story-level CRUD commands to the tkr CLI (`tkr story create`, `tkr story list`, `tkr story show`, `tkr story ship`). Stories are feature slices at the coordination level of the hierarchy — they link a requirement (strategic) to an app (deployable unit) and group tasks (operational). Story state follows the delivery vocabulary: `suggested`, `pitched`, `queued`, `building`, `stalled`, `review`, `shipped`, `archived`. Shipping a story transitions it to the `shipped` terminal state.

## Current State

### Relevant files

- `src/cli.rs` — CLI argument definitions and command dispatch (lines 1–265). The `Commands` enum currently has no `Story` variant.
- `src/ticket.rs` — `TicketManager` struct; provides patterns for ID generation using timestamps and UUIDs (lines 108–140).
- `src/main.rs` — Entry point; module registration and `TicketManager` construction.
- `Cargo.toml` — Dependencies; `rusqlite` added in story 01-001, `uuid` already present.
- `tests/cli_tests.rs` — Integration tests using `assert_cmd`, `predicates`, `tempfile`.
- `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md` — PRD defining the `stories` table schema (lines 229–239) and CLI commands (lines 480–483).

### Code excerpts

The `stories` table from the PRD Data Model:

```sql
CREATE TABLE stories (
    id TEXT PRIMARY KEY,             -- e.g. "story-abc123"
    requirement_id TEXT REFERENCES requirements(id),
    app_id INTEGER REFERENCES apps(id),
    title TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'pitched',    -- suggested, pitched, queued, building, stalled, review, shipped, archived
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0
);
```

CLI commands from the PRD:

```bash
tkr story create "Title" --requirement=<id> --app=<id> --description="..."
tkr story list
tkr story show <id>
tkr story ship <id>
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

- Add `Story` subcommand to `src/cli.rs` with `Create`, `List`, `Show`, and `Ship` sub-subcommands.
- Create `src/story.rs` module with `StoryManager` for CRUD operations against the `stories` SQLite table.
- Generate story IDs with `story-` prefix (e.g. `story-abc123`).
- `tkr story create` accepts `--requirement=<id>`, `--app=<id>`, `--description=<text>`, `--target-date=<YYYY-MM-DD>`, and optional `--state=<state>` (defaults to `pitched`).
- `tkr story create` with `--requirement` and `--app` both optional (a story can be created without links for triage).
- `tkr story list` optionally accepts `--requirement=<id>`, `--app=<id>`, and `--state=<state>` filters.
- `tkr story show <id>` displays full story details including description, state, requirement, app, and linked task count.
- `tkr story ship <id>` transitions state to `shipped` (terminal) and confirms.
- Validate story state values: `suggested`, `pitched`, `queued`, `building`, `stalled`, `review`, `shipped`, `archived`.
- Integration tests for all four commands.

### Out of scope

- Web API endpoints for stories (story 05-002).
- Portfolio views by-story (story 04-003).
- Story deletion (stories are shipped or archived, not deleted — preserves referential integrity for tasks).
- Automatic state transitions (e.g. `queued → building` when first task starts) — manual for now.
- Linking tasks to stories via markdown frontmatter (story 04-001).
- Web UI for story management (story 06-001).
- Archiving stories (the `archived` state is valid but no `tkr story archive` command in this story — future enhancement).

## Sub-Tasks

- [ ] Add `Story` variant to `Commands` enum in `src/cli.rs` with `Create`, `List`, `Show`, `Ship` sub-subcommands
  - Verify: `devbox run just build-internal`
- [ ] Create `src/story.rs` module with `StoryManager` struct wrapping a SQLite connection
  - Verify: `devbox run just build-internal`
- [ ] Implement `StoryManager::generate_id() -> Result<String>` producing `story-<hash>` IDs
  - Verify: `devbox run just test-internal test_story_id_format`
- [ ] Implement `StoryManager::create_story(requirement_id, app_id, title, description, state, target_date) -> Result<String>`
  - Verify: `devbox run just test-internal test_story_create`
- [ ] Implement `StoryManager::list_stories(requirement_id, app_id, state) -> Result<Vec<Story>>` with optional filters
  - Verify: `devbox run just test-internal test_story_list`
- [ ] Implement `StoryManager::show_story(id) -> Result<Story>` with linked task count
  - Verify: `devbox run just test-internal test_story_show`
- [ ] Implement `StoryManager::ship_story(id) -> Result<()>` updating state to `shipped`
  - Verify: `devbox run just test-internal test_story_ship`
- [ ] Add state validation helper: `validate_story_state(state: &str) -> Result<()>`
  - Verify: `devbox run just test-internal test_story_invalid_state`
- [ ] Wire `Story` subcommand dispatch in `Commands::execute`
  - Verify: `devbox run just build-internal`
- [ ] Add `mod story;` to `src/main.rs`
  - Verify: `devbox run just build-internal`
- [ ] Write integration tests in `tests/cli_tests.rs` for story create, list, show, ship
  - Verify: `devbox run just test-internal test_story`
- [ ] Run full lint suite
  - Verify: `devbox run just lint-internal`

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `src/cli.rs` | Modify | Add `Story` subcommand enum variant and dispatch |
| `src/story.rs` | Create | New module with `StoryManager` for story CRUD |
| `src/main.rs` | Modify | Register `story` module |
| `tests/cli_tests.rs` | Modify | Add integration tests for story commands |
| `src/db.rs` or `src/portfolio_db.rs` | Modify | Add story-related query helpers if DB module exists from 01-001 |

## Acceptance Criteria

- [ ] `tkr story create "Portfolio Layer" --requirement=req-abc --app=1 --description="..."` creates a story and prints the new `story-` ID
- [ ] `tkr story create` without `--requirement` or `--app` succeeds (story created for triage)
- [ ] `tkr story create` with a non-existent `--requirement` fails with a clear FK violation error
- [ ] `tkr story create` with a non-existent `--app` fails with a clear FK violation error
- [ ] `tkr story list` lists all stories with id, title, state, requirement, and app
- [ ] `tkr story list --requirement=req-abc` filters by requirement
- [ ] `tkr story list --app=1` filters by app
- [ ] `tkr story list --state=building` filters by state
- [ ] `tkr story show <id>` displays full details including description, state, requirement, app, and task count
- [ ] `tkr story show <invalid-id>` fails with "Story not found"
- [ ] `tkr story ship <id>` transitions state to `shipped` and confirms
- [ ] Story state values are validated: `suggested`, `pitched`, `queued`, `building`, `stalled`, `review`, `shipped`, `archived`
- [ ] `tkr story create --state=invalid` fails with a validation error
- [ ] Story IDs follow the `story-<hash>` format
- [ ] All new tests pass: `devbox run just test-internal`
- [ ] Lint passes: `devbox run just lint-internal`

## Examples

### Example 1: Create a story with full links

**Input:**
```bash
tkr story create "Portfolio Layer" --requirement=req-a1b2c3 --app=2 --description="Cross-project portfolio management"
```

**Output:**
```
Created story "Portfolio Layer" (id: story-x7y8z9) for requirement req-a1b2c3, app 2
```

### Example 2: Create a story for triage (no links)

**Input:**
```bash
tkr story create "Investigate WASM support" --description="Explore WebAssembly compilation"
```

**Output:**
```
Created story "Investigate WASM support" (id: story-a1b2c3)
```

### Example 3: List all stories

**Input:**
```bash
tkr story list
```

**Output:**
```
ID            Title             State     Requirement  App
story-x7y8z9  Portfolio Layer   building  req-a1b2c3   2
story-a1b2c3  Investigate WASM  pitched   —            —
```

### Example 4: List stories filtered by requirement

**Input:**
```bash
tkr story list --requirement=req-a1b2c3
```

**Output:**
```
ID            Title             State     App
story-x7y8z9  Portfolio Layer   building  2
```

### Example 5: Show a story

**Input:**
```bash
tkr story show story-x7y8z9
```

**Output:**
```
Story:      story-x7y8z9
Title:      Portfolio Layer
State:      building
Requirement: req-a1b2c3 (Multi-account Sync)
App:        2 (api)
Target Date: —
Tasks:      5

Description:
Cross-project portfolio management with Kanban board and multi-account sync.
```

### Example 6: Ship a story

**Input:**
```bash
tkr story ship story-x7y8z9
```

**Output:**
```
Story story-x7y8z9 ("Portfolio Layer") transitioned to shipped
```

### Example 7: Invalid state fails

**Input:**
```bash
tkr story create "Test" --state=invalid
```

**Output (stderr):**
```
Error: Invalid story state "invalid". Allowed: suggested, pitched, queued, building, stalled, review, shipped, archived
```

## Test Plan

All tests use `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. The binary name is `tkr`.

### Integration tests to add in `tests/cli_tests.rs`

```rust
#[test]
fn test_story_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement + app
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Portfolio Layer")
        .arg("--requirement=req-a1b2c3")
        .arg("--app=2")
        .arg("--description=Cross-project portfolio management")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created story"))
        .stdout(predicate::str::contains("story-"));
}

#[test]
fn test_story_create_no_links() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Investigate WASM")
        .arg("--description=Explore WebAssembly")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created story"));
}

#[test]
fn test_story_create_invalid_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Test")
        .arg("--state=invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid story state"));
}

#[test]
fn test_story_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement + app + two stories
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer"))
        .stdout(predicate::str::contains("Investigate WASM"));
}

#[test]
fn test_story_list_by_requirement() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement + stories
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("list").arg("--requirement=req-a1b2c3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer"));
}

#[test]
fn test_story_list_by_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create stories with different states
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("list").arg("--state=building")
        .assert()
        .success()
        .stdout(predicate::str::contains("building"));
}

#[test]
fn test_story_show() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement + app + story
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("show").arg("story-x7y8z9")
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer"))
        .stdout(predicate::str::contains("building"));
}

#[test]
fn test_story_show_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("show").arg("story-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_story_ship() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    // Setup: create portfolio + requirement + app + story
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("ship").arg("story-x7y8z9")
        .assert()
        .success()
        .stdout(predicate::str::contains("shipped"));
}

#[test]
fn test_story_id_format() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd.env("TKR_DB_PATH", &db_path)
        .arg("story").arg("create").arg("Test")
        .assert()
        .success()
        .get_output()
        .stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("story-"));
}
```

## Observability

- Each story command logs to stderr on error with context (story ID, requirement ID, app ID, state transition).
- `tkr story list` output is tabular and human-readable; a future `--json` flag will support machine consumption.
- `tkr story show` includes a task count so users can see how much work is attached to a story.
- Shipping a story is a terminal transition — log it clearly for audit purposes.
- DB operations use `anyhow::Context` to chain errors with SQL context.

## Compliance

- Story state values must match the PRD's state vocabulary exactly: `suggested`, `pitched`, `queued`, `building`, `stalled`, `review`, `shipped`, `archived`. No ad-hoc states.
- The `requirement_id` and `app_id` FK constraints must be enforced — creating a story with non-existent links must fail.
- Story IDs must follow the `story-<hash>` format for consistency.
- Shipped stories are not deleted; they remain in the DB for historical reference and referential integrity.
- No secrets or credentials in story descriptions or logs.
- MIT license — no attribution boilerplate added to new files.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Shipping a story with open tasks orphans them | Medium | Medium | Warn on ship if tasks are still open; do not block (user may be intentionally shipping) |
| FK violation on non-existent requirement/app not caught gracefully | Low | Medium | Catch SQLite FK error and return user-friendly message |
| Story ID collision (unlikely with UUID) | Very Low | Low | Use `uuid::Uuid::new_v4()` for entropy; PK constraint catches collisions |
| DB schema not yet migrated (depends on 01-001) | Medium | High | Story 01-001 must be complete; verify `stories` table exists |
| Story created without requirement/app causes issues in views | Low | Low | Allow nullable FKs; views handle `—` display for unlinked stories |

## Dependencies & Sequencing

- **Depends on**: 02-001 (Portfolio CRUD) — stories link to requirements which link to portfolios; portfolios must exist. Also indirectly depends on 03-001 (App CRUD) and 03-002 (Requirement CRUD) for FK targets, but nullable FKs allow stories to be created without links.
- **Depends on**: 01-001 (Portfolio DB foundation) — the `stories` SQLite table must exist.
- **Blocks**: 04-001 (Link tasks to stories) — tasks reference `story_id`.
- **Blocks**: 04-003 (Portfolio views) — by-story view needs stories to exist.
- **Parallel-safe**: Yes — this story only touches `stories` table and `story` CLI; no overlap with app/requirement/sync stories.

## Definition of Done

- [ ] All sub-tasks completed and checked off
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` succeeds with all new tests passing
- [ ] `devbox run just lint-internal` succeeds with no warnings
- [ ] Story state values validated against PRD vocabulary
- [ ] Story IDs follow `story-<hash>` format
- [ ] Ship command transitions to `shipped` terminal state
- [ ] Conventional commit messages used (`feat(story): ...`)
- [ ] No secrets or credentials committed
- [ ] PR describes the "why" not just the "what"

## STOP Conditions

- Stop if story 01-001 (DB foundation) or 02-001 (Portfolio CRUD) is not complete — the `stories` table and portfolio CLI must exist.
- Stop if the DB schema diverges from the PRD Data Model — coordinate with the schema owner.
- Stop if story state vocabulary conflicts with the PRD — the 8 states must match exactly.

## Maintenance Notes

- The `StoryManager` struct should be designed for reuse by the web API (story 05-002) — keep DB access logic in the manager, not in CLI dispatch code.
- The `show` command's task count query should be efficient — use a `COUNT(*)` join, not loading all tasks.
- When adding `--json` output support later, ensure the `Story` struct derives `Serialize`.
- The `ship` command may later need to check that all tasks are `closed` before allowing ship — keep the validation logic isolated for future enhancement.
- The `archived` state is valid but has no CLI command in this story; a future `tkr story archive` command may be added.

## Commit Conventions

Use conventional commits with the `story` module scope:

```
feat(story): add story create/list/show/ship CLI commands
feat(story): validate story state transitions against PRD vocabulary
feat(story): generate story- prefixed IDs for stories
test(story): add integration tests for story CRUD commands
```
