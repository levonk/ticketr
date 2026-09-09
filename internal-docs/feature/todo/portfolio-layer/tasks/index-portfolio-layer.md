# Task Index: Portfolio Layer

PRD: [feat-202609081145-portfolio-layer.md](../feat-202609081145-portfolio-layer.md)

| Story ID | Title | Phase | Status | Assignee | Parallel-safe | Dependencies | Dependants | Modules | Branch |
|---|---|---:|---|---|---|---|---|---|---|
| 01-001 | Add rusqlite dependency + portfolio DB module with schema migrations | 01 | [x] Done |  | true | — | 02-001, 02-002 | db, schema | feature/current/portfolio-layer/story-01-001-portfolio-db-foundation |
| 02-001 | Portfolio CRUD commands (create, list, show, dissolve) | 02 | [x] Done |  | true | 01-001 | 03-002, 03-003, 04-003 | cli, portfolio | feature/current/portfolio-layer/story-02-001-portfolio-crud |
| 02-002 | Project registration CLI commands + link to portfolio | 02 | [x] Done |  | true | 01-001 | 03-001, 03-004, 04-003 | cli, project | feature/current/portfolio-layer/story-02-002-project-registration |
| 03-001 | App CRUD commands + auto-create default app | 03 | [x] Done |  | true | 02-002 | 04-001 | cli, app | feature/current/portfolio-layer/story-03-001-app-crud |
| 03-002 | Requirement CRUD commands (create, list, show, supersede) | 03 | [x] Done |  | true | 02-001 | 04-003 | cli, requirement | feature/current/portfolio-layer/story-03-002-requirement-crud |
| 03-003 | Story CRUD commands (create, list, show, ship) | 03 | [x] Done |  | true | 02-001 | 04-001, 04-003 | cli, story | feature/current/portfolio-layer/story-03-003-story-crud |
| 03-004 | Markdown to SQLite sync (tkr sync) | 03 | [x] Done |  | true | 02-002 | 04-001, 04-002, 05-001 | sync, db | feature/current/portfolio-layer/story-03-004-markdown-sync |
| 04-001 | Link tasks to stories (markdown frontmatter + DB sync) | 04 | [x] Done |  | true | 03-001, 03-003, 03-004 | 05-002 | ticket, story | feature/current/portfolio-layer/story-04-001-task-story-linking |
| 04-002 | Tags field in Ticket + tag CLI + tag sync | 04 | [ ] Todo |  | true | 03-004 | 05-002 | ticket, tag | feature/current/portfolio-layer/story-04-002-tags |
| 04-003 | Portfolio views CLI (by-requirement, by-story, by-tag, by-project) | 04 | [ ] Todo |  | true | 03-002, 03-003, 03-004 | 05-002 | cli, views | feature/current/portfolio-layer/story-04-003-portfolio-views |
| 04-004 | AI Task CRUD | 04 | [ ] Todo |  | true | 03-004 | 05-002 | cli, ai-task | feature/current/portfolio-layer/story-04-004-ai-task-crud |
| 05-001 | File watcher + daemon process management | 05 | [ ] Todo |  | true | 03-004 | 06-002 | daemon, watcher | feature/current/portfolio-layer/story-05-001-daemon-watcher |
| 05-002 | Portfolio API endpoints (extending web.rs) | 05 | [ ] Todo |  | false | 04-001, 04-002, 04-003, 04-004 | 06-001, 06-003 | web, api | feature/current/portfolio-layer/story-05-002-portfolio-api |
| 06-001 | Static web UI (Kanban board, drag-and-drop, count annotations, filtering) | 06 | [ ] Todo |  | true | 05-002 | — | web, ui | feature/current/portfolio-layer/story-06-001-kanban-web-ui |
| 06-002 | GitHub Issues bidirectional sync (multi-account, tag to label) | 06 | [ ] Todo |  | true | 05-001 | — | sync, github | feature/current/portfolio-layer/story-06-002-github-sync |
| 06-003 | Priority order table + drag-and-drop persistence + CLI reordering | 06 | [ ] Todo |  | true | 05-002 | — | db, priority | feature/current/portfolio-layer/story-06-003-priority-ordering |
