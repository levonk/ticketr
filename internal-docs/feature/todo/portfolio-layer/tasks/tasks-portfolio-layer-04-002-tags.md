---
story_id: "04-002"
story_title: "Tags field in Ticket + tag CLI + tag sync"
story_name: "tags"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 4
parallel_id: 2
branch: "feature/current/portfolio-layer/story-04-002-tags"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["03-004"]
parallel_safe: true
modules: ["ticket", "tag", "sync", "cli"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "backend"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Add cross-cutting tags to tkr tasks. Tags are stored in the per-project
markdown frontmatter (`tags: [security, backend]`) and indexed into the
portfolio SQLite DB (`tags` + `task_tags` tables) so they can be queried
cross-project. Deliver a `tkr tag` CLI with `add`, `remove`, and `list`
subcommands. Tags are the primary cross-cutting filter in the portfolio
Kanban board and a prerequisite for GitHub label sync (06-002).

## Current State

- `src/ticket.rs` `Ticket` struct (line 8) has no `tags` field.
- `src/cli.rs` `Commands` enum (line 25) has no `tag` subcommand.
- The PRD data model (lines 272-283) defines the `tags` and `task_tags`
  tables.
- Story 03-004 delivers `tkr sync` and the `tasks` table; this story adds
  the `tags`/`task_tags` tables and the sync mapping for them.
- The PRD CLI section (lines 485-489) specifies:
  `tkr tag add <task-id> <tag>`, `tkr tag remove <task-id> <tag>`,
  `tkr tag list`, `tkr tag list --tasks`.

## Scope

**In scope:**
- Add `tags: Vec<String>` to the `Ticket` struct in `src/ticket.rs` with
  `#[serde(default)]` for backward compatibility.
- Add `tags` to YAML frontmatter serialization (skip when empty to stay
  backward compatible).
- Update every `Ticket { ... }` literal in `src/ticket.rs` to initialize
  `tags: Vec::new()`.
- Create the `tags` and `task_tags` tables in the portfolio DB (migration
  in the db module from 01-001).
- Extend the sync path (from 03-004) to upsert tags and the task-tag join
  rows from the markdown `tags` field.
- Add `tkr tag add <task-id> <tag>`, `tkr tag remove <task-id> <tag>`,
  `tkr tag list`, and `tkr tag list --tasks` CLI commands.
- `tkr tag add/remove` edit the markdown frontmatter and re-sync the
  affected task.
- `tkr tag list` lists all distinct tags (optionally with task counts).
- `tkr tag list --tasks` lists tags with their associated task IDs.
- Normalize tag names: lowercase, trim, reject empty/whitespace-only.

**Out of scope:**
- Tag colors (the `tags.color` column exists in the schema but is not
  set by the CLI in this story).
- GitHub label sync (06-002).
- Tag-based priority ordering (06-003).
- Web UI tag chips (06-001).

## Sub-Tasks

1. Add `tags: Vec<String>` to `Ticket` in `src/ticket.rs` with
   `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
2. Update all `Ticket { ... }` struct literals in `src/ticket.rs` to set
   `tags: Vec::new()`.
3. Add a `tags` migration to the db module creating the `tags` and
   `task_tags` tables (schema per PRD lines 272-283).
4. Add `add_tag(&self, task_id: &str, tag: &str) -> Result<()>` and
   `remove_tag(&self, task_id: &str, tag: &str) -> Result<()>` methods to
   `TicketManager` that edit the markdown `tags` list and save.
5. Extend the sync upsert: for each task, replace its `task_tags` rows
   with the current markdown `tags` set; upsert any new tag names into
   `tags`.
6. Add a `Tag` subcommand enum to `Commands` in `src/cli.rs` with `Add`,
   `Remove`, `List { tasks: bool }` variants.
7. Wire the new commands in `Commands::execute`.
8. Implement tag normalization (lowercase, trim, dedupe within a ticket).
9. Write integration tests in `tests/cli_tests.rs`.

## Relevant Files

- `src/ticket.rs` — `Ticket` struct (line 8), `create_ticket` (line 680),
  `parse_bash_tk_ticket` (line 580), `fix_misplaced_ticket` (line 376),
  `save_ticket` (line 199).
- `src/cli.rs` — `Commands` enum (line 25), `Commands::execute` (line 130).
- `src/db.rs` (from 01-001) — migration runner; add the tags migration here.
- `src/sync.rs` (from 03-004) — task upsert path; add tag join sync here.
- `tests/cli_tests.rs` — CLI integration test patterns.
- PRD: `internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md`
  (lines 170-173 for frontmatter, lines 272-283 for tag schema, lines
  485-489 for CLI).

## Acceptance Criteria

- [ ] `Ticket` struct has `tags: Vec<String>` with `#[serde(default)]`.
- [ ] Existing tickets without a `tags` field still parse (backward
      compatible).
- [ ] `tkr tag add <task-id> <tag>` adds the tag to the markdown `tags`
      list (normalized to lowercase).
- [ ] `tkr tag add` is idempotent — adding a duplicate tag does not duplicate
      it in the list.
- [ ] `tkr tag remove <task-id> <tag>` removes the tag from the markdown
      list; removing a missing tag is a no-op (with a message).
- [ ] `tkr tag list` prints all distinct tags in the DB, one per line.
- [ ] `tkr tag list --tasks` prints each tag followed by its task IDs.
- [ ] `tkr sync` upserts `tags` rows and reconciles `task_tags` for each
      synced task (adds new, removes stale).
- [ ] Tag names are normalized (lowercase, trimmed) before write.
- [ ] Empty/whitespace tags are rejected with an error.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes with new tests.
- [ ] `devbox run just lint-internal` passes.

## Examples

```bash
# Add tags
tkr tag add ja-6b9a0dc security
# => Added tag "security" to ja-6b9a0dc
tkr tag add ja-6b9a0dc Backend
# => Added tag "backend" to ja-6b9a0dc   (normalized to lowercase)

# Add duplicate (idempotent)
tkr tag add ja-6b9a0dc security
# => Tag "security" already on ja-6b9a0dc

# Remove
tkr tag remove ja-6b9a0dc security
# => Removed tag "security" from ja-6b9a0dc

# List all tags
tkr tag list
# backend
# security

# List tags with tasks
tkr tag list --tasks
# backend: ja-6b9a0dc, ja-7c1b2ee
# security: ja-6b9a0dc

# Reject empty
tkr tag add ja-6b9a0dc "   "
# => Error: tag must not be empty
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
tags: [security, backend]
...
---
```

## Test Plan

1. **Unit — backward compat:** load an existing ticket with no `tags` field;
   assert `ticket.tags == vec![]` and no parse error.
2. **Unit — round trip:** set `tags = vec!["security".into()]`, save, reload,
   assert the field persists.
3. **Unit — normalization:** add `"  Backend "`, assert stored as
   `"backend"`.
4. **Unit — dedupe:** add `"security"` twice, assert the list has one entry.
5. **CLI — add:** create a task, run `tkr tag add <task> security`, assert
   the markdown contains `tags: [security]`.
6. **CLI — remove:** run `tkr tag remove <task> security`, assert the
   `tags` line is gone (or empty and skipped).
7. **CLI — remove missing:** run `tkr tag remove <task> nope`, assert
   success with a "not found" message and unchanged markdown.
8. **CLI — empty reject:** run `tkr tag add <task> ""`, assert non-zero
   exit and error.
9. **CLI — list:** after syncing two tagged tasks, run `tkr tag list`,
   assert both tags appear.
10. **CLI — list --tasks:** run `tkr tag list --tasks`, assert each tag
    lists its task IDs.
11. **Sync — reconcile:** add a tag in markdown, sync, assert `task_tags`
    row exists; remove it in markdown, sync, assert the row is gone.
12. **Regression:** full `tests/cli_tests.rs` suite passes.

## Observability

- `tkr tag add/remove` print a confirmation line to stdout.
- `tkr tag list` output is plain text (one tag per line) for easy piping;
  `--tasks` uses `tag: id1, id2` format.
- The sync path logs (via the 03-004 convention) the number of tag rows
  upserted and `task_tags` rows reconciled per run.
- `RUST_LOG=debug` shows the normalized tag name and the upsert SQL.

## Compliance

- No new dependencies.
- No secrets or credentials touched.
- MIT license unaffected.
- Backward compatibility: `tags` is `#[serde(default)]` and skipped when
  empty, so existing tickets keep working.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Adding a struct field breaks `Ticket { ... }` literals | High | Low | Compiler catches all sites; grep and update each. |
| Sync leaves stale `task_tags` rows after tag removal | Medium | Medium | Sync must delete-then-insert (or diff) `task_tags` per task, not append. Add a test. |
| Tag name collisions after normalization (e.g. "Security" vs "security") | Medium | Low | Normalize before dedupe; document that tags are case-insensitive. |
| Large tag lists blow up frontmatter readability | Low | Low | Acceptable; YAML inline `[a, b, c]` is compact. |
| `tags` table grows unbounded with one-off tags | Low | Low | Acceptable for now; a future cleanup command can prune unused tags. |

## Dependencies & Sequencing

- **Depends on:** 03-004 (markdown sync — the upsert path this story
  extends; also needs the `tasks` table to exist for `task_tags` FK).
- **Blocks:** 05-002 (portfolio API — tag filter endpoints), 06-001
  (Kanban UI tag chips + tag filter), 06-002 (GitHub label sync maps tags
  to labels).
- **Parallel-safe:** yes — touches `ticket.rs` frontmatter + a new `tag`
  CLI + new tables, disjoint from the other Phase 04 stories as long as
  the sync module is extended additively.

## Definition of Done

- [ ] All acceptance criteria met.
- [ ] New tests added and passing (`devbox run just test-internal`).
- [ ] Lint clean (`devbox run just lint-internal`).
- [ ] Build clean (`devbox run just build-internal`).
- [ ] No backward-compatibility regressions for existing tickets.
- [ ] PR describes the "why" (cross-cutting portfolio filter) not just the
      "what".
- [ ] Conventional commit message used.

## STOP Conditions

- Stop and escalate if 03-004 is not merged — this story cannot sync tags
  without the sync engine and the `tasks` table.
- Stop if the db module from 01-001 does not support additive migrations —
  this story needs a new migration, not a schema rewrite.
- Stop if normalization rules (lowercase) conflict with an existing tag
  convention in the repo — escalate to decide casing policy.

## Maintenance Notes

- Tags are the second portfolio-level field on `Ticket` (after `story`).
  Keep the sync mapping grouped with the other task-column mappings.
- The `tags.color` column is reserved for future UI use; do not set it
  from the CLI in this story.
- If GitHub label sync (06-002) maps tags to labels, keep the
  normalization rule identical across both paths to avoid drift.
- A future `tkr tag rename` command would need to update markdown in every
  ticket carrying the old tag — note this for the maintainer.

## Commit Conventions

- Branch: `feature/current/portfolio-layer/story-04-002-tags`
- Commit subject prefix: `feat(portfolio): `
- Suggested commits:
  - `feat(portfolio): add tags field to Ticket frontmatter`
  - `feat(portfolio): add tags + task_tags tables and migration`
  - `feat(portfolio): add tkr tag add/remove/list commands`
  - `feat(portfolio): sync tags and task_tags from markdown`
  - `test(portfolio): cover tag add/remove/list and sync`
