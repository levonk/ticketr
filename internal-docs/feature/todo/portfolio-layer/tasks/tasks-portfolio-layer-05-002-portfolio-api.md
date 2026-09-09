---
story_id: "05-002"
story_title: "Portfolio API endpoints (extending web.rs)"
story_name: "portfolio-api"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 5
parallel_id: 2
branch: "feature/current/portfolio-layer/story-05-002-portfolio-api"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["04-001", "04-002", "04-003", "04-004"]
parallel_safe: false
modules: ["web", "api"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend", "api"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Extend the existing `src/web.rs` warp server with a comprehensive REST API for the portfolio layer. The API exposes CRUD endpoints for all 7 hierarchy levels (portfolios, projects, apps, requirements, stories, tasks, ai-tasks, tags), a filtered task query endpoint with composable query parameters, a priority reordering endpoint for drag-and-drop persistence, and GitHub sync trigger/status endpoints. This API is the backend for the Kanban web UI (Story 06-001) and the priority ordering system (Story 06-003).

## Current State

- `src/web.rs` (lines 1-230) has a minimal warp server with two endpoints:
  - `GET /api/tickets` (lines 51-55) — returns all tickets from an in-memory `Arc<RwLock<Vec<Ticket>>>`.
  - `PUT /api/tickets/:id` (lines 57-63) — updates a single ticket's status/title/description/assignee/priority.
- Shared state uses `Arc<RwLock<>>` for both `Vec<Ticket>` and `TicketManager` (lines 41-42).
- Static files are served from `web/` directory (lines 66-68).
- CORS is configured to allow any origin, content-type header, and GET/POST/PUT/DELETE/OPTIONS methods (lines 45-48).
- `TicketApiResponse` struct (lines 188-202) serializes ticket data with fields: id, title, status, project, category, assignee, priority, issue_type, description, created, deps, links.
- `TicketUpdate` struct (lines 223-230) deserializes update payloads.
- No portfolio, project, app, requirement, story, ai-task, tag, priority, or sync endpoints exist.
- No SQLite integration in the web server — all data is in-memory via `TicketManager::list_tickets()`.

## Scope

### In Scope

- Extend `src/web.rs` with the full portfolio API surface defined in the PRD (lines 420-452):
  - **Portfolios**: `GET /api/portfolios`, `POST /api/portfolios`, `PUT /api/portfolios/:id`, `DELETE /api/portfolios/:id`
  - **Projects**: `GET /api/projects`, `POST /api/projects`, `DELETE /api/projects/:id`
  - **Apps**: `GET /api/apps`, `POST /api/apps`, `PUT /api/apps/:id`
  - **Requirements**: `GET /api/requirements`, `POST /api/requirements`, `PUT /api/requirements/:id`, `DELETE /api/requirements/:id`
  - **Stories**: `GET /api/stories`, `POST /api/stories`, `PUT /api/stories/:id`, `DELETE /api/stories/:id`
  - **Tasks**: `GET /api/tasks` with query params (`portfolio`, `project`, `app`, `story`, `requirement`, `tag`, `state`, `assignee`), `GET /api/tasks/:id`, `PUT /api/tasks/:id`
  - **AI Tasks**: `GET /api/ai-tasks`, `POST /api/ai-tasks`, `PUT /api/ai-tasks/:id`
  - **Tags**: `GET /api/tags`, `POST /api/tags`
  - **Priority**: `PUT /api/priority` (body: `{ scope_type, scope_id, task_id, new_position }`), `GET /api/priority?scope_type=&scope_id=`
  - **Sync**: `POST /api/sync/github` (trigger GitHub sync), `GET /api/sync/status` (sync status)
- Integrate the SQLite portfolio database as the data source (replace in-memory `Vec<Ticket>` with SQLite queries via `rusqlite`).
- Shared state: `Arc<RwLock<Connection>>` or connection pool wrapping the SQLite database at `~/.local/share/tkr/portfolio.db`.
- Request/response structs with serde for each entity type.
- Query parameter parsing with `warp::query::query` for filtered task queries.
- Error responses with appropriate HTTP status codes (404 for not found, 400 for bad request, 500 for server error).
- Maintain backward compatibility: existing `GET /api/tickets` should still work (alias to `GET /api/tasks`).

### Out of Scope

- The Kanban web UI itself (Story 06-001).
- GitHub sync implementation logic (Story 06-002) — this story only adds the API endpoints that trigger sync.
- Priority ordering logic and `priority_order` table operations (Story 06-003) — this story adds the API endpoints but delegates to the priority module.
- Daemon process management (Story 05-001).
- SQLite schema creation and migrations (Story 01-001).

## Sub-Tasks

1. **Define API response/request structs** — Create serde structs for each entity: `PortfolioResponse`, `ProjectResponse`, `AppResponse`, `RequirementResponse`, `StoryResponse`, `TaskResponse`, `AiTaskResponse`, `TagResponse`, `PriorityResponse`, `SyncStatusResponse`. Create request structs for POST/PUT bodies.
2. **Create SQLite shared state** — Replace `Arc<RwLock<Vec<Ticket>>>` with `Arc<Mutex<rusqlite::Connection>>` (or `Arc<RwLock<Connection>>`). Initialize from `~/.local/share/tkr/portfolio.db`. Add a `with_db` warp filter similar to existing `with_tickets`/`with_manager` filters.
3. **Implement portfolio endpoints** — `GET /api/portfolios` (list all), `POST /api/portfolios` (create), `PUT /api/portfolios/:id` (update), `DELETE /api/portfolios/:id` (soft delete / state=dissolved).
4. **Implement project endpoints** — `GET /api/projects` (list, optional `?portfolio=` filter), `POST /api/projects` (register), `DELETE /api/projects/:id` (unregister).
5. **Implement app endpoints** — `GET /api/apps` (list, optional `?project=` filter), `POST /api/apps` (create), `PUT /api/apps/:id` (update state).
6. **Implement requirement endpoints** — `GET /api/requirements` (list, optional `?portfolio=` filter), `POST /api/requirements` (create), `PUT /api/requirements/:id` (update), `DELETE /api/requirements/:id` (supersede).
7. **Implement story endpoints** — `GET /api/stories` (list, optional `?requirement=` or `?app=` filter), `POST /api/stories` (create), `PUT /api/stories/:id` (update), `DELETE /api/stories/:id` (archive).
8. **Implement task endpoints** — `GET /api/tasks` with composable query params (`portfolio`, `project`, `app`, `story`, `requirement`, `tag`, `state`, `assignee`). Build SQL WHERE clause dynamically. `GET /api/tasks/:id` (single task). `PUT /api/tasks/:id` (update — writes to markdown via daemon, or directly if daemon not running).
9. **Implement ai-task endpoints** — `GET /api/ai-tasks` (list, optional `?task=` filter), `POST /api/ai-tasks` (create), `PUT /api/ai-tasks/:id` (update state).
10. **Implement tag endpoints** — `GET /api/tags` (list all tags with task counts), `POST /api/tags` (create tag).
11. **Implement priority endpoints** — `PUT /api/priority` (reorder: body `{ scope_type, scope_id, task_id, new_position }`), `GET /api/priority?scope_type=&scope_id=` (read ordering for a scope). Delegate to priority module (Story 06-003 interface).
12. **Implement sync endpoints** — `POST /api/sync/github` (trigger GitHub sync — delegates to Story 06-002 module), `GET /api/sync/status` (returns last sync time, sync state, errors).
13. **Wire all routes** — Compose all route filters with `.or()` chains, maintain CORS and logging middleware.
14. **Write integration tests** — Test each endpoint with `assert_cmd` or direct HTTP requests using a test SQLite database.

## Relevant Files

| File | Role | Changes |
|------|------|---------|
| `src/web.rs` | Web server | **Major extension** — add all portfolio API endpoints, SQLite integration, request/response structs |
| `src/main.rs` | Entry point | Ensure `mod web;` is present (already exists) |
| `src/cli.rs` | CLI commands | No changes needed (Web subcommand already calls `start_web_server`) |
| `Cargo.toml` | Dependencies | Ensure `rusqlite` is available (added in Story 01-001) |
| `tests/cli_tests.rs` | Integration tests | Add API endpoint tests |
| `web/` | Static UI files | No changes (UI is Story 06-001) |

## Acceptance Criteria

- [ ] `GET /api/portfolios` returns all portfolios as JSON array.
- [ ] `POST /api/portfolios` creates a new portfolio and returns 201 with the created portfolio.
- [ ] `PUT /api/portfolios/:id` updates a portfolio's name/description/state.
- [ ] `DELETE /api/portfolios/:id` sets portfolio state to `dissolved`.
- [ ] `GET /api/projects` returns all projects (filtered by `?portfolio=` if provided).
- [ ] `POST /api/projects` registers a new project linked to a portfolio.
- [ ] `DELETE /api/projects/:id` unregisters a project.
- [ ] `GET /api/apps` returns all apps (filtered by `?project=` if provided).
- [ ] `POST /api/apps` creates a new app within a project.
- [ ] `PUT /api/apps/:id` updates an app's state.
- [ ] `GET /api/requirements` returns all requirements (filtered by `?portfolio=` if provided).
- [ ] `POST /api/requirements` creates a new requirement.
- [ ] `PUT /api/requirements/:id` updates a requirement.
- [ ] `DELETE /api/requirements/:id` sets requirement state to `superseded`.
- [ ] `GET /api/stories` returns all stories (filtered by `?requirement=` or `?app=` if provided).
- [ ] `POST /api/stories` creates a new story.
- [ ] `PUT /api/stories/:id` updates a story.
- [ ] `DELETE /api/stories/:id` sets story state to `archived`.
- [ ] `GET /api/tasks?portfolio=&project=&app=&story=&requirement=&tag=&state=&assignee=` returns filtered tasks. Filters compose (multiple params AND together).
- [ ] `GET /api/tasks/:id` returns a single task by ID.
- [ ] `PUT /api/tasks/:id` updates a task (title, description, status, assignee, priority, story, tags).
- [ ] `GET /api/ai-tasks` returns all AI tasks (filtered by `?task=` if provided).
- [ ] `POST /api/ai-tasks` creates a new AI task.
- [ ] `PUT /api/ai-tasks/:id` updates an AI task's state.
- [ ] `GET /api/tags` returns all tags with task counts.
- [ ] `POST /api/tags` creates a new tag.
- [ ] `PUT /api/priority` with body `{ scope_type, scope_id, task_id, new_position }` reorders a task within a scope.
- [ ] `GET /api/priority?scope_type=&scope_id=` returns the ordered task list for a scope.
- [ ] `POST /api/sync/github` triggers GitHub sync and returns 202 Accepted.
- [ ] `GET /api/sync/status` returns sync status (last sync time, in-progress, errors).
- [ ] Existing `GET /api/tickets` still works as an alias to `GET /api/tasks`.
- [ ] All endpoints return appropriate HTTP status codes (200, 201, 400, 404, 500).
- [ ] All endpoints have integration tests.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.

## Examples

### GET /api/tasks with composable filters

```
GET /api/tasks?portfolio=personal&project=tkr&tag=security&state=open

Response (200 OK):
[
  {
    "id": "ja-6b9a0dc",
    "title": "Add rusqlite dependency",
    "state": "open",
    "project": "tkr",
    "app": "default",
    "story": "story-abc123",
    "priority": 2,
    "tags": ["security", "backend"],
    "assignee": "levonk",
    "markdown_path": "/Users/micro/p/gh/levonk/tkr/.tickets/open/ja-6b9a0dc.md"
  }
]
```

### POST /api/portfolios

```
POST /api/portfolios
Content-Type: application/json

{
  "id": "personal",
  "name": "Personal",
  "description": "Personal projects and OSS work"
}

Response (201 Created):
{
  "id": "personal",
  "name": "Personal",
  "description": "Personal projects and OSS work",
  "state": "curated",
  "created": "2026-09-08T12:00:00Z",
  "position": 0
}
```

### PUT /api/priority

```
PUT /api/priority
Content-Type: application/json

{
  "scope_type": "story",
  "scope_id": "story-abc123",
  "task_id": "ja-6b9a0dc",
  "new_position": 0
}

Response (200 OK):
{
  "scope_type": "story",
  "scope_id": "story-abc123",
  "ordered_tasks": ["ja-6b9a0dc", "ja-7c1b2ef", "ja-8d3c4ab"]
}
```

### POST /api/sync/github

```
POST /api/sync/github

Response (202 Accepted):
{
  "status": "syncing",
  "message": "GitHub sync started",
  "projects": ["tkr", "dotfiles"]
}
```

### GET /api/sync/status

```
GET /api/sync/status

Response (200 OK):
{
  "last_sync": "2026-09-08T14:30:00Z",
  "in_progress": false,
  "projects_synced": 3,
  "errors": []
}
```

## Test Plan

### Integration Tests (in `tests/cli_tests.rs` or a new `tests/api_tests.rs`)

1. **`test_api_get_portfolios`** — Create a portfolio via CLI, `GET /api/portfolios`, assert it appears in response.
2. **`test_api_create_portfolio`** — `POST /api/portfolios` with valid body, assert 201 and portfolio appears in `GET /api/portfolios`.
3. **`test_api_update_portfolio`** — `PUT /api/portfolios/:id` with new name, assert 200 and updated name in subsequent GET.
4. **`test_api_delete_portfolio`** — `DELETE /api/portfolios/:id`, assert 200 and state is `dissolved`.
5. **`test_api_get_tasks_filtered`** — Create tasks with different projects/tags/states, `GET /api/tasks?project=tkr&tag=security`, assert only matching tasks returned.
6. **`test_api_get_tasks_compose_filters`** — `GET /api/tasks?portfolio=personal&project=tkr&state=open`, assert filters compose with AND logic.
7. **`test_api_get_task_by_id`** — `GET /api/tasks/:id`, assert 200 with task details.
8. **`test_api_get_task_not_found`** — `GET /api/tasks/nonexistent`, assert 404.
9. **`test_api_put_task`** — `PUT /api/tasks/:id` with new status, assert 200 and markdown file updated.
10. **`test_api_get_tags`** — Create tags via CLI, `GET /api/tags`, assert tags with counts returned.
11. **`test_api_put_priority`** — `PUT /api/priority` with reorder body, assert 200 and `GET /api/priority` reflects new order.
12. **`test_api_get_priority`** — `GET /api/priority?scope_type=story&scope_id=story-abc123`, assert ordered task list.
13. **`test_api_sync_github`** — `POST /api/sync/github`, assert 202 Accepted.
14. **`test_api_sync_status`** — `GET /api/sync/status`, assert 200 with sync status JSON.
15. **`test_api_tickets_alias`** — `GET /api/tickets` returns same data as `GET /api/tasks` (backward compatibility).

### Test Isolation

- Use a temp SQLite database (override `XDG_DATA_HOME` or pass a `--db-path` flag).
- Use `warp::test::request()` for direct route testing without spawning a full server.
- Alternatively, spawn the web server on a random port and use `reqwest` or `assert_cmd` for HTTP tests.

## Observability

- All API endpoints should log requests via `warp::log("web")` (already configured at line 74).
- Error responses should include a JSON body `{ "error": "message" }` for client-side error handling.
- Sync status endpoint provides visibility into GitHub sync health.
- Future: add `/api/health` endpoint for daemon liveness checks.

## Compliance

- All endpoints must validate input (reject malformed JSON, missing required fields).
- DELETE operations are soft deletes (state change, not row deletion) for portfolios, requirements, stories.
- Task updates via `PUT /api/tasks/:id` must write to the markdown file (source of truth), not just SQLite.
- CORS must remain permissive for local development (`allow_any_origin`).
- No authentication required (local-first tool, bound to 127.0.0.1).

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| SQLite connection contention (multiple concurrent API requests) | Medium | High | Use `Arc<Mutex<Connection>>` with short lock scope; enable WAL mode; consider connection pool |
| SQL injection via query parameters | Low | Critical | Use parameterized queries (`rusqlite` params macro), never string-concatenate SQL |
| Breaking existing `/api/tickets` endpoint | Low | Medium | Keep `GET /api/tickets` as alias to `GET /api/tasks`; add deprecation header |
| Large response payloads (all tasks with no filter) | Medium | Low | Add pagination (`?limit=&offset=`) in a future story; for now, return all |
| Task update writes to markdown while daemon is also watching | Medium | Medium | Daemon debounce handles this; API write triggers daemon re-sync which is a no-op (hash unchanged) |

## Dependencies & Sequencing

- **Depends on**:
  - `04-001` (Link tasks to stories) — tasks need `story_id` field for story-filtered queries.
  - `04-002` (Tags) — tasks need tags for tag-filtered queries.
  - `04-003` (Portfolio views CLI) — the view logic can be reused for API query building.
  - `04-004` (AI Task CRUD) — ai-task endpoints need the AI task data model.
- **Dependants**:
  - `06-001` (Kanban web UI) — the UI calls these API endpoints.
  - `06-003` (Priority ordering) — the priority endpoints are defined here but implemented in 06-003.
- **Parallel-safe**: No — this story extensively modifies `src/web.rs` and should not be developed in parallel with other stories touching the same file.
- **External dependencies**: `rusqlite` (from Story 01-001), `warp` (already in Cargo.toml), `serde` (already in Cargo.toml).

## Definition of Done

- [ ] All API endpoints listed in the PRD (lines 420-452) are implemented and return correct responses.
- [ ] SQLite integration replaces in-memory `Vec<Ticket>` as the data source.
- [ ] Query parameter filtering composes correctly (multiple params AND together).
- [ ] `PUT /api/tasks/:id` writes to markdown files (source of truth).
- [ ] `PUT /api/priority` and `GET /api/priority` endpoints work (delegate to priority module).
- [ ] `POST /api/sync/github` and `GET /api/sync/status` endpoints work (delegate to sync module).
- [ ] Existing `GET /api/tickets` backward compatibility maintained.
- [ ] All endpoints have integration tests.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.
- [ ] No secrets or credentials committed.
- [ ] Conventional commit message used.

## STOP Conditions

- If the SQLite connection model (single `Arc<Mutex<Connection>>`) causes deadlocks under concurrent API load, stop and evaluate using a connection pool (`r2d2` + `r2d2_sqlite`).
- If warp's filter composition becomes unmanageable with 30+ routes, stop and evaluate splitting routes into separate modules (`src/web/portfolio_routes.rs`, `src/web/task_routes.rs`, etc.).
- If parameterized query building for composable filters proves error-prone, stop and implement a query builder abstraction with exhaustive tests before proceeding.
- If the `PUT /api/tasks/:id` markdown write conflicts with the daemon's file watcher (infinite loop), stop and coordinate with Story 05-001 to add a "self-triggered" flag that skips re-syncing writes initiated by the API.

## Maintenance Notes

- The API surface is large (30+ endpoints); consider splitting `src/web.rs` into sub-modules if it exceeds 500 lines.
- Request/response structs should be kept in sync with the SQLite schema (Story 01-001). Any schema change requires updating the corresponding API structs.
- The `GET /api/tasks` query parameter set may grow over time; the dynamic WHERE clause builder should be extensible.
- Pagination should be added before the task count grows large enough to cause performance issues.
- The sync endpoints are thin wrappers — the actual sync logic lives in the sync module (Story 06-002). The API only triggers and reports status.

## Commit Conventions

- Use conventional commits: `feat(api): add portfolio API endpoints for all hierarchy levels`.
- Split commits by entity group if the diff is large (e.g., one commit for portfolio/project endpoints, one for task endpoints, one for priority/sync endpoints, one for tests).
- Reference story ID in PR description: `Story: 05-002`.
- Branch: `feature/current/portfolio-layer/story-05-002-portfolio-api`.
