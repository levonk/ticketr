---
story_id: "02-002"
story_title: "Project registration CLI commands + link to portfolio"
story_name: "project-registration"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 2
parallel_id: 1
branch: "feature/current/portfolio-layer/story-02-002-project-registration"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["01-001"]
parallel_safe: true
modules: ["cli", "project", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

This story adds the `tkr project` CLI subcommand with three operations — `register`, `list`, and `unregister` — that manage project records in the SQLite database and link them to portfolios. Projects are the second level of the 7-level hierarchy and represent git repositories registered under a portfolio. The `register` command takes a repo path and a `--portfolio` flag, auto-detects the GitHub owner and repo name from `git remote get-url origin`, resolves the `.tickets` directory, and inserts a row into the `projects` table with state `seeded`. The `list` command displays all registered projects with their portfolio, repo path, and GitHub info. The `unregister` command removes a project from the DB (hard delete, since the markdown tickets remain as the durable source of truth). Projects use the lifecycle-themed state vocabulary: `imagined`, `seeded` (default), `live`, `dormant`, and `retired`.

## Current State

### Relevant files

- **`src/cli.rs`** (lines 1-265) — The `Commands` enum has ticket operations, `Web`, `Tui`, `Version`. No `Project` subcommand exists. The `execute()` method dispatches to `TicketManager` methods.
- **`src/main.rs`** (lines 1-37) — Parses CLI, creates `TicketManager`, calls `cli.command.execute(&mut manager).await`. No `PortfolioDb` is instantiated.
- **`src/db.rs`** (created in story 01-001) — Contains `PortfolioDb` struct with `open()`, `init_schema()`, `migrate()`, `db_path()`. No CRUD methods for projects yet.
- **`src/utils.rs`** (lines 1-57) — `find_tickets_dir(repo_root)` walks up from the current directory to find `.tickets/`. `get_repo_root()` walks up to find `.git/`. These can be reused for project registration.
- **`src/ticket.rs`** (lines 44-49) — `TicketManager` has `tickets_dir: PathBuf`, `project: Option<String>`, `category: Option<String>`.
- **`tests/cli_tests.rs`** (lines 1-705) — Integration tests using `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. No project registration tests exist.

### Existing utils (src/utils.rs lines 44-57)

```rust
#[allow(dead_code)]
pub fn get_repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        if !current.pop() {
            anyhow::bail!("Not in a git repository");
        }
    }
}
```

### Project schema (from PRD, created in story 01-001)

```sql
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id),
    name TEXT NOT NULL,
    repo_path TEXT NOT NULL UNIQUE,
    github_owner TEXT,
    github_repo TEXT,
    github_account TEXT,            -- which gh auth to use
    tickets_dir TEXT NOT NULL,
    state TEXT DEFAULT 'seeded',    -- imagined, seeded, live, dormant, retired
    registered_at TEXT NOT NULL,
    last_synced_at TEXT
);
```

### Project state vocabulary (from PRD)

```
Project:  imagined → seeded → live ⇄ dormant → retired
```

| Pre-draft | Draft | Ready | Active | Blocked | Done | Archived |
|-----------|-------|-------|--------|---------|------|----------|
| `imagined` | `seeded` | `live` | `live` | `dormant` | `retired` | — |

### PRD CLI specification (from PRD lines 463-466)

```bash
tkr project register <path> --portfolio=<id>
tkr project list
tkr project unregister <path>
```

### Build / test / lint commands

```bash
devbox run just build-internal    # cargo build --release
devbox run just test-internal      # cargo test
devbox run just lint-internal      # cargo clippy
```

## Scope

### In scope

- Add `Project` subcommand to `Commands` enum in `src/cli.rs` with nested subcommands:
  - `tkr project register <path> --portfolio=<id> [--name=<name>]` — registers a repo under a portfolio
  - `tkr project list [--portfolio=<id>]` — lists registered projects, optionally filtered by portfolio
  - `tkr project unregister <path>` — removes a project from the DB
- Add CRUD methods to `PortfolioDb` in `src/db.rs`:
  - `register_project(portfolio_id, name, repo_path, tickets_dir, github_owner, github_repo) -> Result<i64>`
  - `list_projects(portfolio_id: Option<&str>) -> Result<Vec<Project>>`
  - `unregister_project(repo_path) -> Result<()>`
  - `project_exists(repo_path) -> Result<bool>`
- Add `Project` struct in `src/db.rs` (serializable, with all `projects` table fields)
- Auto-detect GitHub owner and repo from `git remote get-url origin` in the registered repo
- Auto-detect `.tickets` directory in the registered repo (reuse `find_tickets_dir` logic)
- Validate that the portfolio exists before registering a project
- Validate that the repo path exists and is a git repository
- Prevent duplicate registration (unique `repo_path` constraint)
- Integration tests using `assert_cmd` with `TKR_DB_PATH` env var and temp git repos

### Out of scope

- Portfolio CRUD (story 02-001)
- App CRUD and auto-create default app (story 03-001)
- Markdown-to-DB sync (story 03-004)
- Project state transitions (`live`, `dormant`, `retired` — future enhancement)
- GitHub sync (story 06-002)
- Daemon / file watcher (story 05-001)
- Web API endpoints for projects (story 05-002)

## Sub-Tasks

- [ ] Add `Project` struct to `src/db.rs` with all fields from the `projects` table
  - Verify: `devbox run just build-internal` compiles the struct
- [ ] Add `register_project()` method to `PortfolioDb` — inserts a row with state `seeded`, returns the auto-generated ID
  - Verify: unit test inserts a project and confirms the ID is returned
- [ ] Add `list_projects(portfolio_id)` method to `PortfolioDb` — returns all projects, optionally filtered by portfolio
  - Verify: unit test inserts 2 projects in different portfolios, lists all and filtered
- [ ] Add `unregister_project(repo_path)` method to `PortfolioDb` — deletes the row by repo_path
  - Verify: unit test inserts and unregisters, confirms the row is gone
- [ ] Add `project_exists(repo_path)` method to `PortfolioDb` — returns true/false
  - Verify: unit test confirms false before registration and true after
- [ ] Add `detect_github_info(repo_path)` function to `src/db.rs` or `src/utils.rs` — runs `git remote get-url origin` and parses owner/repo
  - Verify: unit test with a mock git repo returns correct owner/repo
- [ ] Add `Project` subcommand variant to `Commands` enum in `src/cli.rs` with nested `ProjectSubcommand` enum
  - Verify: `devbox run just build-internal` compiles with new subcommand
- [ ] Implement `project register` handler — validates portfolio exists, detects GitHub info, resolves tickets dir, calls `register_project()`
  - Verify: `tkr project register /path/to/repo --portfolio=personal` succeeds
- [ ] Implement `project register` with `--name` override — uses provided name instead of directory name
  - Verify: `tkr project register /path/to/repo --portfolio=personal --name=custom-name` succeeds
- [ ] Implement `project register` error handling — non-existent portfolio, non-existent path, non-git repo, duplicate registration
  - Verify: each error case returns a clear error message and exit code 1
- [ ] Implement `project list` handler — calls `list_projects()`, prints table
  - Verify: `tkr project list` shows registered projects
- [ ] Implement `project list --portfolio=<id>` filter
  - Verify: `tkr project list --portfolio=personal` shows only projects in that portfolio
- [ ] Implement `project unregister` handler — calls `unregister_project()`
  - Verify: `tkr project unregister /path/to/repo` succeeds and project disappears from `list`
- [ ] Write integration tests in `tests/cli_tests.rs` using `assert_cmd` with temp git repos
  - Verify: `devbox run just test-internal` passes all new tests
- [ ] Run clippy
  - Verify: `devbox run just lint-internal` passes with no new warnings

## Relevant Files

| File | Action | Description |
|------|--------|-------------|
| `src/db.rs` | Edit | Add `Project` struct, `register_project()`, `list_projects()`, `unregister_project()`, `project_exists()` methods |
| `src/cli.rs` | Edit | Add `Project` variant to `Commands` enum with nested `ProjectSubcommand` enum; implement handlers |
| `src/utils.rs` | Edit | Add `detect_github_info(repo_path)` function for git remote parsing |
| `src/main.rs` | Edit | Pass DB path or `PortfolioDb` instance to command execution (if needed) |
| `tests/cli_tests.rs` | Edit | Add integration tests for `project register/list/unregister` |

## Acceptance Criteria

- [ ] `tkr project register <path> --portfolio=<id>` registers a project with state `seeded`
- [ ] `tkr project register` auto-detects the project name from the directory name if `--name` is not provided
- [ ] `tkr project register` auto-detects `github_owner` and `github_repo` from `git remote get-url origin`
- [ ] `tkr project register` auto-detects the `.tickets` directory in the repo
- [ ] `tkr project register` with a non-existent portfolio ID returns an error
- [ ] `tkr project register` with a non-existent path returns an error
- [ ] `tkr project register` with a path that is not a git repo returns an error (or proceeds without GitHub info)
- [ ] `tkr project register` with an already-registered repo path returns an error
- [ ] `tkr project list` displays all registered projects with portfolio, name, repo path, GitHub info, and state
- [ ] `tkr project list --portfolio=<id>` filters projects by portfolio
- [ ] `tkr project unregister <path>` removes the project from the DB
- [ ] `tkr project unregister` with a non-existent path returns an error
- [ ] The DB is auto-created and migrated if it doesn't exist when any `project` command runs
- [ ] `TKR_DB_PATH` env var overrides the default DB path (for testing)
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` passes all tests (existing + new)
- [ ] `devbox run just lint-internal` passes with no new warnings

## Examples

### Example 1: Register a project

**Input:**
```bash
tkr project register /home/user/projects/tkr --portfolio=personal
```

**Output:**
```
Registered project: tkr
  Portfolio: personal
  Repo: /home/user/projects/tkr
  GitHub: levonk/tkr
  Tickets: /home/user/projects/tkr/.tickets
  State: seeded
```

### Example 2: Register a project with custom name

**Input:**
```bash
tkr project register /home/user/projects/infrahub --portfolio=business-a --name=infrahub-platform
```

**Output:**
```
Registered project: infrahub-platform
  Portfolio: business-a
  Repo: /home/user/projects/infrahub
  GitHub: levonk/infrahub
  Tickets: /home/user/projects/infrahub/.tickets
  State: seeded
```

### Example 3: Register a non-git project (no GitHub info)

**Input:**
```bash
tkr project register /home/user/projects/local-only --portfolio=personal
```

**Output:**
```
Registered project: local-only
  Portfolio: personal
  Repo: /home/user/projects/local-only
  GitHub: (none)
  Tickets: /home/user/projects/local-only/.tickets
  State: seeded
```

### Example 4: List all projects

**Input:**
```bash
tkr project list
```

**Output:**
```
ID  Name              Portfolio     Repo                          GitHub          State
1   tkr               personal      /home/user/projects/tkr       levonk/tkr      seeded
2   infrahub-platform business-a    /home/user/projects/infrahub  levonk/infrahub  seeded
3   dotfiles          personal      /home/user/projects/dotfiles   levonk/dotfiles  seeded
```

### Example 5: List projects filtered by portfolio

**Input:**
```bash
tkr project list --portfolio=personal
```

**Output:**
```
ID  Name      Portfolio    Repo                        GitHub          State
1   tkr       personal     /home/user/projects/tkr     levonk/tkr      seeded
3   dotfiles  personal     /home/user/projects/dotfiles levonk/dotfiles seeded
```

### Example 6: Unregister a project

**Input:**
```bash
tkr project unregister /home/user/projects/tkr
```

**Output:**
```
Unregistered project: /home/user/projects/tkr
```

### Example 7: Register with non-existent portfolio

**Input:**
```bash
tkr project register /home/user/projects/tkr --portfolio=nonexistent
```

**Output:**
```
Error: Portfolio not found: nonexistent
```
Exit code: 1

### Example 8: Duplicate registration

**Input:**
```bash
tkr project register /home/user/projects/tkr --portfolio=personal
```
(when already registered)

**Output:**
```
Error: Project already registered: /home/user/projects/tkr
```
Exit code: 1

### Example 9: Unregister non-existent project

**Input:**
```bash
tkr project unregister /home/user/projects/nonexistent
```

**Output:**
```
Error: Project not found: /home/user/projects/nonexistent
```
Exit code: 1

## Test Plan

All integration tests use `assert_cmd::Command`, `predicates::prelude::*`, and `tempfile::TempDir`. Tests create temporary git repos with `git init` and `git remote add origin` to simulate real repos. Tests set `TKR_DB_PATH` to a temp directory path for DB isolation.

### Integration tests (tests/cli_tests.rs)

```rust
use std::process::Command as StdCommand;

/// Helper: create a temp git repo with a remote
fn create_temp_git_repo() -> TempDir {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    StdCommand::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("https://github.com/levonk/test-repo.git")
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Create .tickets dir
    std::fs::create_dir_all(repo_path.join(".tickets")).unwrap();

    temp
}

/// Helper: create a portfolio in the DB
fn create_portfolio(db_path: &std::path::Path, id: &str, name: &str) {
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", db_path)
        .arg("portfolio")
        .arg("create")
        .arg(id)
        .arg(name)
        .assert()
        .success();
}

#[test]
fn test_project_register() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("Registered project"))
        .stdout(predicate::str::contains("seeded"))
        .stdout(predicate::str::contains("levonk/test-repo"));
}

#[test]
fn test_project_register_with_custom_name() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .arg("--name")
        .arg("custom-project-name")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-project-name"));
}

#[test]
fn test_project_register_nonexistent_portfolio() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_project_register_nonexistent_path() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg("/nonexistent/path/to/repo")
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("does not exist")));
}

#[test]
fn test_project_register_duplicate() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    // First registration
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // Duplicate registration
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already registered"));
}

#[test]
fn test_project_list() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    // Register a project
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // List
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("seeded"));
}

#[test]
fn test_project_list_filtered_by_portfolio() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    // Create two portfolios
    create_portfolio(&db_path, "personal", "Personal");
    create_portfolio(&db_path, "business", "Business");

    // Register a project in each
    let repo1 = create_temp_git_repo();
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo1.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    let repo2 = create_temp_git_repo();
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo2.path())
        .arg("--portfolio")
        .arg("business")
        .assert()
        .success();

    // List filtered to personal
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("list")
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"));
    // Should not contain business projects
    // (Note: exact assertion depends on output format)
}

#[test]
fn test_project_unregister() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path().to_str().unwrap().to_string();

    create_portfolio(&db_path, "personal", "Personal");

    // Register
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(&repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // Unregister
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("unregister")
        .arg(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unregistered"));

    // List should be empty
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No projects").or(predicate::str::contains("0 projects")));
}

#[test]
fn test_project_unregister_not_found() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("unregister")
        .arg("/nonexistent/path")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_project_auto_creates_db() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();

    // Create portfolio first (this creates the DB)
    create_portfolio(&db_path, "personal", "Personal");

    assert!(db_path.exists());

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_temp.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();
}

#[test]
fn test_project_register_non_git_repo() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    // Create a plain directory (not a git repo)
    let repo_temp = TempDir::new().unwrap();
    std::fs::create_dir_all(repo_temp.path().join(".tickets")).unwrap();

    create_portfolio(&db_path, "personal", "Personal");

    // Should still register, but without GitHub info
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_temp.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("(none)").or(predicate::str::contains("Registered")));
}
```

## Observability

- Each `project` command prints a human-readable confirmation to stdout with key details (name, portfolio, repo path, GitHub info, state)
- Errors are printed to stderr with a clear message and exit code 1
- The DB path is logged on first creation
- GitHub detection failures are logged as warnings, not errors (non-git repos are valid)
- Future: project registration should trigger an initial sync (story 03-004)

## Compliance

- MIT license — no new external dependencies
- No PII stored — project records contain repo paths and GitHub owner/repo (public info)
- `unregister` is a hard delete — the markdown tickets remain as the durable source of truth (no-corner principle)
- GitHub info is detected from the local git remote — no GitHub API calls in this story
- No network calls in this story

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| `git remote get-url` fails in non-git directories | Medium | Low | Catch the error and register without GitHub info; log a warning |
| Git remote URL format varies (SSH vs HTTPS vs git@) | Medium | Medium | Parse multiple formats: `https://github.com/owner/repo.git`, `git@github.com:owner/repo.git`, `ssh://git@github.com/owner/repo.git` |
| Path resolution differs across platforms | Low | Medium | Use `std::path::Path` for cross-platform path handling; canonicalize paths before storing |
| Concurrent registration of the same repo | Low | Low | SQLite UNIQUE constraint on `repo_path` prevents duplicates at the DB level |
| Temp git repos in tests require `git` binary | Low | Low | Tests that need git are marked as integration tests; skip if `git` is not available |
| Repo path stored as absolute vs relative | Medium | Medium | Canonicalize to absolute path before storing and before lookup |

## Dependencies & Sequencing

- **Dependencies**: 01-001 (Portfolio DB foundation) — requires `PortfolioDb`, `open()`, `migrate()`, and the `projects` table
- **Dependants**: 03-001 (App CRUD — apps link to projects), 03-004 (Markdown sync — syncs tickets from registered projects), 04-003 (Portfolio views — shows projects under portfolios)
- **Parallel with**: 02-001 (Portfolio CRUD) — both depend on 01-001 and can be developed in parallel
- **External dependencies**: `git` binary (for `git remote get-url origin` detection) — already a prerequisite for using tkr
- **Existing code reused**: `src/utils.rs::find_tickets_dir()` for `.tickets` directory detection

## Definition of Done

- [ ] `tkr project register <path> --portfolio=<id>` works and creates a project with state `seeded`
- [ ] `tkr project register` auto-detects GitHub owner/repo from `git remote get-url origin`
- [ ] `tkr project register` auto-detects `.tickets` directory
- [ ] `tkr project register --name=<name>` overrides the auto-detected name
- [ ] `tkr project register` validates portfolio existence
- [ ] `tkr project register` prevents duplicate registration
- [ ] `tkr project list` displays all registered projects
- [ ] `tkr project list --portfolio=<id>` filters by portfolio
- [ ] `tkr project unregister <path>` removes the project
- [ ] Non-existent portfolio/path returns a clear error
- [ ] DB auto-creates and migrates on first use
- [ ] `TKR_DB_PATH` env var works for test isolation
- [ ] `devbox run just build-internal` succeeds
- [ ] `devbox run just test-internal` passes all tests (existing + new)
- [ ] `devbox run just lint-internal` passes with no new warnings
- [ ] Code reviewed and merged to the story branch

## STOP Conditions

- Stop and escalate if story 01-001 is not complete (the `PortfolioDb` module and `projects` table must exist)
- Stop and escalate if story 02-001 is not complete (portfolio creation is needed for project registration validation — though a workaround is to create portfolios directly in the DB for testing)
- Stop if `git` binary is not available in the test environment — tests requiring git should be conditionally skipped
- Stop and escalate if existing ticket tests break after adding the `Project` subcommand (indicates a CLI parsing conflict)
- Stop if git remote URL parsing cannot handle the formats used in the project (SSH, HTTPS) — escalate to determine supported formats

## Maintenance Notes

- The `Project` subcommand uses clap's nested subcommand pattern — future project-related commands (e.g. `tkr project sync`, `tkr project state`) will be added to the same `ProjectSubcommand` enum
- The `unregister` operation is a hard delete — the row is removed from the DB. This is intentional because the markdown tickets remain as the durable source of truth. If historical project data is needed, a `retired` state should be used instead
- The `detect_github_info()` function should be kept in `src/utils.rs` (or `src/db.rs`) and reused by the GitHub sync story (06-002)
- Repo paths are stored as absolute, canonicalized paths — the `unregister` command must canonicalize the input path before lookup
- The `github_account` field is left as `None` in this story — it will be populated by the GitHub sync story (06-002) when multi-account support is added
- The `Project` struct should be kept in sync with the `projects` table schema

## Commit Conventions

Use conventional commit messages with the `project`, `cli`, or `db` module scope:

```
feat(db): add Project struct and CRUD methods to PortfolioDb
feat(utils): add detect_github_info for git remote URL parsing
feat(cli): add project subcommand with register, list, unregister
feat(cli): validate portfolio existence before project registration
feat(cli): auto-detect GitHub owner/repo and tickets dir on registration
test(cli): add integration tests for project registration commands
```
