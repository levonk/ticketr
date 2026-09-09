---
story_id: "04-003"
story_title: "Portfolio views CLI (by-requirement, by-story, by-tag, by-project)"
story_name: "portfolio-views"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 4
parallel_id: 3
branch: "feature/current/portfolio-layer/story-04-003-portfolio-views"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["03-002", "03-003", "03-004"]
parallel_safe: true
modules: ["cli", "views", "db"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Add a `tkr portfolio view` subcommand that queries the portfolio SQLite DB
and prints cross-project groupings of tasks. The four views —
`--by-requirement`, `--by-story`, `--by-tag=<tag>`, `--by-project` — give
the CLI user the same cross-altitude visibility that the Kanban board
(06-001) will provide in the browser. Each group shows a header, its
member tasks, and a count, so the output doubles as a live histogram of
the portfolio.

## Current State

- `src/cli.rs` has no `portfolio` subcommand (the `Commands` enum at line
  25 covers only per-project ticket ops + web/tui).
- Story 02-001 delivers `tkr portfolio create/list/show/dissolve` and the
  `portfolios` table.
- Story 03-002 delivers `tkr requirement` CRUD and the `requirements` table.
- Story 03-003 delivers `tkr story` CRUD and the `stories` table.
- Story 03-004 delivers `tkr sync` which populates `tasks` (and, via 04-001
  and 04-002, `tasks.story_id` and `task_tags`).
- The PRD CLI section (lines 491-496) specifies the four view flags.

## Scope

**In scope:**
- Add a `portfolio view` subcommand to `src/cli.rs` with flags:
  `--by-requirement`, `--by-story`, `--by-tag=<tag>`, `--by-project`.
  Exactly one flag is required; error if none or multiple are given.
- `--by-requirement`: group tasks by their story's requirement (via
  `stories.requirement_id`), then by requirement title. Show requirements
  with zero tasks (empty-state).
- `--by-story`: group tasks by `tasks.story_id` joined to `stories.title`.
  Include a `(none)` group for tasks with no story link.
- `--by-tag=<tag>`: if a tag is given, show tasks carrying that tag grouped
  by project; if no tag value, list all tags with counts (like `tkr tag
  list` but scoped to the view).
- `--by-project`: group tasks by `projects.name`, show project header +
  task list + count.
- Each group prints: group header, count, and the member tasks
  (`<task-id> - <title> (<state>)`).
- Print a total line: `Showing <N> tasks across <M> groups`.
- Query the SQLite DB directly (read-only); no markdown writes.
- Support `--json` output for machine consumption (optional but
  recommended).

**Out of scope:**
- The Kanban board UI (06-001).
- Priority ordering within groups (06-003).
- Filtering by state/assignee in the view (future enhancement; the PRD
  API supports it but the CLI view MVP is grouping only).
- Writing/saving views.

## Sub-Tasks

1. Add a `Portfolio` subcommand enum to `Commands` in `src/cli.rs` (or
   extend the one from 02-001) with a `View` variant carrying the four
   flags + optional `--json`.
2. Add a `portfolio_view` module (e.g. `src/portfolio_view.rs`) with one
   function per grouping that runs a SQL query and returns structured
   rows.
3. Implement `view_by_requirement`: join `tasks` → `stories` →
   `requirements`, group by requirement, count tasks.
4. Implement `view_by_story`: join `tasks` → `stories`, group by story,
   include `(none)` bucket for null `story_id`.
5. Implement `view_by_tag`: if tag given, join `tasks` → `task_tags` →
   `tags` filtered by tag, group by project; if no tag, list tags with
   counts.
6. Implement `view_by_project`: join `tasks` → `projects`, group by
   project.
7. Implement the text renderer: group header, count, task lines, total.
8. Implement the `--json` renderer (serde-serialize the grouped rows).
9. Validate exactly one grouping flag; error otherwise.
10. Wire the command in `Commands::execute`.
11. Write integration tests in `tests/cli_tests.rs` (using a temp DB
    seeded via `tkr sync`).

## Relevant Files

- `src/cli.rs` — `Commands` enum (line 25), `Commands::execute` (line 130).
- `src/db.rs` (from 01-001) — connection helper; reuse for read queries.
- `src/sync.rs` (from 03-004) — used in tests to seed the DB from
  markdown.
- `src/portfolio_view.rs` — new module (this story).
- `tests/cli_tests.rs` — CLI integration test patterns.
- PRD: `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md`
  (lines 491-496 for CLI, lines 408-417 for view descriptions, lines
  354-376 for count-annotation semantics).

## Acceptance Criteria

- [ ] `tkr portfolio view --by-requirement` groups tasks by requirement
      and prints counts per group.
- [ ] `tkr portfolio view --by-story` groups tasks by story, includes a
      `(none)` group for unlinked tasks.
- [ ] `tkr portfolio view --by-tag=security` shows tasks tagged
      `security` grouped by project.
- [ ] `tkr portfolio view --by-project` groups tasks by project.
- [ ] Each group shows a header, a count, and its tasks
      (`<id> - <title> (<state>)`).
- [ ] A total line `Showing <N> tasks across <M> groups` is printed.
- [ ] Empty groups (count 0) are shown but marked as empty, matching the
      PRD's empty-state indicator concept.
- [ ] Running with no grouping flag errors with a clear message.
- [ ] Running with multiple grouping flags errors with a clear message.
- [ ] `--json` outputs valid JSON of the grouped rows.
- [ ] The command only reads the DB; it does not write to markdown or DB.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes with new tests.
- [ ] `devbox run just lint-internal` passes.

## Examples

```bash
# By requirement
tkr portfolio view --by-requirement
# Multi-account Sync (9)
#   ja-6b9a0dc - Add rusqlite dependency (in_progress)
#   ja-7c1b2ee - Add notify watcher (open)
#   ...
# Offline-first (5)
#   ...
# Showing 14 tasks across 2 groups

# By story
tkr portfolio view --by-story
# Portfolio Layer (5)
#   ja-6b9a0dc - Add rusqlite dependency (in_progress)
#   ...
# Auth Refactor (3)
#   ...
# (none) (6)
#   ja-9f3a1bb - Tidy docs (open)
#   ...
# Showing 14 tasks across 3 groups

# By tag
tkr portfolio view --by-tag=security
# tkr (3)
#   ja-6b9a0dc - Add rusqlite dependency (in_progress)
#   ...
# dotfiles (1)
#   ...
# Showing 4 tasks across 2 groups

# By project
tkr portfolio view --by-project
# tkr (8)
#   ja-6b9a0dc - Add rusqlite dependency (in_progress)
#   ...
# dotfiles (4)
#   ...
# Showing 12 tasks across 2 groups

# JSON
tkr portfolio view --by-story --json
# [{"group":"Portfolio Layer","count":5,"tasks":[...]}]

# Errors
tkr portfolio view
# => Error: specify one of --by-requirement, --by-story, --by-tag, --by-project
tkr portfolio view --by-story --by-project
# => Error: specify exactly one grouping flag
```

## Test Plan

1. **Setup:** in a temp dir, register a portfolio + project, create a
   requirement, a story, some tasks (some linked to the story, some not),
   add tags, run `tkr sync`.
2. **CLI — by-requirement:** run `tkr portfolio view --by-requirement`,
   assert the requirement header, count, and task lines appear; assert
   the total line.
3. **CLI — by-story:** run `tkr portfolio view --by-story`, assert the
   story group and the `(none)` group both appear.
4. **CLI — by-tag:** run `tkr portfolio view --by-tag=security`, assert
   only tasks with that tag appear, grouped by project.
5. **CLI — by-project:** run `tkr portfolio view --by-project`, assert
   each project header + its tasks.
6. **CLI — empty group:** create a requirement with no tasks, run
   `--by-requirement`, assert the requirement appears with count 0 and an
   empty marker.
7. **CLI — no flag:** run `tkr portfolio view`, assert non-zero exit and
   the error message.
8. **CLI — multiple flags:** run with two flags, assert non-zero exit.
9. **CLI — json:** run `--by-story --json`, assert the output parses as
   JSON with the expected group keys.
10. **Read-only:** confirm no markdown files change after running any
    view (compare mtimes or content hashes before/after).
11. **Regression:** full `tests/cli_tests.rs` suite passes.

## Observability

- Each view prints a total line to stdout for quick histogram reading.
- `RUST_LOG=debug` logs the SQL query and row count per group.
- The `--json` output is stable-shaped for piping into `jq` or other
  tooling.

## Compliance

- No new dependencies.
- No secrets or credentials touched.
- MIT license unaffected.
- Read-only command — no data mutation, safe to run anytime.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| DB not synced / empty → confusing empty output | Medium | Low | Print `No tasks found. Run 'tkr sync' first.` when the `tasks` table is empty. |
| Joins miss tasks with NULL story/requirement | Medium | Medium | Use LEFT JOINs; include explicit `(none)` buckets. Add a test for unlinked tasks. |
| Large portfolios produce noisy output | Low | Low | Acceptable for CLI; future `--limit`/`--state` filters can tame it. |
| `--json` shape changes break consumers | Low | Medium | Version the JSON shape (add a `version` field) and document it. |
| Conflicting flags parse silently | Low | Low | clap `conflicts_with` or manual validation; tested. |

## Dependencies & Sequencing

- **Depends on:** 03-002 (requirement CRUD — `requirements` table),
  03-003 (story CRUD — `stories` table), 03-004 (sync — `tasks` table
  populated). Also benefits from 04-001 (`tasks.story_id`) and 04-002
  (`task_tags`) for the by-story and by-tag views, but can degrade
  gracefully (show empty groups) if those land after.
- **Blocks:** 05-002 (portfolio API — the view queries inform the API
  endpoints), 06-001 (Kanban UI — same groupings).
- **Parallel-safe:** yes — read-only queries against the DB; no writes to
  shared structs. Disjoint from the other Phase 04 stories.

## Definition of Done

- [ ] All acceptance criteria met.
- [ ] New tests added and passing (`devbox run just test-internal`).
- [ ] Lint clean (`devbox run just lint-internal`).
- [ ] Build clean (`devbox run just build-internal`).
- [ ] Read-only behavior verified (no markdown/DB writes).
- [ ] PR describes the "why" (cross-altitude CLI visibility) not just the
      "what".
- [ ] Conventional commit message used.

## STOP Conditions

- Stop and escalate if 03-002, 03-003, or 03-004 is not merged — the views
  query tables those stories create.
- Stop if the db module from 01-001 does not expose a read connection
  helper — this story should not open a second connection path; reuse the
  existing one.
- Stop if the `--by-tag` view cannot be implemented because 04-002 has
  not landed — implement the other three views and stub `--by-tag` with a
  clear "tags not yet synced" message, then escalate.

## Maintenance Notes

- The four view functions are the CLI mirror of the API endpoints in
  05-002. Keep the SQL queries in one module so they can be shared or
  kept in sync with the API handlers.
- When state/assignee filters are added (future), extend the flag set
  rather than adding new subcommands — the grouping flags compose with
  filters.
- The `--json` shape is the first machine-readable CLI output; treat it
  as a public contract and bump the `version` field on breaking changes.

## Commit Conventions

- Branch: `feature/current/portfolio-layer/story-04-003-portfolio-views`
- Commit subject prefix: `feat(portfolio): `
- Suggested commits:
  - `feat(portfolio): add portfolio view module with grouping queries`
  - `feat(portfolio): add tkr portfolio view CLI with four groupings`
  - `feat(portfolio): add --json output for portfolio views`
  - `test(portfolio): cover portfolio view groupings and errors`
