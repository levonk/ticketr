---
story_id: "06-001"
story_title: "Static web UI (Kanban board, drag-and-drop, count annotations, filtering)"
story_name: "kanban-web-ui"
prd_name: "portfolio-layer"
prd_file: "internal-docs/feature/todo/portfolio-layer/feat-202609081145-portfolio-layer.md"
phase: 6
parallel_id: 1
branch: "feature/current/portfolio-layer/story-06-001-kanban-web-ui"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["05-002"]
parallel_safe: true
modules: ["web", "ui"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "frontend", "ui"]
due: ""
created_at: "2026-09-08"
updated_at: "2026-09-08"
---

## Summary

Build the static web UI for the portfolio layer — a Kanban board served from the `web/` directory as static HTML/CSS/JavaScript. The board displays tasks as cards organized into columns by task state (logged, open, in_progress, blocked, ready, closed). Users can filter by portfolio, project, app, story, requirement, tag, state, and assignee. Every container shows superscript count badges scoped to active filters. Drag-and-drop within a column reorders priority (writes to `priority_order` via API), and drag between columns changes task state (writes to markdown via API). The cross-altitude rule disables drag-reorder when no single story filter is active. A card detail panel allows editing task metadata.

## Current State

- `web/index.html` (22,234 bytes) contains the existing Kanban board UI for the pre-portfolio `tkr web` command.
- `src/web.rs` (lines 66-68) serves static files from the `web/` directory via `warp::fs::dir("web")`.
- The existing UI is a single-file HTML with inline CSS/JS — no build step, no framework, no module system.
- The existing UI calls `GET /api/tickets` and `PUT /api/tickets/:id` (the two endpoints in `src/web.rs`).
- No portfolio filtering, no count annotations, no drag-and-drop priority reordering, no story/requirement/tag awareness.
- The portfolio API (Story 05-002) provides `GET /api/tasks` with composable filters, `PUT /api/priority`, `GET /api/priority`, and all CRUD endpoints for hierarchy entities.

## Scope

### In Scope

- Rebuild `web/index.html` (or split into `web/index.html` + `web/css/` + `web/js/`) as the portfolio Kanban board.
- **Kanban columns** mapping to Task-level state flow: `logged`, `open`, `in_progress`, `blocked`, `ready`, `closed`.
- **Card rendering**: compact cards showing title, project badge, app badge, tag chips, priority indicator, dependency count, blocked indicator.
- **Filtering** (sidebar/top bar) with composable filters:
  - Portfolio (single-select dropdown)
  - Project (multi-select)
  - App (single-select, scoped to selected project)
  - Story (single-select)
  - Requirement (single-select, shows stories under it)
  - Tag (multi-select)
  - State (show/hide individual columns)
  - Assignee (single-select)
- **Count annotations** (superscript badges) on every container:
  - Column headers: `Open ¹²`, `In Progress ³`, `Ready ⁵`
  - Filter sidebar options: `personal ⁸`, `tkr ⁸`, `default ⁵`, `Portfolio Layer ⁵`, `security ⁴`
  - Total at top: `Showing 14 of 42 tasks`
  - Counts are scoped to active filters (selecting `tag=security` updates project counts to show only security tasks per project)
  - Empty-state indicators: count `⁰` is greyed out but still visible
- **Drag-and-drop**:
  - Within a column: reorders priority → `PUT /api/priority` with `{ scope_type, scope_id, task_id, new_position }`
  - Between columns: changes task state → `PUT /api/tasks/:id` with new `state`
  - Cross-altitude rule: when no single story filter is active, drag-reorder is disabled with tooltip: "Filter to a single story to reorder — cross-altitude prioritization is not meaningful."
- **Card detail panel**: click a card to open a side panel for editing title, description, tags, story link, dependencies.
- **Filter composition display**: show active filters as removable chips at the top.
- **Auto-refresh**: poll `GET /api/tasks` every 5 seconds (or use WebSocket/SSE in future) to reflect daemon-triggered changes.
- **Responsive layout**: works on desktop (primary target); degrades gracefully on tablet.

### Out of Scope

- Backend API implementation (Story 05-002).
- Daemon and file watcher (Story 05-001).
- GitHub sync (Story 06-002).
- Priority ordering backend logic (Story 06-003).
- Other views mentioned in PRD (requirement view, tag view, project view, portfolio view, backlog view) — these are future enhancements; the Kanban board is the primary view for this story.
- Mobile-specific layout (desktop-first; responsive but not mobile-optimized).
- Build tooling (no webpack/vite — static files only, no npm dependencies).

## Sub-Tasks

1. **HTML structure** — Create the page layout: header (title, total count, active filter chips), filter sidebar (collapsible), Kanban board area (6 columns), card detail panel (slide-in from right).
2. **CSS styling** — Style the Kanban board, cards, filter sidebar, count badges, drag-and-drop visual feedback, detail panel. Use CSS Grid/Flexbox. No external CSS framework (keep it dependency-free).
3. **JavaScript: data fetching** — Implement `fetchTasks()` that calls `GET /api/tasks` with current filter state as query params. Implement `fetchFilterOptions()` that calls `GET /api/portfolios`, `GET /api/projects`, `GET /api/apps`, `GET /api/stories`, `GET /api/requirements`, `GET /api/tags` to populate filter dropdowns.
4. **JavaScript: rendering** — Render cards into columns based on task state. Render filter sidebar with count badges. Render total count. Update all counts when filters change (re-fetch with new params).
5. **JavaScript: count annotations** — Compute counts for each filter option scoped to current active filters. Display as superscript badges. Grey out options with count `⁰`.
6. **JavaScript: filtering** — Handle filter selection/deselection. Compose filters into query params. Re-fetch tasks and filter options on any filter change. Show active filters as removable chips.
7. **JavaScript: drag-and-drop within column** — Use HTML5 Drag and Drop API. On drop within a column, call `PUT /api/priority` with the new position. Update card order optimistically, revert on error.
8. **JavaScript: drag-and-drop between columns** — On drop into a different column, call `PUT /api/tasks/:id` with the new state. Move card to new column optimistically, revert on error.
9. **JavaScript: cross-altitude rule** — Check if a single story filter is active. If not, disable drag-reorder within columns (add `drag-disabled` class, show tooltip on hover). Drag between columns (state change) remains enabled regardless.
10. **JavaScript: card detail panel** — On card click, open a side panel. Fetch full task details (`GET /api/tasks/:id`). Allow editing: title, description, tags (add/remove), story link (dropdown), dependencies (add/remove). Save changes via `PUT /api/tasks/:id`.
11. **JavaScript: auto-refresh** — Poll `GET /api/tasks` every 5 seconds with current filters. Diff results and update DOM efficiently (don't full re-render if only counts changed).
12. **File organization** — Split into `web/index.html`, `web/css/kanban.css`, `web/js/kanban.js`, `web/js/filters.js`, `web/js/drag-drop.js`, `web/js/detail-panel.js` (or keep as single file if simpler — match existing pattern).

## Relevant Files

| File | Role | Changes |
|------|------|---------|
| `web/index.html` | Main UI page | **Major rewrite** — portfolio Kanban board with filtering, counts, drag-and-drop |
| `web/css/kanban.css` | **NEW** (optional) — Stylesheet | Kanban board, card, filter, badge styles |
| `web/js/kanban.js` | **NEW** (optional) — Main JS | Data fetching, rendering, auto-refresh |
| `web/js/filters.js` | **NEW** (optional) — Filter JS | Filter composition, count annotations |
| `web/js/drag-drop.js` | **NEW** (optional) — D&D JS | Drag-and-drop within and between columns |
| `web/js/detail-panel.js` | **NEW** (optional) — Detail JS | Card detail panel editing |
| `src/web.rs` | Web server | No changes needed (static file serving already configured at lines 66-68) |

## Acceptance Criteria

- [ ] Kanban board renders 6 columns: `logged`, `open`, `in_progress`, `blocked`, `ready`, `closed`.
- [ ] Each column displays task cards with: title, project badge, app badge, tag chips, priority indicator, dependency count, blocked indicator.
- [ ] Filter sidebar allows selecting: portfolio, project (multi), app, story, requirement, tag (multi), state (show/hide columns), assignee.
- [ ] Filters compose: selecting `portfolio=personal` + `project=tkr` + `tag=security` + `state=open` shows only matching tasks.
- [ ] Active filters appear as removable chips at the top of the board.
- [ ] Column headers show superscript count badges (e.g., `Open ¹²`).
- [ ] Filter sidebar options show superscript count badges (e.g., `tkr ⁸`, `security ⁴`).
- [ ] Total count at top shows `Showing N of M tasks`.
- [ ] Counts are scoped to active filters: selecting `tag=security` updates project counts to show only security tasks per project.
- [ ] Empty-state indicators: filter options with count `⁰` are greyed out but still visible.
- [ ] Drag-and-drop within a column reorders priority and calls `PUT /api/priority`.
- [ ] Drag-and-drop between columns changes task state and calls `PUT /api/tasks/:id`.
- [ ] Cross-altitude rule: drag-reorder within columns is disabled when no single story filter is active. Tooltip shown: "Filter to a single story to reorder — cross-altitude prioritization is not meaningful."
- [ ] Drag between columns (state change) is always enabled regardless of story filter.
- [ ] Clicking a card opens a detail panel for editing title, description, tags, story, dependencies.
- [ ] Saving edits in the detail panel calls `PUT /api/tasks/:id` and updates the board.
- [ ] Board auto-refreshes every 5 seconds to reflect daemon-triggered changes.
- [ ] UI works in modern browsers (Chrome, Firefox, Safari — no IE support needed).
- [ ] No external JavaScript dependencies (no npm, no CDN — pure vanilla JS).
- [ ] `devbox run just build-internal` succeeds (static files don't affect build, but verify no breakage).
- [ ] `devbox run just test-internal` passes.

## Examples

### Kanban board layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  tkr Portfolio Board          Showing 14 of 42 tasks                     │
│  [personal ▾] [tkr ▾] [security ×] [open ×]                     [Filters] │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────────────┤
│ Logged ² │ Open ⁵   │ In Prog ³│ Blocked ¹│ Ready ⁴  │ Closed ⁰         │
│          │          │          │          │          │                  │
│ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │                  │
│ │Task A│ │ │Task C│ │ │Task E│ │ │Task G│ │ │Task H│ │                  │
│ │ tkr  │ │ │ tkr  │ │ │ tkr  │ │ │ tkr  │ │ │ tkr  │ │                  │
│ │ #sec │ │ │ #bck │ │ │      │ │ │ ⚠blk│ │ │      │ │                  │
│ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │                  │
│ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │          │ ┌──────┐ │                  │
│ │Task B│ │ │Task D│ │ │Task F│ │          │ │Task I│ │                  │
│ └──────┘ │ └──────┘ │ └──────┘ │          │ └──────┘ │                  │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────────────┘
```

### Filter sidebar with counts

```
┌─ Filters ──────────────────┐
│ Portfolio                  │
│   personal ⁸  (selected)    │
│   business-a ⁴             │
│   client-x ²                │
│                            │
│ Project                    │
│   tkr ⁸  (selected)        │
│   dotfiles ⁴               │
│   infrahub ²                │
│                            │
│ Story                       │
│   Portfolio Layer ⁵        │
│   Auth Refactor ³          │
│   (none) ⁶                 │
│                            │
│ Tag                        │
│   security ⁴  (selected)   │
│   backend ⁷                │
│   frontend ²  (greyed ⁰)   │
└────────────────────────────┘
```

### Cross-altitude rule tooltip

```
When no story filter is active and user tries to drag-reorder within a column:

  ┌──────────────────────────────────────────┐
  │ ⚠ Filter to a single story to reorder —  │
  │   cross-altitude prioritization is not    │
  │   meaningful.                            │
  └──────────────────────────────────────────┘
```

### Card detail panel

```
┌─ Task Detail ─────────────────┐
│ ID: ja-6b9a0dc                │
│ Title: [Add rusqlite dep___]  │
│ State: open ▾                 │
│ Story: Portfolio Layer ▾      │
│ Tags: [security] [backend] +  │
│ Assignee: levonk ▾            │
│ Priority: 2                   │
│ Deps: ja-old1234 (×)           │
│                               │
│ Description:                  │
│ ┌─────────────────────────┐  │
│ │ Add rusqlite 0.40 with  │  │
│ │ bundled feature...      │  │
│ └─────────────────────────┘  │
│                               │
│ [Save]  [Cancel]              │
└───────────────────────────────┘
```

## Test Plan

### Manual Testing

1. **Board rendering** — Start daemon + web server, open `http://127.0.0.1:8080`, verify Kanban board renders with all 6 columns.
2. **Filter composition** — Select `portfolio=personal`, `project=tkr`, `tag=security`, verify only matching tasks appear.
3. **Count scoping** — Select `tag=security`, verify project counts update to show only security tasks per project.
4. **Empty-state** — Find a tag with 0 tasks under current filters, verify it's greyed out but visible.
5. **Drag within column** — Filter to a single story, drag a card within a column, verify `PUT /api/priority` is called and order persists after refresh.
6. **Drag between columns** — Drag a card from `open` to `in_progress`, verify `PUT /api/tasks/:id` is called and card moves.
7. **Cross-altitude rule** — Clear story filter, try to drag-reorder within a column, verify it's disabled with tooltip.
8. **Card detail panel** — Click a card, verify panel opens, edit title, save, verify board updates.
9. **Auto-refresh** — Modify a markdown ticket file externally, verify board updates within 5 seconds without manual refresh.
10. **Filter chip removal** — Click the × on a filter chip, verify filter is removed and board updates.

### Automated Testing

- **DOM structure tests** — Use a headless browser (or manual snapshot) to verify column count, card count, badge presence.
- **API integration tests** — Verify the UI calls the correct API endpoints (can be tested via the API tests in Story 05-002).
- **Note**: Full E2E browser tests are out of scope for this story (no browser test framework in the project). Manual testing checklist above is the primary verification method.

## Observability

- Console logging in development mode (`console.log` for API calls, filter changes, drag events).
- Visual feedback for API errors (toast notification on failed `PUT /api/priority` or `PUT /api/tasks/:id`).
- Loading indicators during API calls (spinner or skeleton cards).
- Drag-and-drop visual feedback (card opacity, drop zone highlight).

## Compliance

- No external dependencies (no npm, no CDN, no framework — pure HTML/CSS/vanilla JS).
- No tracking, no analytics, no external API calls (all requests go to localhost).
- No secrets in static files (API tokens are handled server-side).
- Accessible: use semantic HTML, ARIA labels for drag-and-drop, keyboard navigation for filters.
- Works offline (static files served from local warp server).

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| HTML5 Drag and Drop API inconsistencies across browsers | Medium | Medium | Test in Chrome/Firefox/Safari; use pointer events as fallback if needed |
| Large task counts cause rendering performance issues | Low | Medium | Implement virtual scrolling or pagination in a future story; for now, cap at 200 cards per column |
| Auto-refresh causes flicker or lost drag state | Medium | Medium | Diff results and update only changed cards; pause auto-refresh during drag operations |
| Single-file HTML becomes unmaintainable | Medium | Low | Split into separate CSS/JS files if exceeding 1000 lines |
| Cross-altitude rule UX confusion | Low | Medium | Clear tooltip text; visual indicator (lock icon) on columns when reorder is disabled |

## Dependencies & Sequencing

- **Depends on**: `05-002` (Portfolio API endpoints) — the UI calls all the API endpoints defined in that story.
- **Dependants**: None (this is a leaf story in the dependency graph).
- **Parallel-safe**: Yes — this story only touches `web/` directory files. It does not modify any Rust source files.
- **External dependencies**: None (pure static files, no build step).

## Definition of Done

- [ ] Kanban board renders with 6 state columns.
- [ ] Cards display title, project, app, tags, priority, dependency count, blocked indicator.
- [ ] Filter sidebar with all 8 filter types (portfolio, project, app, story, requirement, tag, state, assignee).
- [ ] Filters compose correctly (AND logic).
- [ ] Count annotations (superscript badges) on column headers, filter options, and total.
- [ ] Counts scoped to active filters.
- [ ] Empty-state indicators (count ⁰ greyed out).
- [ ] Drag-and-drop within column calls `PUT /api/priority`.
- [ ] Drag-and-drop between columns calls `PUT /api/tasks/:id`.
- [ ] Cross-altitude rule enforced (no story filter → drag-reorder disabled with tooltip).
- [ ] Card detail panel with editing (title, description, tags, story, deps).
- [ ] Auto-refresh every 5 seconds.
- [ ] No external JavaScript dependencies.
- [ ] Manual testing checklist completed.
- [ ] `devbox run just build-internal` succeeds.
- [ ] `devbox run just test-internal` passes.

## STOP Conditions

- If the HTML5 Drag and Drop API is too inconsistent across target browsers, stop and evaluate using a drag-and-drop library (e.g., SortableJS — but this adds a dependency, so discuss first).
- If the single-file HTML approach becomes unmaintainable past 2000 lines, stop and split into separate files before continuing.
- If auto-refresh polling causes excessive API load or flicker, stop and evaluate Server-Sent Events (SSE) via warp for push-based updates.
- If the count annotation computation (scoped to active filters) requires too many API calls, stop and evaluate a single aggregated counts endpoint (`GET /api/counts?filters=...`).

## Maintenance Notes

- The UI is pure static HTML/CSS/JS with no build step — changes take effect on browser refresh.
- The existing `web/index.html` (22KB) will be replaced; keep a backup or git history for reference.
- If the file is split into `web/css/` and `web/js/` subdirectories, ensure `warp::fs::dir("web")` still serves them (it does — it serves the entire directory tree).
- The auto-refresh interval (5 seconds) is a constant in the JS code; can be made configurable in the future.
- The cross-altitude rule tooltip text is defined in the PRD (line 398) and should match exactly.
- Count badge format uses Unicode superscript characters (⁰¹²³⁴⁵⁶⁷⁸⁹) — ensure the font supports them.

## Commit Conventions

- Use conventional commits: `feat(ui): add portfolio Kanban board with filtering and drag-and-drop`.
- Split commits by feature area if the diff is large (e.g., one commit for HTML structure, one for CSS, one for JS data fetching, one for drag-and-drop, one for detail panel).
- Reference story ID in PR description: `Story: 06-001`.
- Branch: `feature/current/portfolio-layer/story-06-001-kanban-web-ui`.
