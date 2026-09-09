---
story_id: "06-003"
story_title: "Priority order table + drag-and-drop persistence + CLI reordering"
story_name: "priority-ordering"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 6
parallel_id: 3
branch: "feature/current/portfolio-layer/story-06-003-priority-ordering"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["05-002"]
parallel_safe: true
modules: ["db", "priority"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend", "priority"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Implement the `priority_order` table operations that store relative task ordering per `(scope_type, scope_id)` pair. This enables drag-and-drop priority reordering in the web UI (persisted via `PUT /api/priority`), CLI-based reordering (`tkr priority set` / `tkr priority list`), and correct sort order when displaying tasks in any scope. The seven scope types are: `global`, `portfolio`, `project`, `app`, `requirement`, `story`, `tag`. When a task is reordered within a scope, positions of other tasks in that scope are shifted to maintain a gap-free sequence. The web UI's drag-and-drop (Story 06-001) calls `PUT /api/priority` which delegates to this module.

## Current State

- The `priority_order` table is defined in the PRD schema (lines 287-293):
  ```sql
  CREATE TABLE priority_order (
      scope_type TEXT NOT NULL,
      scope_id TEXT NOT NULL,
      task_id TEXT NOT NULL REFERENCES tasks(id),
      position INTEGER NOT NULL,
      PRIMARY KEY (scope_type, scope_id, task_id)
  );
  ```
- The table is created as part of the SQLite schema migrations (Story 01-001).
- `src/web.rs` has no priority endpoints (defined in Story 05-002 as `PUT /api/priority` and `GET /api/priority`).
- `src/cli.rs` (lines 1-265) has no `Priority` subcommand.
- `src/ticket.rs` `Ticket` struct (lines 7-36) has a `priority: i32` field (1-5 numeric priority), but this is separate from the `priority_order` table which stores relative ordering positions.
- The PRD (lines 378-398) defines the priority formalization: drag-and-drop ordering within a column, scoped to the active filter context. The `priority_order` table stores position per `(scope_type, scope_id)`.
- Scope types and their meanings (PRD lines 384-392): `global` (default ordering), `portfolio`, `project`, `app`, `requirement`, `story`, `tag`.
- The cross-altitude rule (PRD lines 394-399) is enforced in the web UI (Story 06-001), but the backend must support all scope types regardless.

## Scope

### In Scope

- Create `src/priority.rs` module with CRUD operations for the `priority_order` table:
  - `set_position(scope_type, scope_id, task_id, new_position)` — move a task to a new position, shift others.
  - `get_ordering(scope_type, scope_id)` — return ordered list of task IDs for a scope.
  - `get_position(scope_type, scope_id, task_id)` — return the position of a single task.
  - `remove_task(task_id)` — remove all priority_order rows for a task (when task is deleted).
  - `init_task(scope_type, scope_id, task_id)` — add a new task at the end of a scope's ordering.
- Implement position shifting: when a task moves to position N, tasks at positions >= N are shifted +1. When a task is removed, positions > removed position are shifted -1 to maintain gap-free sequence.
- Add `Priority` CLI subcommand:
  - `tkr priority set <scope_type> <scope_id> <task-id> <position>` — set a task's position in a scope.
  - `tkr priority list <scope_type> <scope_id>` — list tasks in order for a scope.
- Implement the backend logic for `PUT /api/priority` and `GET /api/priority` API endpoints (endpoints defined in Story 05-002, logic implemented here).
- Integrate with task listing: when tasks are fetched via `GET /api/tasks`, they should be sorted by `priority_order` position for the relevant scope (fall back to `priority` field or creation date if no ordering exists).
- Handle the case where a task exists in SQLite but has no `priority_order` entry for a scope — auto-assign it to the end of the ordering.
- Support all 7 scope types: `global`, `portfolio`, `project`, `app`, `requirement`, `story`, `tag`.

### Out of Scope

- The web UI drag-and-drop interaction (Story 06-001) — this story provides the backend persistence.
- The API endpoint route definitions (Story 05-002) — this story provides the handler logic.
- The `priority` field on the `Ticket` struct (1-5 numeric priority) — this is a separate concept from relative ordering. The `priority_order` table stores positions, not priority levels.
- SQLite schema creation (Story 01-001) — the `priority_order` table is created there.

## Sub-Tasks

1. **Create `src/priority.rs` module** — Define `PriorityManager` struct wrapping a SQLite connection. Implement `set_position`, `get_ordering`, `get_position`, `remove_task`, `init_task` methods.
2. **Implement `set_position`** — Move a task to `new_position` in `(scope_type, scope_id)`. If task already has a position, shift other tasks to close the gap at old position, then shift tasks at >= new_position to make room. Use a SQLite transaction for atomicity.
3. **Implement `get_ordering`** — Query `priority_order` for a `(scope_type, scope_id)`, join with `tasks` table, return ordered list of task IDs (or full task data).
4. **Implement `get_position`** — Query single task's position in a scope. Return `None` if no entry exists.
5. **Implement `remove_task`** — Delete all `priority_order` rows for a task_id across all scopes. Shift positions to close gaps.
6. **Implement `init_task`** — Add a new task at the end of a scope's ordering (position = max existing position + 1, or 0 if scope is empty).
7. **Implement auto-assignment** — When `get_ordering` is called and some tasks in the scope have no `priority_order` entry, auto-assign them to the end of the ordering and return the complete list.
8. **Add `Priority` CLI subcommand** — Add `Priority { action: PriorityAction }` to `Commands` enum. `PriorityAction::Set { scope_type, scope_id, task_id, position }` and `PriorityAction::List { scope_type, scope_id }`.
9. **Implement API handler logic** — Implement the handler functions for `PUT /api/priority` (calls `set_position`) and `GET /api/priority` (calls `get_ordering`). These are called by the route handlers in `src/web.rs` (Story 05-002).
10. **Integrate with task listing** — When `GET /api/tasks` returns tasks filtered by a scope (e.g., `?story=story-abc123`), sort results by `priority_order` position for that scope. Fall back to `priority` field, then `created` date.
11. **Write integration tests** — Test set_position, get_ordering, gap-free shifting, auto-assignment, CLI commands, API handlers.

## Relevant Files

| File | Role | Changes |
|------|------|---------|
| `src/priority.rs` | **NEW** — Priority ordering module | `PriorityManager` with CRUD operations for `priority_order` table |
| `src/main.rs` | Entry point | Add `mod priority;` |
| `src/cli.rs` | CLI commands | Add `Priority` subcommand with `Set` and `List` actions |
| `src/web.rs` | Web server | Wire `PUT /api/priority` and `GET /api/priority` handlers to `PriorityManager` (route definitions from 05-002) |
| `tests/cli_tests.rs` | Integration tests | Add priority CLI tests, API handler tests |

## Acceptance Criteria

- [ ] `tkr priority set <scope_type> <scope_id> <task-id> <position>` moves a task to the specified position and shifts other tasks to maintain gap-free ordering.
- [ ] `tkr priority list <scope_type> <scope_id>` returns tasks in priority order for the specified scope.
- [ ] All 7 scope types are supported: `global`, `portfolio`, `project`, `app`, `requirement`, `story`, `tag`.
- [ ] `set_position` is atomic (uses SQLite transaction) — no partial updates on failure.
- [ ] After `set_position`, positions are gap-free (0, 1, 2, 3, ... with no skips).
- [ ] When a task is removed from a scope, remaining tasks' positions are shifted to close the gap.
- [ ] When a new task is added to a scope (via `init_task`), it gets position = max + 1.
- [ ] When `get_ordering` is called and some tasks have no `priority_order` entry, they are auto-assigned to the end.
- [ ] `PUT /api/priority` with body `{ scope_type, scope_id, task_id, new_position }` persists the reorder and returns the updated ordering.
- [ ] `GET /api/priority?scope_type=&scope_id=` returns the ordered task list for the scope.
- [ ] `GET /api/tasks` with a scope filter (e.g., `?story=story-abc123`) returns tasks sorted by `priority_order` position for that scope.
- [ ] Tasks without a `priority_order` entry fall back to `priority` field, then `created` date for sorting.
- [ ] `remove_task` cleans up all `priority_order` entries for a deleted task across all scopes.
- [ ] All priority commands have integration tests.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.

## Examples

### CLI: Set priority within a story

```bash
$ tkr priority set story story-abc123 ja-6b9a0dc 0
Moved ja-6b9a0dc to position 0 in scope (story, story-abc123)
Updated ordering:
  0: ja-6b9a0dc (Add rusqlite dependency)
  1: ja-7c1b2ef (Create schema migrations)
  2: ja-8d3c4ab (Write migration tests)
```

### CLI: List priority for a scope

```bash
$ tkr priority list story story-abc123
Priority ordering for (story, story-abc123):
  0: ja-6b9a0dc - Add rusqlite dependency [open]
  1: ja-7c1b2ef - Create schema migrations [open]
  2: ja-8d3c4ab - Write migration tests [in_progress]
  3: ja-9e4d5cd - Add notify dependency [logged]
```

### CLI: Set global priority

```bash
$ tkr priority set global global ja-6b9a0dc 0
Moved ja-6b9a0dc to position 0 in scope (global, global)
```

### API: PUT /api/priority

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
  "ordered_tasks": [
    {"task_id": "ja-6b9a0dc", "position": 0, "title": "Add rusqlite dependency"},
    {"task_id": "ja-7c1b2ef", "position": 1, "title": "Create schema migrations"},
    {"task_id": "ja-8d3c4ab", "position": 2, "title": "Write migration tests"}
  ]
}
```

### API: GET /api/priority

```
GET /api/priority?scope_type=story&scope_id=story-abc123

Response (200 OK):
{
  "scope_type": "story",
  "scope_id": "story-abc123",
  "ordered_tasks": [
    {"task_id": "ja-6b9a0dc", "position": 0, "title": "Add rusqlite dependency", "state": "open"},
    {"task_id": "ja-7c1b2ef", "position": 1, "title": "Create schema migrations", "state": "open"},
    {"task_id": "ja-8d3c4ab", "position": 2, "title": "Write migration tests", "state": "in_progress"}
  ]
}
```

### Position shifting example

```
Before: [A(0), B(1), C(2), D(3)]
Move C to position 0:
After:  [C(0), A(1), B(2), D(3)]

Before: [A(0), B(1), C(2), D(3)]
Move A to position 2:
After:  [B(0), C(1), A(2), D(3)]

Before: [A(0), B(1), C(2), D(3)]
Remove B:
After:  [A(0), C(1), D(2)]
```

## Test Plan

### Unit Tests

1. **`test_set_position_basic`** — Set position 0 for a task, verify ordering is correct.
2. **`test_set_position_shifts_others`** — Move a task from position 2 to 0, verify others shift down.
3. **`test_set_position_gap_free`** — After multiple set_position calls, verify no gaps in positions.
4. **`test_get_ordering_empty_scope`** — Get ordering for a scope with no entries, verify empty list.
5. **`test_get_ordering_auto_assign`** — Tasks exist in SQLite but no priority_order entries; call get_ordering, verify auto-assigned to end.
6. **`test_remove_task_closes_gap`** — Remove a task from the middle, verify positions shift to close gap.
7. **`test_init_task_appends`** — Init a new task, verify it gets max_position + 1.
8. **`test_all_scope_types`** — Test set_position and get_ordering for each of the 7 scope types.

### Integration Tests (in `tests/cli_tests.rs`)

1. **`test_cli_priority_set`** — Run `tkr priority set story story-abc123 ja-6b9a0dc 0`, assert exit 0 and output contains "Moved".
2. **`test_cli_priority_list`** — Run `tkr priority list story story-abc123`, assert exit 0 and output contains ordered task list.
3. **`test_cli_priority_set_global`** — Run `tkr priority set global global ja-6b9a0dc 0`, assert exit 0.
4. **`test_cli_priority_invalid_scope_type`** — Run `tkr priority set invalid_scope scope1 ja-6b9a0dc 0`, assert error message for invalid scope type.
5. **`test_api_put_priority`** — Call `PUT /api/priority` with valid body, assert 200 and ordered_tasks in response.
6. **`test_api_get_priority`** — Call `GET /api/priority?scope_type=story&scope_id=story-abc123`, assert 200 with ordered list.
7. **`test_api_tasks_sorted_by_priority`** — Call `GET /api/tasks?story=story-abc123`, assert tasks are sorted by priority_order position.
8. **`test_priority_persistence_across_restart`** — Set priority, restart daemon, verify ordering persists (stored in SQLite).

### Test Isolation

- Use `tempfile::TempDir` for the SQLite database.
- Override `XDG_DATA_HOME` to redirect the database to a temp location.
- Each test creates its own set of tasks and priority entries to avoid interference.

## Observability

- `tkr priority list` output shows the full ordering with positions and task titles.
- `PUT /api/priority` response includes the full updated ordering for client-side verification.
- Log warnings when auto-assignment occurs (tasks without explicit ordering).
- Log errors when position shifting fails (e.g., SQLite transaction failure).

## Compliance

- All position updates must be atomic (SQLite transaction).
- Position values must be non-negative integers (0-based).
- The `priority_order` table is a rebuildable index — if corrupted, it can be rebuilt by assigning default ordering (by `created` date).
- The `priority` field on the `Ticket` struct (1-5) is separate from `priority_order` positions — do not conflate them.
- Scope type validation: reject unknown scope types with a clear error message.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Concurrent reorders cause race conditions | Medium | High | Use SQLite transactions with `BEGIN IMMEDIATE` to serialize writes; retry on `SQLITE_BUSY` |
| Position gaps accumulate over time | Low | Low | Run a compaction step after each set_position (renumber to 0, 1, 2, ...) |
| Auto-assignment creates unexpected ordering | Medium | Low | Log when auto-assignment occurs; user can explicitly set positions to override |
| Large scopes (100+ tasks) cause slow position shifting | Low | Medium | Use batch UPDATE in a single SQL statement rather than row-by-row updates |
| Task exists in multiple scopes with conflicting orders | None | None | This is by design — each (scope_type, scope_id) is independent. No conflict. |

## Dependencies & Sequencing

- **Depends on**: `05-002` (Portfolio API endpoints) — the API route definitions for `PUT /api/priority` and `GET /api/priority` are in that story; this story provides the handler logic.
- **Dependants**: None (this is a leaf story in the dependency graph).
- **Parallel-safe**: Yes — this story creates `src/priority.rs` and adds the `Priority` subcommand to `src/cli.rs`. It does not conflict with `06-001` (web UI) or `06-002` (GitHub sync) which touch different files.
- **External dependencies**: `rusqlite` (from Story 01-001).

## Definition of Done

- [ ] `src/priority.rs` created with `PriorityManager` and all CRUD operations.
- [ ] `mod priority;` added to `src/main.rs`.
- [ ] `Priority` subcommand added to `src/cli.rs` with `Set` and `List` actions.
- [ ] `set_position` maintains gap-free ordering with atomic transactions.
- [ ] `get_ordering` returns ordered list with auto-assignment for missing entries.
- [ ] `remove_task` cleans up all priority_order entries and closes gaps.
- [ ] `PUT /api/priority` handler works (delegates to `PriorityManager::set_position`).
- [ ] `GET /api/priority` handler works (delegates to `PriorityManager::get_ordering`).
- [ ] `GET /api/tasks` with scope filter returns tasks sorted by `priority_order`.
- [ ] All 7 scope types supported and tested.
- [ ] Integration tests pass for CLI and API.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.
- [ ] `devbox run just lint-internal` passes.
- [ ] No secrets or credentials committed.
- [ ] Conventional commit message used.

## STOP Conditions

- If concurrent reorders cause data corruption despite transactions, stop and evaluate using a single-writer model (all priority writes go through the daemon, CLI commands send messages to the daemon).
- If the auto-assignment logic causes unexpected ordering for existing users, stop and make auto-assignment opt-in (only assign when explicitly requested, not on every `get_ordering` call).
- If position shifting for large scopes (100+ tasks) is too slow, stop and implement batch UPDATE optimization before proceeding.
- If the `priority` field (1-5) on the `Ticket` struct and the `priority_order` table cause user confusion, stop and document the distinction clearly in CLI help text and API docs.

## Maintenance Notes

- The `priority_order` table is a rebuildable index — if it gets corrupted, truncate and rebuild by assigning default ordering (by `created` date).
- Position compaction (renumbering to 0, 1, 2, ...) runs after each `set_position` call to maintain gap-free sequences. For very large scopes, this could be optimized to run periodically instead of on every write.
- The 7 scope types are fixed by the PRD. Adding new scope types requires updating the validation logic in `src/priority.rs` and the CLI help text.
- The `priority` field (1-5) on the `Ticket` struct is the original tkr priority system. The `priority_order` table is the new relative ordering system. They coexist — `priority_order` is used for drag-and-drop ordering, `priority` is used as a fallback sort key.
- The `PUT /api/priority` response includes the full updated ordering so the web UI can update its DOM without a separate `GET` call.

## Commit Conventions

- Use conventional commits: `feat(priority): add priority_order table operations and CLI reordering`.
- Split commits by sub-task if the diff is large (e.g., one commit for `src/priority.rs`, one for CLI subcommand, one for API handler wiring, one for tests).
- Reference story ID in PR description: `Story: 06-003`.
- Branch: `feature/current/portfolio-layer/story-06-003-priority-ordering`.
