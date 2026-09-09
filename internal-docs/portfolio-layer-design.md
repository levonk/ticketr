# Portfolio Layer Design

## Problem

tkr manages tickets per-project (markdown files in each repo's `.tickets/`).
When working across multiple projects, there's no way to:

1. Prioritize tickets relative to each other across projects
2. Group tickets into initiatives/objectives that span repos
3. Tag tickets for cross-cutting concerns (e.g. "security", "performance")
4. View and manipulate the portfolio from a unified interface

## Influences

Prodpad's "backlog hierarchy" article identifies three altitudes of product
decisions (after Klaus Leopold's flight levels model):

- **Strategic (Objectives)** — "Where are we going and why?" Quarterly cadence.
- **Coordination (Initiatives)** — "What problem are we solving?" Now/Next/Later.
- **Operational (Tickets)** — "How might we solve this?" Continuous.

Key insight: prioritization only works when comparing items at the same
altitude. A platform migration and a button tweak can't share a rubric.
The missing layer in most systems is **initiatives** — the coordination layer
that groups tickets into coherent problem-areas.

## Architecture

Three independent layers, each durable on its own:

```
PER-PROJECT MARKDOWN (durable, git-friendly)
  Source of truth for ticket content
  Tags in YAML frontmatter (visible per-project)
  Lives in: each repo's .tickets/
        │ daemon watches + syncs
        ▼
PORTFOLIO SQLite DB (rebuildable index)
  ~/.local/share/tkr/portfolio.db
  Stores: project registry, objectives, initiatives,
          relative ordering, tag index, GitHub sync state
  Can be rebuilt from: markdown files + GitHub API
        │ daemon syncs (bidirectional, optional)
        ▼
GITHUB PROJECTS (optional cloud collaboration layer)
  Issues synced from tkr tickets (via gh API)
  Custom fields: initiative, objective, priority
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
initiative: init-abc123              # new — links ticket to initiative
```

### Portfolio SQLite schema

```sql
-- Project registry
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    repo_path TEXT NOT NULL UNIQUE,
    github_owner TEXT,
    github_repo TEXT,
    github_account TEXT,            -- which gh auth to use
    tickets_dir TEXT NOT NULL,      -- path to .tickets/
    registered_at TEXT NOT NULL,
    last_synced_at TEXT
);

-- Strategic level
CREATE TABLE objectives (
    id TEXT PRIMARY KEY,             -- e.g. "obj-2026q1-revenue"
    title TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'active',    -- active, achieved, abandoned
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0       -- relative ordering
);

-- Coordination level
CREATE TABLE initiatives (
    id TEXT PRIMARY KEY,             -- e.g. "init-abc123"
    title TEXT NOT NULL,
    description TEXT,
    objective_id TEXT REFERENCES objectives(id),
    status TEXT DEFAULT 'open',      -- open, in_progress, done, abandoned
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0       -- relative ordering within objective
);

-- Operational level (index of tickets from all projects)
CREATE TABLE tickets (
    id TEXT PRIMARY KEY,             -- ticket ID (e.g. "ja-6b9a0dc")
    project_id INTEGER NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    priority INTEGER DEFAULT 2,
    issue_type TEXT DEFAULT 'task',
    initiative_id TEXT REFERENCES initiatives(id),
    markdown_path TEXT NOT NULL,
    markdown_hash TEXT,             -- for change detection
    github_issue_number INTEGER,
    github_issue_url TEXT,
    github_synced_at TEXT,
    github_etag TEXT,
    synced_at TEXT NOT NULL
);

-- Tags (cross-project)
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT                       -- optional, for web UI
);

CREATE TABLE ticket_tags (
    ticket_id TEXT NOT NULL REFERENCES tickets(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (ticket_id, tag_id)
);

-- Relative priority ordering (drag-and-drop in web UI)
-- Separated from tickets table because ordering is portfolio-level,
-- not ticket-level. A ticket has different positions in different views.
CREATE TABLE priority_order (
    scope_type TEXT NOT NULL,        -- 'objective', 'initiative', 'tag', 'global'
    scope_id TEXT NOT NULL,          -- the objective/initiative/tag ID, or 'global'
    ticket_id TEXT NOT NULL REFERENCES tickets(id),
    position INTEGER NOT NULL,
    PRIMARY KEY (scope_type, scope_id, ticket_id)
);

-- Sync state (cursors, etags, config)
CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

## Daemon

### Components

1. **File watcher** (`notify` crate) — watches all registered projects' `.tickets/` directories. On change: reparse markdown, update SQLite.
2. **SQLite sync** — bidirectional between markdown and DB. Tags written to DB also written to markdown frontmatter. Initiative assignment written to both.
3. **Web server** (`warp`, extending existing `web.rs`) — REST API + static web UI.
4. **GitHub sync** (bidirectional, `reqwest` or `gh` CLI) — syncs tickets to/from GitHub Issues. Maps tags→labels, priority→custom field, initiative→custom field. Configurable interval.

### Process management

- `tkr daemon start` — starts daemon as background process (PID file in `~/.local/share/tkr/daemon.pid`)
- `tkr daemon stop` — stops daemon
- `tkr daemon status` — shows daemon status, registered projects, sync state
- `tkr daemon restart` — restarts daemon

### Conflict resolution (bidirectional sync)

- Each ticket has `markdown_hash` and `github_updated_at` in SQLite
- On sync: compare local hash vs markdown, compare GitHub updated_at vs stored
- If both changed: last-write-wins by timestamp, with warning logged
- Future: manual conflict resolution UI in web interface

## Web UI

### Views

1. **Portfolio board** — all tickets across all projects, grouped by initiative, drag-and-drop prioritization
2. **Objective view** — objectives with their initiatives, initiatives with their tickets
3. **Tag view** — tickets filtered by tag, across projects
4. **Project view** — single project's tickets (existing tkr web view, enhanced)
5. **Backlog view** — flat list with sorting/filtering by all dimensions

### API

```
GET    /api/portfolio                    -- full portfolio state
GET    /api/projects                     -- registered projects
POST   /api/projects                     -- register a project
DELETE /api/projects/:id                  -- unregister a project
GET    /api/objectives                    -- list objectives
POST   /api/objectives                    -- create objective
PUT    /api/objectives/:id                -- update objective
DELETE /api/objectives/:id                -- delete objective
GET    /api/initiatives                   -- list initiatives
POST   /api/initiatives                   -- create initiative
PUT    /api/initiatives/:id               -- update initiative
DELETE /api/initiatives/:id               -- delete initiative
GET    /api/tickets                        -- all tickets (with filters)
GET    /api/tickets/:id                   -- single ticket
PUT    /api/tickets/:id                   -- update ticket (writes to markdown via daemon)
GET    /api/tags                          -- all tags
POST   /api/tags                          -- create tag
PUT    /api/priority                       -- reorder tickets (drag-and-drop)
POST   /api/sync/github                   -- trigger GitHub sync
GET    /api/sync/status                   -- sync status
```

## CLI additions

```bash
# Project management
tkr project register <path>           -- register a project in portfolio
tkr project list                      -- list registered projects
tkr project unregister <path>        -- remove project from portfolio

# Objective management (strategic level)
tkr objective create "Title" --description="..." --target=2026-03-31
tkr objective list
tkr objective show <id>
tkr objective close <id>

# Initiative management (coordination level)
tkr initiative create "Title" --objective=<id> --description="..."
tkr initiative list
tkr initiative show <id>
tkr initiative close <id>

# Tag management
tkr tag add <ticket-id> <tag>
tkr tag remove <ticket-id> <tag>
tkr tag list
tkr tag list --tickets                 -- show tickets per tag

# Portfolio views
tkr portfolio                         -- show portfolio summary
tkr portfolio --by-objective           -- group by objective
tkr portfolio --by-initiative          -- group by initiative
tkr portfolio --by-tag=<tag>           -- filter by tag

# Daemon
tkr daemon start                       -- start background daemon
tkr daemon stop                        -- stop daemon
tkr daemon status                      -- show daemon status
tkr daemon restart                     -- restart daemon

# Sync
tkr sync                               -- manual sync (markdown ↔ DB ↔ GitHub)
tkr sync --github                      -- GitHub sync only
tkr sync --status                      -- show sync status
```

## Dependencies to add

```toml
[dependencies]
rusqlite = { version = "0.40", features = ["bundled"] }
notify = "6"              # file watcher
reqwest = { version = "0.12", features = ["json"] }  # GitHub API (or use gh CLI)
```

## Phased implementation

### Phase 1: Portfolio DB + project registration
- Add `rusqlite` dependency
- Create portfolio DB schema (migrations)
- `tkr project register/list/unregister` commands
- `tkr sync` (markdown → SQLite, one direction)
- Tests

### Phase 2: Objectives + Initiatives
- `tkr objective create/list/show/close` commands
- `tkr initiative create/list/show/close` commands
- Link tickets to initiatives (markdown `initiative` field + DB)
- `tkr portfolio` views (by-objective, by-initiative)
- Tests

### Phase 3: Tags
- Add `tags` field to Ticket struct + markdown frontmatter
- `tkr tag add/remove/list` commands
- Tag sync (markdown ↔ DB)
- `tkr portfolio --by-tag` view
- Tests

### Phase 4: Daemon + web UI
- `notify` file watcher
- `tkr daemon start/stop/status/restart`
- Extend `web.rs` with portfolio API endpoints
- Static web UI (portfolio board, drag-and-drop)
- `tkr sync` becomes continuous (daemon-driven)

### Phase 5: Bidirectional GitHub sync
- GitHub Issues sync (bidirectional)
- Multi-account support (gh auth switch)
- Tag → label mapping
- Initiative → custom field mapping
- Conflict resolution (last-write-wins with logging)
- Tests

### Phase 6: Relative priority ordering
- `priority_order` table
- Drag-and-drop in web UI
- `tkr portfolio --priority` view (ordered list)
- CLI commands for reordering
