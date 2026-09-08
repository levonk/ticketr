
# Developer Guide: tkr - Rust CLI Ticket Management System

This guide is for developers working on the codebase. For user-facing project overview and install/deploy instructions, see the root [`AGENTS.md`](../../AGENTS.md).

## JIT Index
- Out of Scope: [`internal-docs/oos/`](../../internal-docs/oos/) - What this repo explicitly does NOT do (check before adding features)
- Improvements: [`internal-docs/improvements/INDEX.md`](../../internal-docs/improvements/INDEX.md) - Potential improvements to consider (check before proposing changes to avoid re-proposing already-evaluated improvements)
- Anti-Patterns: [`internal-docs/anti-patterns/INDEX.md`](../../internal-docs/anti-patterns/INDEX.md) - Things explicitly NOT to do (check before implementing changes to avoid re-introducing known-bad approaches)
- Knowledge Bundles: [`.agents/knowledge/bundles/`](../../.agents/knowledge/bundles/) - Offline practice bundles (universal + stack-matched); see root AGENTS.md for the full table including URL-referenced domain bundles

## Setup (Development Environment)

How a contributor stands up a dev environment. This is NOT user-facing install — for that, see `## Install` in the root AGENTS.md.

**Environment Activation (Fresh Shell)**
```bash
# 1. Enter project directory
cd /path/to/tkr

# 2. Bootstrap environment (auto-detects devbox)
just bootstrap

# 3. Verify environment
just doctor
```

**Build Commands (via just — auto-detecting devbox)**
- Build: `just build`
- Test: `just test`
- Lint: `just lint`
- Typecheck: `just typecheck`
- Dev: `just dev`
- Bootstrap: `just bootstrap`
- Doctor: `just doctor`
- Quality: `just quality` (lint + test + typecheck)

**Note**: All `just` targets auto-detect the devbox environment via the `_devbox`
helper. If `DEVBOX_SHELL_ENABLED=1` (inside devbox), implementation runs directly.
If not, the target re-execs via `devbox run -- just <target>_impl`. If devbox is
missing, `just doctor` runs automatically to diagnose the issue. AI agents get
fresh shells — `just build` handles everything, no need for `devbox run --` prefix.

**AI Agent and Automated Systems Workflow**:
```bash
# Primary pattern for AI agents: devbox run + just x-internal
devbox run just build-internal      # Build the project
devbox run just test-internal       # Run all tests
devbox run just test-internal test_name  # Run specific test
devbox run just test-internal -- --nocapture  # Run tests with output
devbox run just debug-internal      # Build debug version

# Skip unimplemented tests
devbox run just test-internal -- -- --skip test_dep_tree_command --skip test_link_unlink_commands
```

**Why AI agents use devbox run + *-internal**:
- AI agents operate in automated contexts where environment consistency is critical
- Direct calls to *-internal targets avoid the extra wrapper overhead
- More efficient for repeated automated operations
- Clearer intent in automated scripts and CI/CD pipelines

## Tech Stack
- **Rust** (edition 2021) - Core language
- **Cargo** - Build system and package manager
- **clap 4.5** - CLI argument parsing with derive macros and env var support
- **tokio 1.0** - Async runtime with full features
- **serde 1.0** + **serde_yaml 0.9** + **serde_json 1.0** - Serialization
- **chrono 0.4** - Date/time handling with UTC and serde support
- **anyhow 1.0** + **thiserror 1.0** - Error handling
- **warp 0.3** - Web framework for HTTP API server
- **ratatui 0.24** - Terminal User Interface framework
- **crossterm 0.27** - Cross-platform terminal manipulation
- **devbox** - Reproducible development environment
- **direnv** - Automatic environment activation
- **just** - Command runner for development tasks

## Devbox Commands (Environment)
- `devbox run -- <command>` - Run single command in devbox environment (rarely needed — `just` handles this)
- `devbox add <package>` - Add package to devbox environment
- `devbox shell` - Enter interactive devbox shell

---
description: Shared devbox missing-package remediation guidance — when a required tool/package is missing, add it to devbox.json and run via `devbox run --` instead of installing on the host. Wired into agent-file-upsert's developer.md template so new repositories inherit the logic
---

### Missing Package Remediation

When a command fails with "command not found" or a required tool/package is
missing on the system, do NOT install it on the host. Add it to `devbox.json`
and run the command through devbox:

1. Check whether the tool is already declared in `devbox.json`. If not, add it:
   ```bash
   devbox add <package>      # preferred: updates devbox.json + devbox.lock
   # or edit devbox.json directly for version-pinned packages
   ```
2. Re-run the command via devbox:
   ```bash
   devbox run -- <command>
   ```
3. Prefer `devbox run --` over installing packages on the host system. This
   keeps the environment reproducible and avoids polluting the host.

**Do NOT** install missing tools via `npm`, `brew`, `apt`, `pip install --user`,
`pipx`, `cargo install`, `go install`, or other host-level package managers
unless the tool is explicitly a host-level prerequisite (e.g. devbox/nix
itself, or a system-level binary that cannot live inside devbox). Add it to
`devbox.json` instead and run via `devbox run --`.

## Workflow

**AI Agent Development Loop (Ticket-Driven)**

This project uses tkr itself for ticket management. The systematic workflow:

```bash
# 0. Assure work is starting from clean foundation
# - Assure typechecks, lint, build, test, and security checks pass
# - Assure the repository is clean, and everything is committed
# - Assure you're on the latest version of the repository
# - Assure you're on a working branch

# 1. Grab the next ticket
./target/release/tkr ready                    # Get next actionable ticket
# OR
./target/release/tkr list --status=open       # See all open tickets

# 2. Start work on the ticket
./target/release/tkr start <ticket-id>         # Mark as in_progress
./target/release/tkr add-note <ticket-id> "Starting work on..."

# 3. Do the work (TDD approach)
# - Add tests FIRST - write failing tests that reproduce the issue or feature
# - Implement the required changes
# - Follow coding standards and best practices
# - Update documentation

# 4. Verify the work
devbox run just test-internal       # Run ALL tests

# 5. Complete the ticket
./target/release/tkr close <ticket-id>         # Mark as completed
# OR
./target/release/tkr ready <ticket-id>         # If ready for review/merge

# 6. Commit the changes

# 7. Loop again - grab the next ticket
./target/release/tkr ready                    # Back to step 1
```

**Ticket Status Flow**
```
open → in_progress → ready → closed
  ↑         ↓           ↓
  └─────── ready ←─────┘
```

- **open**: Ready to start work
- **in_progress**: Currently being worked on
- **ready**: Work complete, ready for review
- **closed**: Fully completed and verified

**Workflow Principles**
1. **Atomic Operations** - Update ticket status immediately when starting/completing work; never leave tickets in ambiguous states
2. **Verification First** - Always run tests before marking a ticket complete; ensure all linting passes; verify acceptance criteria
3. **Documentation Updates** - Update relevant documentation as part of the work; add notes to tickets explaining what was done
4. **Quality Gates** - All tests must pass before completion; code must follow established patterns; dependencies must be properly resolved

## Key Directories
- `src/` - Rust source code
  - `main.rs` - Entry point and application initialization (async, Tokio)
  - `cli.rs` - CLI argument parsing (clap-derived `Cli` struct + `Commands` enum) and command execution
  - `ticket.rs` - Core ticket management logic (`TicketManager` + data structures)
  - `utils.rs` - Path resolution and directory discovery utilities
  - `tui.rs` - Terminal User Interface (ratatui)
  - `web.rs` - Web API and UI server (Warp)
- `tests/` - Integration tests
  - `cli_tests.rs` - Comprehensive CLI integration tests (assert_cmd + predicates)
- `web/` - Web UI assets and templates
- `.tickets/` - Default ticket storage directory
- `scripts/` - Project scripts (pre-commit hooks, etc.)
- `.devbox/` - Devbox environment configuration

## Key Files
**Core Configuration**
- `justfile` - Command runner recipes (auto-detecting targets + `*_impl` implementation targets)
- `devbox.json` - Devbox environment configuration
- `.envrc` - direnv configuration for auto-activation
- `Cargo.toml` - Rust project dependencies and metadata
- `Cargo.lock` - Locked dependency versions
- `flake.nix` - Nix flake for reproducible builds
- `Dockerfile` - Container configuration
- `docker-compose.yml` - Container orchestration

**Documentation**
- `AGENTS.md` - Agent documentation (PRIMARY, progressive-disclosure root)
- `README.md` - Project overview and quick start
- `.agents/knowledge/developer.md` - This file — developer guide

## Patterns

**Code Organization**
- Modular design with clear separation of concerns (cli → ticket → utils)
- Async entry point using Tokio runtime
- Error handling via `anyhow::Result<T>` with `.context()` for error chaining

**Adding New Commands**
1. Update CLI enum in `src/cli.rs`:
```rust
#[derive(Subcommand)]
pub enum Commands {
    NewCommand {
        arg1: String,
        #[arg(long)]
        optional_arg: Option<String>,
    },
}
```
2. Implement command logic in `execute` method
3. Add `TicketManager` method if needed in `src/ticket.rs`
4. Write tests in `tests/cli_tests.rs`

**Testing Patterns**
- ✅ DO use `assert_cmd::Command` for CLI integration tests
- ✅ DO use `tempfile::TempDir` for test isolation
- ✅ DO set `TICKETS_DIR` env var in tests to point to temp directory
- ✅ DO verify file movement after status changes (old file removed, new file created)
- ✅ DO write failing tests first (TDD), then implement
- ❌ DON'T use `Command::cargo_bin("tk")` — use `"tkr"` (the binary name is `tkr`)
- ❌ DON'T skip regression tests for bug fixes

**CLI Testing Pattern**:
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_command_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tkr").unwrap();  // Use "tkr" not "tk"
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("command")
        .arg("argument")
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

**File Status Testing**:
```rust
// After status changes, always verify file location
let new_status_path = tickets_dir.join("new_status").join(format!("{}.md", ticket_id));
let old_status_path = tickets_dir.join("old_status").join(format!("{}.md", ticket_id));

assert!(new_status_path.exists());
assert!(!old_status_path.exists());  // Critical: verify old file removed
```

## Boundaries

### Always
- Run tests before committing: `just test`
- Use `just <command>` for all build/test/lint operations — auto-detects devbox
- Follow TDD: write failing tests first, then implement
- All features must include comprehensive tests
- All bug fixes must include regression tests
- Use `anyhow::Result<T>` for error propagation with `.context()` for chaining
- Activate direnv on first entry: `direnv allow && source .envrc`

### Ask First
- Modifying public CLI API (backward compatibility)
- Changing the ticket file format (YAML frontmatter structure)
- Adding new host-level packages outside `devbox.json`
- Changing the ID generation algorithm
- Modifying the ticket storage directory structure

### Never
- Commit secrets or credentials
- Delete tests
- Use `Command::cargo_bin("tk")` in tests — the binary is `tkr`
- Skip testing for any change
- Make bandaids — fix root causes
- Leave ticket files in duplicate locations after status changes

## Known Gotchas
- **Binary name**: The compiled binary is `tkr` (not `tk`) — use `Command::cargo_bin("tkr")` in tests, not `"tk"`
- **Fresh shell**: AI agents get fresh shells — `just build` auto-detects devbox via the `_devbox` helper, so no need for `devbox run --` prefix
- **Justfile pattern**: Normal targets delegate to `_devbox` helper which auto-detects `DEVBOX_SHELL_ENABLED`; implementation lives in `*_impl` targets (underscore-prefixed, hidden from `just --list`)
- **File movement bug (fixed)**: `tkr close` previously reported success but didn't move ticket files between status directories — `update_status` now removes old files after saving to new location. Regression tests: `test_ticket_file_movement_between_status_directories`, `test_no_duplicate_files_created_during_status_changes`
- **Environment variables**: `TICKETS_DIR` overrides default tickets directory; `REPO_ROOT` specifies repo root for auto-discovery; `TICKET_PROJECT`/`TICKET_CATEGORY` set default tags
- **Path resolution priority**: `--tickets-dir` CLI arg → `TICKETS_DIR` env var → auto-discovery from cwd up to git root → fallback to `.tickets` in cwd
- **Debug mode**: Set `RUST_LOG=debug` for detailed logging output
- **Missing tools go in devbox.json, not on the host**: When a required tool is missing, add it to `devbox.json` and run via `devbox run --`. See Missing Package Remediation above.

## Key Data Structures

**Ticket Structure** (`src/ticket.rs`):
```rust
pub struct Ticket {
    pub id: String,                    // Unique identifier (e.g., "ja-1234")
    pub title: String,
    pub status: String,
    pub deps: Vec<String>,             // Dependency ticket IDs
    pub links: Vec<String>,            // Linked ticket IDs
    pub created: DateTime<Utc>,
    pub issue_type: String,
    pub priority: i32,                 // 1-5
    pub description: Option<String>,
    pub design: Option<String>,
    pub acceptance: Option<String>,
    pub assignee: Option<String>,
    pub external_ref: Option<String>,
    pub parent: Option<String>,
    pub project: Option<String>,       // Mono-repo project tag
    pub category: Option<String>,      // Mono-repo category tag
    pub notes: Option<Vec<Note>>,
}
```

**ID Generation**: `{prefix}-{timestamp_suffix}{uuid_suffix}` where prefix is derived from directory name, timestamp is milliseconds since epoch, and UUID suffix ensures uniqueness.

**File Storage Format**: Markdown with YAML frontmatter:
```markdown
---
id: ja-1234
title: Example Ticket
status: open
deps: []
links: []
created: 2023-01-01T12:00:00Z
type: task
priority: 2
project: backend
category: api
---

# Example Ticket

Ticket description goes here.

## Notes

**2023-01-01 12:30:00**: Initial investigation
```

## Definition of Done
- [ ] Tests pass: `just test`
- [ ] Lint passes: `just lint`
- [ ] Typecheck passes: `just typecheck`
- [ ] No secrets or credentials committed
- [ ] Conventional commit message used
- [ ] PR describes the "why" not just the "what"
- [ ] Rebased on `main` if diverged
- [ ] Affected AGENTS.md files updated per Maintenance Protocol
