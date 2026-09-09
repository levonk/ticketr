---
story_id: "04-004"
story_title: "AI Task CRUD"
story_name: "ai-task-crud"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 4
parallel_id: 4
branch: "feature/current/portfolio-layer/story-04-004-ai-task-crud"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["03-004"]
parallel_safe: true
modules: ["cli", "ai-task", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Add the AI Task level — the seventh and lowest tier of the portfolio
hierarchy. AI Tasks are delegated subtasks linked to a parent Task (e.g.
"write migration tests" under "add rusqlite dependency"). They live in
the portfolio SQLite DB (not in per-project markdown, since they are
execution-level and agent-owned) and are managed via a new `tkr ai-task`
CLI with `create`, `list`, `show`, and `update-state` subcommands. AI
Tasks use their own state vocabulary — `identified`, `dispatched`,
`running`, `returned` — which is disjoint from every other level so the
level is identifiable from the state alone.

## Current State

- `src/cli.rs` has no `ai-task` subcommand (the `Commands` enum at line
  25 covers only per-project ticket ops + web/tui).
- `src/ticket.rs` has no AI Task concept; AI Tasks are DB-only, not
  markdown `Ticket`s.
- Story 03-004 delivers `tkr sync` and the `tasks` table; AI Tasks
  reference `tasks(id)` via `ai_tasks.task_id`.
- The PRD data model (lines 260-270) defines the `ai_tasks` table with
  `id`, `task_id`, `title`, `state`, `agent_profile`, `created`,
  `completed`, `result_summary`.
- The PRD state vocabulary (line 86) defines AI Task states:
  `identified → dispatched → running → returned`.
- The PRD altitude mapping (line 106) places AI Task at the Operational
  level alongside Task.

## Scope

**In scope:**
- Create the `ai_tasks` table in the portfolio DB (migration in the db
  module from 01-001), per PRD lines 260-270.
- Add a `tkr ai-task` subcommand to `src/cli.rs` with variants:
  - `create <task-id> <title> [--agent-profile=<profile>]` — creates an
    AI Task linked to a parent task, state defaults to `identified`.
  - `list [--task=<task-id>] [--state=<state>]` — lists AI Tasks,
    optionally filtered by parent task or state.
  - `show <ai-task-id>` — prints AI Task details.
  - `update-state <ai-task-id> <state>` — transitions the AI Task state;
    validates against the allowed set.
- AI Task state values: `identified`, `dispatched`, `running`, `returned`.
  Validate on `update-state`; reject unknown states.
- `agent_profile` is an optional string (e.g. `subagent_general`); stored
  on create, displayed on `show`.
- `create` validates that the parent `<task-id>` exists in the `tasks`
  table.
- `update-state` to `returned` stamps `completed` with the current
  timestamp and optionally stores `result_summary` (via `--summary` flag).
- AI Task IDs are generated (e.g. `ai-<short-hash>`).
- All commands query/mutate the SQLite DB only; no markdown writes.

**Out of scope:**
- Agent dispatch/execution (this is CRUD only, not orchestration).
- Markdown representation of AI Tasks (they are DB-only by design).
- Web UI for AI Tasks (06-001).
- AI Task → GitHub sync (06-002 may map them to sub-issues later).
- State transition graph enforcement (e.g. preventing `identified` →
  `returned` skips) — MVP allows any valid state to be set; a future
  story can add transition validation.

## Sub-Tasks

1. Add an `ai_tasks` migration to the db module (schema per PRD lines
   260-270).
2. Add an `AiTask` struct (in a new `src/ai_task.rs` or in the db module)
   with serde for any future JSON output.
3. Add a DB access layer: `create_ai_task`, `list_ai_tasks`,
   `get_ai_task`, `update_ai_task_state` functions.
4. Implement AI Task ID generation (`ai-<short-hash>`).
5. Add an `AiTask` subcommand enum to `Commands` in `src/cli.rs` with
   `Create`, `List`, `Show`, `UpdateState` variants.
6. Wire the new commands in `Commands::execute`.
7. Implement state validation against
   `["identified", "dispatched", "running", "returned"]`.
8. Implement parent task existence check on `create`.
9. Implement `completed` timestamping on transition to `returned` and
   `result_summary` storage via `--summary`.
10. Write integration tests in `tests/cli_tests.rs` (using a temp DB
    seeded with a parent task via `tkr sync`).

## Relevant Files

- `src/cli.rs` — `Commands` enum (line 25), `Commands::execute` (line 130).
- `src/db.rs` (from 01-001) — migration runner; add the `ai_tasks`
  migration here.
- `src/ai_task.rs` — new module (this story) for the struct + DB access.
- `src/sync.rs` (from 03-004) — used in tests to seed a parent `tasks`
  row from markdown.
- `tests/cli_tests.rs` — CLI integration test patterns.
- PRD: `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md`
  (lines 260-270 for `ai_tasks` schema, line 86 for state vocabulary,
  line 106 for altitude mapping).

## Acceptance Criteria

- [ ] `tkr ai-task create <task-id> <title>` creates an AI Task linked to
      the parent task with state `identified` and prints the new AI Task
      ID.
- [ ] `tkr ai-task create --agent-profile=subagent_general <task-id>
      <title>` stores the agent profile.
- [ ] Creating an AI Task for a non-existent parent task fails with a
      clear error.
- [ ] `tkr ai-task list` prints all AI Tasks (`<ai-task-id> - <title>
      (<state>) -> <task-id>`).
- [ ] `tkr ai-task list --task=<task-id>` filters to AI Tasks under that
      parent.
- [ ] `tkr ai-task list --state=running` filters by state.
- [ ] `tkr ai-task show <ai-task-id>` prints full details (id, title,
      state, agent_profile, parent task_id, created, completed,
      result_summary).
- [ ] `tkr ai-task update-state <ai-task-id> <state>` updates the state
      and validates against the allowed set.
- [ ] `tkr ai-task update-state <ai-task-id> returned --summary="tests
      written"` stamps `completed` and stores `result_summary`.
- [ ] Updating to an unknown state fails with a clear error listing the
      valid states.
- [ ] AI Task state values are exactly `identified`, `dispatched`,
      `running`, `returned` (no overlap with other levels).
- [ ] Commands only touch the SQLite DB; no markdown files are created
      or modified.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes with new tests.
- [ ] `devbox run just lint-internal` passes.

## Examples

```bash
# Create an AI Task under a parent task
tkr ai-task create ja-6b9a0dc "Write migration tests"
# => ai-3f8a1c

# Create with an agent profile
tkr ai-task create ja-6b9a0dc "Generate schema docs" --agent-profile=subagent_general
# => ai-4b9e2d

# Create against a missing parent
tkr ai-task create ja-nope "Orphan"
# => Error: parent task "ja-nope" not found

# List all
tkr ai-task list
# ai-3f8a1c - Write migration tests (identified) -> ja-6b9a0dc
# ai-4b9e2d - Generate schema docs (identified) -> ja-6b9a0dc

# List filtered by parent
tkr ai-task list --task=ja-6b9a0dc
# ai-3f8a1c - Write migration tests (identified) -> ja-6b9a0dc
# ai-4b9e2d - Generate schema docs (identified) -> ja-6b9a0dc

# List filtered by state
tkr ai-task list --state=running
# ai-3f8a1c - Write migration tests (running) -> ja-6b9a0dc

# Show
tkr ai-task show ai-3f8a1c
# id: ai-3f8a1c
# title: Write migration tests
# state: identified
# agent_profile: (none)
# task_id: ja-6b9a0dc
# created: 2026-09-08T12:00:00Z
# completed: (none)
# result_summary: (none)

# Update state
tkr ai-task update-state ai-3f8a1c dispatched
# => Updated ai-3f8a1c -> dispatched
tkr ai-task update-state ai-3f8a1c running
# => Updated ai-3f8a1c -> running
tkr ai-task update-state ai-3f8a1c returned --summary="12 tests added, all green"
# => Updated ai-3f8a1c -> returned (completed: 2026-09-08T12:05:00Z)

# Invalid state
tkr ai-task update-state ai-3f8a1c closed
# => Error: invalid AI Task state "closed". Valid: identified, dispatched, running, returned
```

## Test Plan

1. **Setup:** in a temp dir, register a portfolio + project, create a
   task in markdown, run `tkr sync` so the `tasks` row exists.
2. **CLI — create:** run `tkr ai-task create <task> "title"`, assert an
   `ai-` ID is printed and the row exists in the DB with state
   `identified`.
3. **CLI — create with profile:** run with `--agent-profile`, assert the
   profile is stored.
4. **CLI — create missing parent:** run with a bogus task ID, assert
   non-zero exit and error, assert no row created.
5. **CLI — list:** create several AI Tasks, run `tkr ai-task list`, assert
   all appear with id/title/state/parent.
6. **CLI — list --task:** run with `--task=<id>`, assert only that
   parent's AI Tasks appear.
7. **CLI — list --state:** run with `--state=running`, assert only
   running AI Tasks appear.
8. **CLI — show:** run `tkr ai-task show <id>`, assert all fields print.
9. **CLI — update-state:** transition through `identified → dispatched →
   running → returned`, assert each state persists.
10. **CLI — update-state returned:** assert `completed` is stamped and
    `result_summary` stored when `--summary` is given.
11. **CLI — invalid state:** run `tkr ai-task update-state <id> closed`,
    assert non-zero exit and the valid-states error message.
12. **Read-only on markdown:** confirm no `.tickets/` files are created or
    modified by any AI Task command.
13. **Regression:** full `tests/cli_tests.rs` suite passes.

## Observability

- `tkr ai-task create` prints the new ID to stdout (for piping).
- `tkr ai-task update-state` prints the transition and, for `returned`,
    the completed timestamp.
- `tkr ai-task list` output is plain text (one row per line) for easy
  piping; a future `--json` flag can be added.
- `RUST_LOG=debug` logs the SQL statements and the generated AI Task ID.

## Compliance

- No new dependencies.
- No secrets or credentials touched (agent_profile is a name, not a
  credential).
- MIT license unaffected.
- AI Tasks are DB-only by design — no markdown format change, so no
  backward-compatibility risk for existing tickets.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Parent task not synced → dangling AI Task | Medium | Medium | Validate parent existence at create time; document that `tkr sync` must run first. |
| State vocabulary drift (someone adds "closed") | Medium | High | Centralize the valid-state list as a const; validate on every update; test the rejection. |
| AI Task IDs collide | Low | Low | Use timestamp + UUID suffix like `Ticket::generate_id`. |
| Orphaned AI Tasks after parent task deletion | Low | Medium | Out of scope for MVP; note for future cascade-delete story. |
| `result_summary` grows unbounded | Low | Low | Store as TEXT; acceptable. A future truncation/cleanup can cap it. |

## Dependencies & Sequencing

- **Depends on:** 03-004 (sync — `tasks` table must exist for the
  `ai_tasks.task_id` FK and for parent-existence validation).
- **Blocks:** 05-002 (portfolio API — AI Task endpoints), 06-001
  (Kanban UI — AI Task display under parent cards).
- **Parallel-safe:** yes — adds a new table + new CLI; disjoint from the
  other Phase 04 stories (ticket frontmatter changes in 04-001/04-002,
  read-only views in 04-003). The only shared surface is the db module's
  migration runner, which is additive.

## Definition of Done

- [ ] All acceptance criteria met.
- [ ] New tests added and passing (`devbox run just test-internal`).
- [ ] Lint clean (`devbox run just lint-internal`).
- [ ] Build clean (`devbox run just build-internal`).
- [ ] No markdown files created or modified by AI Task commands.
- [ ] State vocabulary exactly matches the PRD (4 values, no overlap).
- [ ] PR describes the "why" (delegated execution layer) not just the
      "what".
- [ ] Conventional commit message used.

## STOP Conditions

- Stop and escalate if 03-004 is not merged — AI Tasks reference the
  `tasks` table and need a parent task row to validate against.
- Stop if the db module from 01-001 does not support additive migrations
  — this story needs a new `ai_tasks` migration, not a schema rewrite.
- Stop if there is disagreement about AI Tasks being DB-only vs markdown
  — the PRD specifies DB-only (they are agent-owned execution items),
  but escalate if a reviewer requires markdown mirroring.

## Maintenance Notes

- AI Tasks are the only DB-only entity in the hierarchy (all others are
  markdown-sourced). Keep the access layer isolated in `src/ai_task.rs`
  so the DB-only nature is obvious and the sync engine does not try to
  index them from markdown.
- The valid-state list is a single const — keep it referenced from both
  the CLI validation and any future API validation to avoid drift.
- When agent dispatch/execution is added (future story), it will call
  `update-state` to drive the lifecycle; keep the state-stamping logic
  (`completed` on `returned`) in the DB access layer, not the CLI, so the
  future dispatcher reuses it.
- `agent_profile` is a free-form string for now; a future registry of
  profiles can add validation.

## Commit Conventions

- Branch: `feature/current/portfolio-layer/story-04-004-ai-task-crud`
- Commit subject prefix: `feat(portfolio): `
- Suggested commits:
  - `feat(portfolio): add ai_tasks table and migration`
  - `feat(portfolio): add ai_task module with DB access layer`
  - `feat(portfolio): add tkr ai-task create/list/show/update-state`
  - `feat(portfolio): validate ai-task states and parent task`
  - `test(portfolio): cover ai-task CRUD and state transitions`
