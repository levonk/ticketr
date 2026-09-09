---
story_id: "06-002"
story_title: "GitHub Issues bidirectional sync (multi-account, tag to label)"
story_name: "github-sync"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 6
parallel_id: 2
branch: "feature/current/portfolio-layer/story-06-002-github-sync"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["05-001"]
parallel_safe: true
modules: ["sync", "github"]
priority: "MUST"
risk_level: "high"
tags: ["feat", "backend", "sync", "github"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Implement bidirectional sync between tkr tasks and GitHub Issues, working across multiple GitHub accounts (lrepo52, levonk). The sync runs inside the daemon process (from Story 05-001) and can be triggered via `tkr sync --github` or the `POST /api/sync/github` API endpoint. tkr tasks are pushed to GitHub Issues with tag-to-label mapping and story-to-custom-field mapping. GitHub Issues are pulled back into tkr markdown files. Conflict resolution uses last-write-wins by timestamp, with warnings logged. The SQLite `tasks` table tracks `github_issue_number`, `github_issue_url`, `github_synced_at`, and `github_etag` for each synced task.

## Current State

- `src/cli.rs` (lines 1-265) has no `Sync` subcommand.
- `src/web.rs` has no sync functionality (sync endpoints are defined in Story 05-002 but the logic is this story).
- `src/ticket.rs` (lines 7-36) `Ticket` struct has no GitHub-related fields.
- `Cargo.toml` (lines 14-34) does not include `reqwest` or any HTTP client dependency.
- The PRD (lines 309-310) specifies the daemon includes "GitHub sync (bidirectional, `reqwest` or `gh` CLI)".
- The PRD (lines 319-323) specifies conflict resolution: compare local hash vs markdown, compare GitHub `updated_at` vs stored, last-write-wins by timestamp with warning logged.
- The SQLite `tasks` table schema (PRD lines 242-258) includes: `github_issue_number`, `github_issue_url`, `github_synced_at`, `github_etag`.
- The `projects` table (PRD lines 191-203) includes: `github_owner`, `github_repo`, `github_account` (which `gh auth` to use).
- No `src/sync.rs` or `src/github.rs` module exists.

## Scope

### In Scope

- Add `reqwest = { version = "0.12", features = ["json"] }` to `Cargo.toml` (or use `gh` CLI via `std::process::Command` — evaluate both approaches).
- Create `src/github_sync.rs` module with bidirectional sync logic.
- **Push (tkr → GitHub)**:
  - For each registered project with `github_owner` and `github_repo` set:
  - For each task in the project that has changed (markdown hash differs from stored hash):
  - Create or update a GitHub Issue: title from task title, body from task description, labels from task tags.
  - Set GitHub custom fields: story, requirement, priority (if GitHub Projects custom fields are configured).
  - Store `github_issue_number`, `github_issue_url`, `github_synced_at` in SQLite.
- **Pull (GitHub → tkr)**:
  - For each registered project with GitHub info:
  - List GitHub Issues (using `If-None-Match` / `ETag` for efficient polling).
  - For each issue: find matching tkr task by `github_issue_number` or title.
  - If issue updated since `github_synced_at`: update the tkr markdown file (title, description, state mapping, labels → tags).
  - Update `github_etag` and `github_synced_at` in SQLite.
- **Multi-account support**:
  - Use `gh auth switch` to switch between GitHub accounts (lrepo52, levonk) per project.
  - Each project's `github_account` field determines which auth context to use.
  - Alternatively, use `reqwest` with per-account tokens from `gh auth token`.
- **Tag ↔ Label mapping**: tkr tags map to GitHub labels (1:1). New tags create GitHub labels if they don't exist. GitHub labels map back to tkr tags.
- **State mapping**: tkr task states map to GitHub issue states:
  - `closed` → GitHub issue `closed`
  - All other tkr states (`logged`, `open`, `in_progress`, `blocked`, `ready`) → GitHub issue `open`
- **Conflict resolution**: last-write-wins by timestamp. If both tkr markdown and GitHub Issue changed since last sync, compare timestamps and take the later one. Log a warning for the conflict.
- **CLI commands**: `tkr sync --github` (trigger sync), `tkr sync --status` (show sync status).
- **Integration with daemon**: the sync runs periodically (every 5 minutes) inside the daemon process, or on-demand via CLI/API trigger.
- Track sync state in the `sync_state` SQLite table (key-value: `last_github_sync`, `github_sync_errors`, etc.).

### Out of Scope

- GitHub Projects (Projects v2) board sync — this story syncs Issues only, not Projects custom field views.
- GitHub Milestones mapping.
- GitHub Pull Request integration.
- Web UI for sync configuration (Story 06-001 covers the Kanban board only).
- The API endpoints `POST /api/sync/github` and `GET /api/sync/status` (defined in Story 05-002 — this story provides the underlying logic).
- Daemon process management (Story 05-001 — this story's sync runs inside the daemon).

## Sub-Tasks

1. **Add `reqwest` dependency** — Add `reqwest = { version = "0.12", features = ["json"] }` to `Cargo.toml`. Alternatively, evaluate using `gh` CLI via `std::process::Command` (simpler, no HTTP client needed, but requires `gh` installed).
2. **Create `src/github_sync.rs` module** — Define `GitHubSync` struct with `push()`, `pull()`, `sync()` (bidirectional), and `status()` methods. Takes a SQLite connection and project list.
3. **Implement GitHub API client** — Use `reqwest` to call GitHub REST API v3 (or `gh api` via subprocess). Endpoints needed: `GET /repos/:owner/:repo/issues`, `POST /repos/:owner/:repo/issues`, `PATCH /repos/:owner/:repo/issues/:number`, `GET /repos/:owner/:repo/labels`, `POST /repos/:owner/:repo/labels`.
4. **Implement multi-account auth** — For each project, read `github_account` from SQLite. Use `gh auth token` (subprocess) to get the token for that account, or use `gh auth switch` before API calls. Set `Authorization: Bearer <token>` header.
5. **Implement push (tkr → GitHub)** — For each changed task (hash mismatch): create or update GitHub Issue. Map title, body (description), labels (tags). Store issue number/URL/synced_at in SQLite.
6. **Implement pull (GitHub → tkr)** — List issues using ETag for efficiency. For each updated issue: find matching task, update markdown file (title, description, state, tags from labels). Update ETag and synced_at in SQLite.
7. **Implement tag ↔ label mapping** — On push: tkr tags → GitHub labels (create if missing). On pull: GitHub labels → tkr tags (add to markdown frontmatter).
8. **Implement state mapping** — `closed` tkr state → GitHub `closed` state. All other tkr states → GitHub `open` state. On pull: GitHub `closed` → tkr `closed`, GitHub `open` → tkr `open` (or preserve existing tkr state if more specific).
9. **Implement conflict resolution** — Compare `markdown_hash` vs stored hash, and GitHub `updated_at` vs `github_synced_at`. If both changed: last-write-wins by timestamp. Log warning with task ID, local timestamp, remote timestamp.
10. **Add `Sync` CLI subcommand** — Add `Sync { github: bool, status: bool }` to `Commands` enum in `src/cli.rs`. `--github` triggers sync, `--status` shows sync status.
11. **Integrate with daemon** — Add periodic sync (every 5 minutes) to the daemon event loop from Story 05-001. Also support on-demand sync trigger.
12. **Write integration tests** — Test push, pull, conflict resolution, multi-account switching. Use mock GitHub API or `gh` CLI in dry-run mode if possible.

## Relevant Files

| File | Role | Changes |
|------|------|---------|
| `Cargo.toml` | Dependencies | Add `reqwest = { version = "0.12", features = ["json"] }` |
| `src/github_sync.rs` | **NEW** — GitHub sync module | Bidirectional sync, multi-account, conflict resolution |
| `src/main.rs` | Entry point | Add `mod github_sync;` |
| `src/cli.rs` | CLI commands | Add `Sync` subcommand with `--github` and `--status` flags |
| `src/daemon.rs` | Daemon (from 05-001) | Integrate periodic GitHub sync into daemon loop |
| `src/web.rs` | Web server | Sync endpoints delegate to `github_sync` module (endpoints defined in 05-002) |
| `src/ticket.rs` | Ticket model | No changes needed (GitHub fields are in SQLite, not markdown) |
| `tests/cli_tests.rs` | Integration tests | Add sync command tests, multi-account tests |

## Acceptance Criteria

- [ ] `reqwest = { version = "0.12", features = ["json"] }` is added to `Cargo.toml` and `devbox run just build-internal` succeeds.
- [ ] `tkr sync --github` triggers a bidirectional sync for all registered projects with GitHub info.
- [ ] `tkr sync --status` shows the last sync time, number of tasks synced, and any errors.
- [ ] Push (tkr → GitHub): tasks with changed markdown are created/updated as GitHub Issues with correct title, body, and labels.
- [ ] Pull (GitHub → tkr): updated GitHub Issues are pulled into tkr markdown files with updated title, description, state, and tags.
- [ ] Tag → label mapping: tkr tags appear as GitHub labels. New tags create GitHub labels if they don't exist.
- [ ] Label → tag mapping: GitHub labels are pulled back as tkr tags in markdown frontmatter.
- [ ] State mapping: tkr `closed` → GitHub issue `closed`. GitHub `closed` → tkr `closed`. Other states → GitHub `open`.
- [ ] Multi-account: sync works across multiple GitHub accounts (lrepo52, levonk). Each project uses its configured `github_account`.
- [ ] Conflict resolution: when both local and remote changed, last-write-wins by timestamp. Warning logged with task ID and timestamps.
- [ ] ETag support: pull uses `If-None-Match` header for efficient polling (GitHub returns 304 Not Modified when no changes).
- [ ] SQLite tracks: `github_issue_number`, `github_issue_url`, `github_synced_at`, `github_etag` for each synced task.
- [ ] Sync state tracked in `sync_state` table: `last_github_sync`, `github_sync_errors`.
- [ ] Daemon integration: sync runs periodically (every 5 minutes) when daemon is running.
- [ ] `POST /api/sync/github` triggers sync (delegates to this module).
- [ ] `GET /api/sync/status` returns sync status (delegates to this module).
- [ ] All new sync commands have tests.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.

## Examples

### Trigger GitHub sync

```bash
$ tkr sync --github
Syncing 3 projects with GitHub:
  tkr (levonk/job-aide) via account: levonk
    Pushed 2 tasks, pulled 1 task, 0 conflicts
  dotfiles (levonk/dotfiles) via account: levonk
    Pushed 0 tasks, pulled 0 tasks, 0 conflicts
  infrahub (lrepo52/infrahub) via account: lrepo52
    Pushed 1 task, pulled 0 tasks, 1 conflict (resolved: remote newer)
```

### Check sync status

```bash
$ tkr sync --status
GitHub Sync Status:
  Last sync: 2026-09-08T14:30:00Z
  Projects synced: 3
  Total tasks pushed: 3
  Total tasks pulled: 1
  Conflicts: 1 (resolved by last-write-wins)
  Errors: 0

Per-project:
  tkr (levonk/job-aide): last synced 2026-09-08T14:30:00Z, 42 tasks linked
  dotfiles (levonk/dotfiles): last synced 2026-09-08T14:30:00Z, 12 tasks linked
  infrahub (lrepo52/infrahub): last synced 2026-09-08T14:29:55Z, 8 tasks linked
```

### Conflict resolution log (daemon.log)

```
[2026-09-08T14:30:00Z] WARN  Conflict: task ja-6b9a0dc
  Local changed at: 2026-09-08T14:25:00Z (markdown hash: a1b2c3...)
  Remote changed at: 2026-09-08T14:28:00Z (GitHub updated_at)
  Resolution: remote is newer, pulling remote changes
```

### Multi-account auth flow

```
Project: tkr
  github_owner: levonk
  github_repo: job-aide
  github_account: levonk
  → gh auth token (levonk) → ghp_xxx...
  → Authorization: Bearer ghp_xxx...
  → GET /repos/levonk/job-aide/issues

Project: infrahub
  github_owner: lrepo52
  github_repo: infrahub
  github_account: lrepo52
  → gh auth token (lrepo52) → ghp_yyy...
  → Authorization: Bearer ghp_yyy...
  → GET /repos/lrepo52/infrahub/issues
```

### Tag → Label mapping

```
tkr task:
  tags: [security, backend]

GitHub Issue:
  labels: [security, backend]
  (labels created if they don't exist on the repo)
```

## Test Plan

### Unit Tests

1. **Tag ↔ label mapping** — Test that tkr tags correctly map to GitHub labels and vice versa.
2. **State mapping** — Test that tkr `closed` → GitHub `closed`, and all other states → GitHub `open`.
3. **Conflict resolution** — Test that last-write-wins logic picks the correct side based on timestamps.
4. **ETag handling** — Test that ETag is stored and used in subsequent requests (304 Not Modified).

### Integration Tests (in `tests/cli_tests.rs`)

1. **`test_sync_github_command`** — Run `tkr sync --github`, assert it outputs sync results without errors.
2. **`test_sync_status_command`** — Run `tkr sync --status`, assert it outputs sync status.
3. **`test_sync_push_creates_issue`** — Create a task, run sync, assert GitHub Issue is created (requires mock or dry-run mode).
4. **`test_sync_pull_updates_markdown`** — Modify a GitHub Issue, run sync, assert tkr markdown is updated.
5. **`test_sync_multi_account`** — Configure two projects with different `github_account` values, run sync, assert both accounts are used.
6. **`test_sync_conflict_resolution`** — Modify both tkr markdown and GitHub Issue, run sync, assert last-write-wins and warning is logged.
7. **`test_sync_tag_label_mapping`** — Create a task with tags, run sync, assert GitHub labels match tags.

### Test Isolation

- GitHub API calls require authentication — use a mock HTTP server (e.g., `wiremock` or `mockito` crate) for unit/integration tests.
- Alternatively, add a `--dry-run` flag that logs what would be synced without making API calls.
- Do NOT make real GitHub API calls in tests (rate limits, authentication, side effects).
- Use `tempfile::TempDir` for the SQLite database and markdown files.

## Observability

- **Daemon log**: All sync actions logged to `~/.local/share/tkr/daemon.log` with timestamps.
- **`tkr sync --status`**: Exposes last sync time, per-project sync state, error count, conflict count.
- **`GET /api/sync/status`**: Same information as JSON for the web UI.
- **Sync state table**: `sync_state` table in SQLite stores `last_github_sync`, `github_sync_errors`, and per-project sync metadata.
- **Error handling**: Failed API calls are logged with the HTTP status code and response body. Sync continues with the next task/project.

## Compliance

- GitHub API rate limits: respect `X-RateLimit-Remaining` header. Pause sync if rate limit is low (<10 remaining). Log rate limit warnings.
- No GitHub tokens stored in the SQLite database — tokens are fetched on-demand via `gh auth token`.
- No GitHub tokens logged in log files.
- Markdown files remain the source of truth — GitHub Issues are a projection, not a dependency (no-corner principle, PRD lines 159-165).
- The SQLite DB is always rebuildable from markdown + GitHub API (PRD line 161).

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| GitHub API rate limits (5000 req/hour per token) | Medium | Medium | Use ETag for efficient polling; batch operations; pause sync when rate limit is low |
| Multi-account auth token retrieval fails | Medium | High | Log clear error message; skip the project; continue with other projects |
| Conflict resolution loses data (last-write-wins) | Medium | High | Log all conflicts with full details; provide `tkr sync --dry-run` to preview; future: manual conflict resolution UI |
| GitHub API changes / breaking changes | Low | High | Pin to GitHub REST API v3 (stable); monitor deprecation headers |
| `gh` CLI not installed | Medium | High | If using `gh` CLI approach, check for `gh` binary at startup and provide clear error. If using `reqwest`, no `gh` dependency needed |
| Large number of issues causes slow sync | Low | Medium | Use ETag + incremental sync (only fetch issues updated since last sync); paginate with `per_page=100` |
| Tag/label name conflicts (GitHub label names have restrictions) | Low | Low | Sanitize tag names for GitHub (replace spaces with hyphens, lowercase, max 50 chars) |

## Dependencies & Sequencing

- **Depends on**: `05-001` (Daemon watcher) — the sync runs inside the daemon process and uses the same SQLite connection.
- **Dependants**: None (this is a leaf story in the dependency graph).
- **Parallel-safe**: Yes — this story creates `src/github_sync.rs` and adds the `Sync` subcommand to `src/cli.rs`. It does not conflict with `06-001` (web UI) or `06-003` (priority ordering) which touch different files.
- **External dependencies**: `reqwest = { version = "0.12", features = ["json"] }` (or `gh` CLI via subprocess).

## Definition of Done

- [ ] `reqwest` (or `gh` CLI approach) added to `Cargo.toml`.
- [ ] `src/github_sync.rs` created with bidirectional sync logic.
- [ ] `mod github_sync;` added to `src/main.rs`.
- [ ] `Sync` subcommand added to `src/cli.rs` with `--github` and `--status` flags.
- [ ] Push (tkr → GitHub) creates/updates GitHub Issues with correct title, body, labels.
- [ ] Pull (GitHub → tkr) updates markdown files with title, description, state, tags.
- [ ] Tag ↔ label mapping works in both directions.
- [ ] State mapping works (closed ↔ closed, others ↔ open).
- [ ] Multi-account support works (lrepo52, levonk).
- [ ] Conflict resolution (last-write-wins) implemented with warning logging.
- [ ] ETag support for efficient polling.
- [ ] SQLite tracks `github_issue_number`, `github_issue_url`, `github_synced_at`, `github_etag`.
- [ ] Daemon integration: periodic sync every 5 minutes.
- [ ] `tkr sync --github` and `tkr sync --status` commands work.
- [ ] Integration tests pass (using mock GitHub API or dry-run mode).
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.
- [ ] No secrets or credentials committed.
- [ ] Conventional commit message used.

## STOP Conditions

- If GitHub API authentication via `gh auth token` is unreliable or requires interactive login, stop and evaluate using personal access tokens stored in a config file (with appropriate file permissions).
- If the `reqwest` crate causes build issues on the target platform, stop and evaluate using `gh api` via `std::process::Command` as the sole approach.
- If GitHub API rate limits prevent testing, stop and implement a mock HTTP server for tests before proceeding.
- If conflict resolution (last-write-wins) causes data loss in testing, stop and evaluate adding a conflict resolution UI or a `--force-local` / `--force-remote` flag.
- If multi-account auth via `gh auth switch` is not supported or causes race conditions, stop and evaluate using separate tokens per account with `reqwest` directly.

## Maintenance Notes

- GitHub REST API v3 is used (not GraphQL) for simplicity. If custom fields (GitHub Projects v2) are needed in the future, switch to GraphQL API.
- The sync interval (5 minutes) is a constant in the daemon code; can be made configurable via `TKR_SYNC_INTERVAL_SECS` environment variable.
- The `sync_state` table is a simple key-value store; add new keys as needed for future sync features.
- GitHub label names are sanitized (lowercase, hyphens for spaces, max 50 chars). The mapping is not reversible for labels with special characters — document this limitation.
- The `github_account` field on the `projects` table maps to a `gh auth` account name. Ensure `gh auth login` has been run for each account before sync.
- ETag-based polling is efficient but requires the `If-None-Match` header. If GitHub changes ETag behavior, the sync will fall back to full issue listing (less efficient but correct).

## Commit Conventions

- Use conventional commits: `feat(sync): add bidirectional GitHub Issues sync with multi-account support`.
- Split commits by sub-task if the diff is large (e.g., one commit for `reqwest` dependency, one for `src/github_sync.rs`, one for CLI subcommand, one for daemon integration, one for tests).
- Reference story ID in PR description: `Story: 06-002`.
- Branch: `feature/current/portfolio-layer/story-06-002-github-sync`.
