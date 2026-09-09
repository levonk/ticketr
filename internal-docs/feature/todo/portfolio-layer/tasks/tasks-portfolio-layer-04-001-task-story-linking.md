---
story_id: "04-001"
story_title: "Link tasks to stories (markdown frontmatter + DB sync)"
story_name: "task-story-linking"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 4
parallel_id: 1
branch: "feature/current/portfolio-layer/story-04-001-task-story-linking"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["03-001", "03-003", "03-004"]
parallel_safe: true
modules: ["ticket", "story", "sync", "cli"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Add a `story` field to the per-project `Ticket` markdown frontmatter so individual
tasks can be linked to a portfolio-level Story (the coordination layer in the
7-level hierarchy). Wire the field through YAML serialization, the sync path
that indexes markdown into the SQLite portfolio DB (`tasks.story_id`), and a
new `tkr story link` / `tkr story unlink` CLI command pair. This is the bridge
that lets the Kanban board group tasks by story and enables the
cross-altitude priority rule from the PRD.

## Current State

- `src/ticket.rs` defines `Ticket` with fields up through `notes` (line 36).
  There is no `story` field.
- `src/cli.rs` has no `story` subcommand; the `Commands` enum (line 25) covers
  create/start/close/list/show/web/tui but not portfolio-level story
  management.
- Story 03-003 delivers the `story` CRUD commands (`create/list/show/ship`)
  and the `stories` SQLite table. This story consumes that table.
- Story 03-004 delivers `tkr sync` which indexes markdown into the `tasks`
  table. This story extends that sync to populate `tasks.story_id`.
- The PRD data model (lines 242-258) defines `tasks.story_id TEXT REFERENCES
  stories(id)` and the markdown frontmatter addition `story: story-abc123`
  (line 174).

## Scope

**In scope:**
- Add `story: Option<String>` to the `Ticket` struct in `src/ticket.rs`.
- Add `story` to YAML frontmatter serialization (skip when `None` to stay
  backward compatible with existing tickets).
- Update every `Ticket { ... }` literal in `src/ticket.rs` to initialize the
  new field (`create_ticket`, `parse_bash_tk_ticket`, `fix_misplaced_ticket`).
- Extend the sync path (from 03-004) so `tasks.story_id` is written/updated
  from the markdown `story` field during `tkr sync`.
- Add `tkr story link <task-id> <story-id>` and `tkr story story unlink
  <task-id>` CLI commands that edit the markdown frontmatter and re-sync the
  affected task.
- Validate that `<story-id>` exists in the `stories` table before linking
  (fail with a clear error if not).

**Out of scope:**
- The `stories` table itself (owned by 03-003).
- The sync engine itself (owned by 03-004); this story only adds a column
  mapping.
- Web UI grouping by story (owned by 06-001).
- Priority ordering within a story scope (owned by 06-003).

## Sub-Tasks

1. Add `story: Option<String>` field to `Ticket` in `src/ticket.rs` with
   `#[serde(skip_serializing_if = "Option::is_none")]`.
2. Update all `Ticket { ... }` struct literals in `src/ticket.rs` to set
   `story: None` (or `story: self.story.clone()` if a manager-level default
   is introduced).
3. Add a `link_story(&self, task_id: &str, story_id: &str) -> Result<()>`
   method to `TicketManager` that loads the ticket, sets `story`, and saves.
4. Add an `unlink_story(&self, task_id: &str) -> Result<()>` method that
   clears the `story` field and saves.
5. Extend the sync mapping (in the sync module from 03-004) to write
   `tasks.story_id = ticket.story` on upsert.
6. Add `Story` subcommand variants `Link { task_id, story_id }` and `Unlink
   { task_id }` to `Commands` in `src/cli.rs` (or extend the `Story` enum from
   03-003).
7. Wire the new commands in `Commands::execute` to call the new
   `TicketManager` methods and trigger a single-task re-sync.
8. Add a story-existence check before linking (query `stories` table).
9. Write integration tests in `tests/cli_tests.rs`.

## Relevant Files

- `src/ticket.rs` — `Ticket` struct (line 8), `create_ticket` (line 680),
  `parse_bash_tk_ticket` (line 580), `fix_misplaced_ticket` (line 376),
  `save_ticket` (line 199).
- `src/cli.rs` — `Commands` enum (line 25), `Commands::execute` (line 130).
- `src/sync.rs` (or equivalent from 03-004) — markdown-to-SQLite upsert path.
- `src/db.rs` (or equivalent from 01-001) — `tasks` table schema, story
  lookup helper.
- `tests/cli_tests.rs` — CLI integration test patterns.
- PRD: `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md`
  (lines 170-175 for frontmatter, lines 242-258 for `tasks` schema).

## Acceptance Criteria

- [ ] `Ticket` struct has `story: Option<String>` with
      `#[serde(skip_serializing_if = "Option::is_none")]`.
- [ ] Existing tickets without a `story` field still parse (backward
      compatible).
- [ ] `tkr story link <task-id> <story-id>` writes `story: <story-id>` into
      the task's markdown frontmatter.
- [ ] `tkr story unlink <task-id>` removes the `story` field from the
      markdown frontmatter.
- [ ] Linking to a non-existent story ID fails with a clear error and does
      not modify the markdown.
- [ ] `tkr sync` populates `tasks.story_id` from the markdown `story` field.
- [ ] Re-running `tkr sync` after unlinking clears `tasks.story_id` (sets to
      NULL).
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes with new tests.
- [ ] `devbox run just lint-internal` passes.

## Examples

```bash
# Link a task to a story
tkr story link ja-6b9a0dc story-abc123
# => Updated ja-6b9a0dc -> story: story-abc123

# Unlink
tkr story unlink ja-6b9a0dc
# => Cleared story link for ja-6b9a0dc

# Link to a missing story
tkr story link ja-6b9a0dc story-doesnotexist
# => Error: story "story-doesnotexist" not found in portfolio DB

# After sync, the DB reflects the link
tkr sync
# SQLite: tasks.story_id = "story-abc123" for ja-6b9a0dc
```

Frontmatter before/after:

```yaml
# before
---
id: ja-6b9a0dc
title: Add rusqlite dependency
status: open
...
---

# after
---
id: ja-6b9a0dc
title: Add rusqlite dependency
status: open
story: story-abc123
...
---
```

## Test Plan

1. **Unit — backward compat:** load an existing ticket file with no `story`
   field; assert `ticket.story == None` and no parse error.
2. **Unit — round trip:** set `story = Some("story-abc123".into())`, save,
   reload, assert the field persists.
3. **CLI — link:** create a story (via 03-003), create a task, run
   `tkr story link <task> <story>`, assert the markdown file contains
   `story: <story>`.
4. **CLI — unlink:** run `tkr story unlink <task>`, assert the `story` line
   is gone from the markdown.
5. **CLI — missing story:** run `tkr story link <task> story-nope`, assert
   non-zero exit and error message, assert markdown unchanged.
6. **Sync — link propagates:** after linking, run `tkr sync`, query
   `tasks.story_id` from the DB, assert it equals the story ID.
7. **Sync — unlink propagates:** after unlinking, run `tkr sync`, assert
   `tasks.story_id` is NULL.
8. **Regression:** run the full existing `tests/cli_tests.rs` suite to
   confirm no breakage from the new struct field.

## Observability

- `tkr story link` and `tkr story unlink` print a confirmation line to
  stdout (`Updated <id> -> story: <story>` / `Cleared story link for <id>`).
- The sync path logs (via the 03-004 logging convention) when a task's
  `story_id` changes between sync runs.
- `RUST_LOG=debug` should show the story lookup query and the upserted
  `story_id` value.

## Compliance

- No new dependencies.
- No secrets or credentials touched.
- MIT license unaffected.
- Backward compatibility: the `story` field is optional and skipped when
  `None`, so existing tickets and any tooling that parses the frontmatter
  keep working.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Adding a struct field breaks `Ticket { ... }` literals elsewhere | High | Low | Compiler catches all sites; grep for `Ticket {` and update each. |
| Sync writes stale `story_id` after unlink | Medium | Medium | Sync must upsert `NULL` when markdown `story` is absent, not skip the column. Add a test for this case. |
| Linking to a deleted story leaves dangling `story_id` | Medium | Medium | Validate story existence at link time; on sync, warn (not fail) if `story_id` references a missing row. |
| `serde_yaml` reorders frontmatter keys | Low | Low | Acceptable — key order is not contractual; tests assert on parsed values, not raw text. |

## Dependencies & Sequencing

- **Depends on:** 03-001 (app CRUD — tasks reference `app_id`), 03-003
  (story CRUD — `stories` table + `tkr story` subcommand exists), 03-004
  (markdown sync — the upsert path this story extends).
- **Blocks:** 05-002 (portfolio API — needs `story_id` on tasks for
  grouping/filtering), 06-001 (Kanban UI group-by-story).
- **Parallel-safe:** yes — touches `ticket.rs` frontmatter + sync mapping,
  which are disjoint from the other Phase 04 stories (tags, views, AI tasks)
  as long as the sync module is extended additively.

## Definition of Done

- [ ] All acceptance criteria met.
- [ ] New tests added and passing (`devbox run just test-internal`).
- [ ] Lint clean (`devbox run just lint-internal`).
- [ ] Build clean (`devbox run just build-internal`).
- [ ] No backward-compatibility regressions for existing tickets.
- [ ] PR describes the "why" (cross-altitude grouping) not just the "what".
- [ ] Conventional commit message used.

## STOP Conditions

- Stop and escalate if 03-003 or 03-004 is not merged — this story cannot be
  implemented without the `stories` table and the sync engine.
- Stop if adding the `story` field to `Ticket` causes a serde regression in
  the TUI or web paths — escalate to decide whether to gate the field behind
  a feature flag or fix the consumers.
- Stop if the sync module from 03-004 does not expose a per-task upsert hook
  — this story needs to extend the upsert, not rewrite it.

## Maintenance Notes

- The `story` field is the first portfolio-level link on the `Ticket` struct.
  Future links (requirement, app) should follow the same pattern: optional
  `Option<String>`, skip-when-none serialization, validated at link time,
  synced as a column on `tasks`.
- If the `stories` table schema changes (e.g. ID format), the link command's
  validation query must be updated in lockstep.
- The sync mapping for `story_id` lives next to the other task-column
  mappings; keep it grouped so future column additions are obvious.

## Commit Conventions

- Branch: `feature/current/portfolio-layer/story-04-001-task-story-linking`
- Commit subject prefix: `feat(portfolio): `
- Suggested commits:
  - `feat(portfolio): add story field to Ticket frontmatter`
  - `feat(portfolio): add tkr story link/unlink commands`
  - `feat(portfolio): sync tasks.story_id from markdown`
  - `test(portfolio): cover task-story linking and sync`
