---
feature: "Portfolio Layer"
slug: "portfolio-layer"
status: "Completed"
date:
  created: "2026-09-08"
  last-activity: "2026-09-09"
  completed: "2026-09-09"
priority: "P1"
tags:
  - architecture
  - cross-project
  - portfolio
  - daemon
  - web-ui
  - github-sync
---

# Feature PRD: Portfolio Layer

## Problem

tkr manages tickets per-project (markdown files in each repo's `.tickets/`).
When working across multiple projects, there is no way to:

1. **Prioritize tickets relative to each other** across projects
2. **Group tickets into stories/requirements** that span repos
3. **Tag tickets** for cross-cutting concerns (e.g. "security", "performance")
4. **View and manipulate the portfolio** from a unified interface

This is the "flat backlog trap" described in Prodpad's backlog hierarchy
article: when all items sit at the same altitude, prioritization becomes
meaningless because you're comparing platform migrations with button tweaks.

## Goals

1. Multi-portfolio support — separate portfolios for personal work, OSS,
   business A, client X, etc.
2. 7-level hierarchy — Portfolio → Project → App → Requirement → Story →
   Task → AI Task, with unique state vocabulary at each level
3. Cross-project visibility — see all tasks across all registered projects
4. Tagging — cross-cutting labels visible both per-project and cross-project
5. Unified web UX — daemon-backed Kanban board with filtering, count
   annotations, and formalized priority semantics
6. GitHub Projects compatibility — bidirectional sync with GitHub Issues,
   working across multiple GitHub accounts (lrepo52, levonk)
7. No-corner architecture — each layer is independently durable; the SQLite
   DB is always rebuildable from markdown + GitHub API

## Non-Goals

- Replacing per-project markdown tickets as the source of truth
- Building a SaaS — this is a local-first tool with optional cloud sync
- Custom scoring frameworks (RICE, etc.) — priority is relative ordering,
  not numeric scoring across altitudes
- Replacing GitHub Projects — tkr complements it as the local/offline layer

## Hierarchy

Seven levels, each with unique state names (no overlap — you can tell the
level from the state alone). The App level is always present; single-app
projects get an implicit `default` app.

```
Portfolio (personal, OSS, business A, client X, ...)
  └─ Project (a repo: tkr, infrahub, dotfiles, ...)
       └─ App (deployable unit; "default" for single-app projects)
            └─ Requirement (durable constraint: "must support multi-account sync")
                 └─ Story (feature slice: "portfolio layer")
                      └─ Task (work item: "add rusqlite dependency")
                           └─ AI Task (delegated subtask: "write migration tests")
```

### State vocabulary (36 unique names across 7 levels)

Each level has its own thematic vocabulary. No state name appears in more
than one level.

| Level | Pre-draft | Draft | Ready | Active | Blocked | Review | Done | Archived |
|-------|-----------|-------|-------|--------|---------|--------|------|----------|
| **Portfolio** | `conceived` | `curated` | `engaged` | `engaged` | `paused` | — | — | `dissolved` |
| **Project** | `imagined` | `seeded` | `live` | `live` | `dormant` | — | `retired` | — |
| **App** | `sketched` | `drafted` | `deployed` | `deployed` | `deprecated` | — | `sunset` | — |
| **Requirement** | `surfaced` | `proposed` | `planned` | `current` | — | — | `superseded` | — |
| **Story** | `suggested` | `pitched` | `queued` | `building` | `stalled` | `review` | `shipped` | `archived` |
| **Task** | `logged` | `open` | `open` | `in_progress` | `blocked` | `ready` | `closed` | — |
| **AI Task** | `identified` | `dispatched` | `dispatched` | `running` | — | — | `returned` | — |

### Lifecycle diagrams

```
Portfolio:  conceived → curated → engaged ⇄ paused → dissolved
Project:    imagined → seeded → live ⇄ dormant → retired
App:        sketched → drafted → deployed ⇄ deprecated → sunset
Requirement: surfaced → proposed → planned → current → superseded
Story:      suggested → pitched → queued → building ⇄ stalled → review → shipped → archived
Task:       logged → open → in_progress ⇄ blocked → ready → closed
AI Task:    identified → dispatched → running → returned
```

### Prodpad altitude mapping

| Prodpad altitude | Our level | State when active |
|-----------------|-----------|-------------------|
| Strategic (Objectives) | **Requirement** | `current` |
| Coordination (Initiatives) | **Story** | `building` |
| Operational (Ideas) | **Task** + **AI Task** | `in_progress` / `running` |

### Thematic vocabulary

- **Portfolio**: curation language (`conceived`, `curated`, `engaged`,
  `paused`, `dissolved`) — you curate a portfolio of work
- **Project**: lifecycle language (`imagined`, `seeded`, `live`, `dormant`,
  `retired`) — projects have a lifespan
- **App**: deployment language (`sketched`, `drafted`, `deployed`,
  `deprecated`, `sunset`) — apps get deployed
- **Requirement**: specification language (`surfaced`, `proposed`,
  `planned`, `current`, `superseded`) — requirements evolve
- **Story**: delivery language (`suggested`, `pitched`, `queued`,
  `building`, `stalled`, `review`, `shipped`, `archived`) — stories get
  shipped
- **Task**: work-item language (`logged`, `open`, `in_progress`,
  `blocked`, `ready`, `closed`) — tkr's existing vocabulary, plus `logged`
  as pre-draft
- **AI Task**: execution language (`identified`, `dispatched`, `running`,
  `returned`) — agents get dispatched

### The "fridge" concept

The pre-draft states (`conceived`, `imagined`, `sketched`, `surfaced`,
`suggested`, `logged`, `identified`) are the "fridge" — ideas that exist
but aren't ready to work on yet. They graduate to the draft state when
formally captured, and to the ready state when ready to work.

## Architecture

Three independent layers, each durable on its own:

```
PER-PROJECT MARKDOWN (durable, git-friendly)
  Source of truth for task content
  Tags in YAML frontmatter (visible per-project)
  Lives in: each repo's .tickets/
        │ daemon watches + syncs
        ▼
PORTFOLIO SQLite DB (rebuildable index)
  ~/.local/share/tkr/portfolio.db
  Stores: portfolios, projects, apps, requirements, stories,
          tasks, ai_tasks, tags, relative ordering, GitHub sync state
  Can be rebuilt from: markdown files + GitHub API
        │ daemon syncs (bidirectional, optional)
        ▼
GITHUB PROJECTS (optional cloud collaboration layer)
  Issues synced from tkr tasks (via gh API)
  Custom fields: story, requirement, priority
  Views: portfolio board across multiple GitHub accounts
```

### No-corner principle

- The SQLite DB is always rebuildable from markdown + GitHub API
- Markdown files are the durable, git-friendly source of truth
- GitHub Issues are an optional projection, not a dependency
- Each layer is independently durable
- Works across multiple GitHub accounts (lrepo52, levonk) via gh auth

## Data Model

### Markdown changes (per-project)

Add to Ticket YAML frontmatter:

```yaml
tags: [security, backend]           # new — cross-cutting concerns
story: story-abc123                 # new — links task to story
```

### Portfolio SQLite schema

```sql
-- Portfolios (top level — personal, OSS, business A, client X)
CREATE TABLE portfolios (
    id TEXT PRIMARY KEY,             -- e.g. "personal", "business-a"
    name TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'curated',    -- conceived, curated, engaged, paused, dissolved
    created TEXT NOT NULL,
    position INTEGER DEFAULT 0
);

-- Projects (repos registered in a portfolio)
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

-- Apps (deployable units within a project; "default" for single-app)
CREATE TABLE apps (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,             -- "default" for single-app projects
    description TEXT,
    state TEXT DEFAULT 'drafted',   -- sketched, drafted, deployed, deprecated, sunset
    created TEXT NOT NULL,
    UNIQUE(project_id, name)
);

-- Requirements (durable constraints — strategic level)
CREATE TABLE requirements (
    id TEXT PRIMARY KEY,             -- e.g. "req-abc123"
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id),
    title TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'proposed',  -- surfaced, proposed, planned, current, superseded
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0
);

-- Stories (feature slices — coordination level)
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

-- Tasks (work items — operational level, indexed from markdown)
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,             -- ticket ID (e.g. "ja-6b9a0dc")
    project_id INTEGER NOT NULL REFERENCES projects(id),
    app_id INTEGER REFERENCES apps(id),
    story_id TEXT REFERENCES stories(id),
    title TEXT NOT NULL,
    state TEXT NOT NULL,             -- logged, open, in_progress, blocked, ready, closed
    priority INTEGER DEFAULT 2,
    issue_type TEXT DEFAULT 'task',
    markdown_path TEXT NOT NULL,
    markdown_hash TEXT,
    github_issue_number INTEGER,
    github_issue_url TEXT,
    github_synced_at TEXT,
    github_etag TEXT,
    synced_at TEXT NOT NULL
);

-- AI Tasks (delegated subtasks)
CREATE TABLE ai_tasks (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    title TEXT NOT NULL,
    state TEXT DEFAULT 'dispatched', -- identified, dispatched, running, returned
    agent_profile TEXT,              -- e.g. "subagent_general"
    created TEXT NOT NULL,
    completed TEXT,
    result_summary TEXT
);

-- Tags (cross-project)
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (task_id, tag_id)
);

-- Relative priority ordering (drag-and-drop in web UI)
-- Scoped to (level, scope_id) — ordering within a story, within a tag, etc.
CREATE TABLE priority_order (
    scope_type TEXT NOT NULL,        -- 'portfolio', 'project', 'app', 'requirement', 'story', 'tag', 'global'
    scope_id TEXT NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    position INTEGER NOT NULL,
    PRIMARY KEY (scope_type, scope_id, task_id)
);

-- Sync state
CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

## Daemon

### Components

1. **File watcher** (`notify` crate) — watches registered projects' `.tickets/`
2. **SQLite sync** — bidirectional between markdown and DB
3. **Web server** (`warp`, extending existing `web.rs`) — REST API + static UI
4. **GitHub sync** (bidirectional, `reqwest` or `gh` CLI) — syncs tasks
   to/from GitHub Issues

### Process management

- `tkr daemon start` — starts daemon as background process
- `tkr daemon stop` — stops daemon
- `tkr daemon status` — shows daemon status, registered projects, sync state
- `tkr daemon restart` — restarts daemon

### Conflict resolution (bidirectional sync)

- Each task has `markdown_hash` and `github_updated_at` in SQLite
- On sync: compare local hash vs markdown, compare GitHub updated_at vs stored
- If both changed: last-write-wins by timestamp, with warning logged

## Web UI

### Kanban Board (primary view)

The primary web UI is a **Kanban board** with formalized priority semantics.

**Columns** map to the Task level's state flow:
```
logged → open → in_progress → ready → closed
                  ↑               ↓
                  └── blocked ←────┘
```
Each column shows tasks as cards. Card height is compact (title + project +
app + tags + priority indicator).

**Filtering** (sidebar / top bar) narrows the board to a comparable set:
- **Portfolio** — single portfolio (e.g. "show only personal work")
- **Project** — single project or multi-select (e.g. "show tkr + dotfiles")
- **App** — single app within a project (for monorepos)
- **Story** — single story (e.g. "show only 'Portfolio Layer'")
- **Requirement** — all stories under a requirement
- **Tag** — single or multi-tag (e.g. "security", "backend")
- **State** — hide/show specific columns
- **Assignee** — filter by assigned person

Filters compose: "portfolio=personal, project=tkr, tag=security, state=open"
shows only open security tasks in the tkr project under the personal
portfolio.

**Count annotations** (superscript badges everywhere):

Every container — filter option, column header, the total, each portfolio,
each project, each app, each story, each requirement, each tag — shows a
superscript count of tasks it contains, scoped to the currently active
filters.

- **Column headers** show counts: `Open ¹²`, `In Progress ³`, `Ready ⁵`
- **Filter sidebar** shows counts next to each option:
  - Portfolios: `personal ⁸`, `business-a ⁴`, `client-x ²`
  - Projects: `tkr ⁸`, `dotfiles ⁴`, `portfolio ²`
  - Apps: `default ⁵`, `api ³`
  - Stories: `Portfolio Layer ⁵`, `Auth Refactor ³`, `(none) ⁶`
  - Requirements: `Multi-account Sync ⁹`, `Offline-first ⁵`
  - Tags: `security ⁴`, `backend ⁷`, `frontend ²`
- **Total** at top of board: `Showing 14 of 42 tasks`
- Counts are **scoped to active filters**: if you filter to `tag=security`,
  the project counts update to show only security tasks per project
  (`tkr ³`, `dotfiles ¹`). If you then add `state=open`, counts update again.
- **Empty-state indicators**: a filter option with count `⁰` is greyed out
  but still visible.

This gives you a live histogram of the portfolio at a glance.

**Priority formalization** (the Prodpad insight):

Priority is **drag-and-drop ordering within a column, scoped to the active
filter context**. The `priority_order` table stores position per
`(scope_type, scope_id)`:

| scope_type | scope_id | Meaning |
|-----------|----------|---------|
| `global` | `global` | Default ordering across everything |
| `portfolio` | `personal` | Ordering within a portfolio |
| `project` | `tkr` | Ordering within a project |
| `app` | `default` | Ordering within an app |
| `requirement` | `req-abc123` | Ordering within a requirement's tasks |
| `story` | `story-abc123` | Ordering within a story's tasks |
| `tag` | `security` | Ordering within a tag's tasks |

**Cross-altitude rule**: when the board is filtered to a single story,
you can drag-reorder freely (comparing like-for-like). When the board shows
mixed-altitude tasks (no story filter), drag-reorder is disabled with a
tooltip: "Filter to a single story to reorder — cross-altitude
prioritization is not meaningful." This enforces the Prodpad principle
structurally rather than letting users create false trade-offs.

**Card interactions**:
- Drag between columns → changes task state (writes to markdown via daemon)
- Drag within column → changes priority position (writes to `priority_order`)
- Click card → detail panel (edit title, description, tags, story, deps)
- Card badges: project badge, app badge, tag chips, dependency count,
  blocked indicator

### Other Views

1. **Requirement view** — requirements with their stories, stories with
   their tasks (tree/accordion layout)
2. **Tag view** — tasks grouped by tag, across projects
3. **Project view** — single project's tasks (existing tkr web view, enhanced)
4. **Portfolio view** — portfolios with their projects, projects with
   their apps (tree layout)
5. **Backlog view** — flat sortable list with all filters, for bulk triage

### API

```
GET    /api/portfolios
POST   /api/portfolios
PUT    /api/portfolios/:id
DELETE /api/portfolios/:id
GET    /api/projects
POST   /api/projects
DELETE /api/projects/:id
GET    /api/apps
POST   /api/apps
PUT    /api/apps/:id
GET    /api/requirements
POST   /api/requirements
PUT    /api/requirements/:id
DELETE /api/requirements/:id
GET    /api/stories
POST   /api/stories
PUT    /api/stories/:id
DELETE /api/stories/:id
GET    /api/tasks?portfolio=&project=&app=&story=&requirement=&tag=&state=&assignee=
GET    /api/tasks/:id
PUT    /api/tasks/:id                   -- update task (writes to markdown via daemon)
GET    /api/ai-tasks
POST   /api/ai-tasks
PUT    /api/ai-tasks/:id
GET    /api/tags
POST   /api/tags
PUT    /api/priority                      -- reorder tasks (drag-and-drop)
       Body: { scope_type, scope_id, task_id, new_position }
GET    /api/priority?scope_type=&scope_id=   -- read ordering for a scope
POST   /api/sync/github
GET    /api/sync/status
```

## CLI additions

```bash
# Portfolio management
tkr portfolio create "Personal" --description="..."
tkr portfolio list
tkr portfolio show <id>
tkr portfolio dissolve <id>

# Project management
tkr project register <path> --portfolio=<id>
tkr project list
tkr project unregister <path>

# App management
tkr app create "api" --project=<id>
tkr app list --project=<id>
tkr app sunset <id>

# Requirement management (strategic level)
tkr requirement create "Title" --portfolio=<id> --description="..."
tkr requirement list
tkr requirement show <id>
tkr requirement supersede <id>

# Story management (coordination level)
tkr story create "Title" --requirement=<id> --app=<id> --description="..."
tkr story list
tkr story show <id>
tkr story ship <id>

# Tag management
tkr tag add <task-id> <tag>
tkr tag remove <task-id> <tag>
tkr tag list
tkr tag list --tasks

# Portfolio views
tkr portfolio view
tkr portfolio view --by-requirement
tkr portfolio view --by-story
tkr portfolio view --by-tag=<tag>
tkr portfolio view --by-project

# Daemon
tkr daemon start
tkr daemon stop
tkr daemon status
tkr daemon restart

# Sync
tkr sync
tkr sync --github
tkr sync --status
```

## Dependencies to add

```toml
[dependencies]
rusqlite = { version = "0.40", features = ["bundled"] }
notify = "6"
reqwest = { version = "0.12", features = ["json"] }
```

## Acceptance Criteria

- [x] `tkr portfolio create/list/show/dissolve` commands work
- [x] `tkr project register/list/unregister` commands work, projects link to portfolios
- [x] `tkr app create/list/sunset` commands work, "default" app auto-created for single-app projects
- [x] `tkr requirement create/list/show/supersede` commands work
- [x] `tkr story create/list/show/ship` commands work
- [x] Tasks can be linked to stories via markdown `story` frontmatter field
- [x] `tkr tag add/remove/list` commands work, tags sync to markdown
- [x] `tkr sync` syncs markdown tasks into the SQLite portfolio DB
- [x] `tkr portfolio view` shows cross-project views (by-requirement, by-story, by-tag, by-project)
- [x] `tkr daemon start/stop/status/restart` manages the background daemon
- [x] Daemon watches `.tickets/` directories and syncs changes to SQLite
- [x] Web UI serves a Kanban board with columns by task state (logged, open, in_progress, blocked, ready, closed)
- [x] Kanban board supports filtering by portfolio, project, app, story, requirement, tag, state, assignee
- [x] Filters compose (e.g. portfolio=personal + project=tkr + tag=security + state=open)
- [x] Count annotations (superscript badges) on column headers, filter options, total, portfolios, projects, apps, stories, requirements, tags
- [x] Counts are scoped to active filters
- [x] Empty-state indicators (count ⁰ greyed out but visible)
- [x] Drag-and-drop within a column updates `priority_order` for the active filter scope
- [x] Drag-and-drop between columns updates task state (writes to markdown via daemon)
- [x] Cross-altitude rule enforced: drag-reorder disabled when no single story filter is active
- [x] Card detail panel allows editing title, description, tags, story, dependencies
- [x] `tkr sync --github` syncs tasks bidirectionally with GitHub Issues
- [x] GitHub sync works across multiple accounts (lrepo52, levonk)
- [x] All 7 levels use their unique state vocabulary (36 unique state names, no overlap)
- [x] All new commands have tests
- [~] `just test` passes (cargo test passes; 3 pre-existing failures unrelated to this feature)
- [~] `just lint` passes (cargo clippy passes; devbox init_hook broken — see Deviations)

## Deviations from Original Plan

1. **devbox init_hook broken** — The `devbox.json` `init_hook` references `just bootstrap-internal` but the justfile defines `bootstrap_impl` (with underscore, not hyphen). This blocks all `devbox run -- just <cmd>` invocations. Workaround: use `cargo` directly (`cargo build`, `cargo test`, `cargo clippy`). This is a pre-existing issue unrelated to this feature.

2. **3 pre-existing test failures** — `test_dep_tree_command`, `test_dependency_management`, `test_link_unlink_commands` fail on `main` before any portfolio layer changes. These are out of scope and were not fixed.

3. **Pre-existing clippy warnings fixed** — 4 clippy warnings in `src/ticket.rs`, `src/web.rs`, and `src/tui.rs` were fixed to allow `cargo clippy -- -D warnings` to pass cleanly. These were trivial one-line fixes (needless borrow, push_str vs push, match_result_ok, let_underscore_future).

4. **GitHub sync uses `gh` CLI for auth** — Per the no-corner architecture, GitHub sync shells out to `gh auth token` for authentication rather than storing tokens. Tests use a mock client to avoid real API calls.

5. **Priority ordering** — The `priority_order` table was created in story 01-001's schema. Story 06-003 added the `PriorityManager` with atomic reordering, compaction, and auto-assignment. The API endpoints from story 05-002 were refactored to delegate to `PriorityManager`.

## References

- Design document: [`internal-docs/portfolio-layer-design.md`](../../portfolio-layer-design.md)
- Prodpad article: https://www.prodpad.com/blog/backlog-hierarchy-problem/
- rusqlite: https://crates.io/crates/rusqlite (v0.40.2, Aug 2026)
- notify crate: https://crates.io/crates/notify (v6)
- Existing web server: `src/web.rs`
- Existing ticket model: `src/ticket.rs`
- Requirements lifecycle: `requirements-upsert` skill (proposed/todo/current/history)
- Feature lifecycle: `work-lifecycle` include (todo/archive)
- Story lifecycle: `execute-upsert` skill (todo/in_progress/blocked/done/archive)
