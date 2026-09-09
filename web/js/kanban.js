/**
 * kanban.js — Main Kanban board logic: data fetching, rendering, auto-refresh.
 *
 * This module manages the core board state and rendering. Filter logic
 * lives in filters.js, drag-and-drop in drag-drop.js, and the detail
 * panel in detail-panel.js.
 */

// Board state
let allTasks = [];
let allFilterOptions = {};
let activeFilters = {
    portfolio: null,     // single-select: portfolio id
    project: new Set(),  // multi-select: project names
    app: null,           // single-select: app name
    story: null,         // single-select: story id
    requirement: null,   // single-select: requirement id
    tag: new Set(),      // multi-select: tag names
    state: new Set(),    // show/hide columns: state names
    assignee: null,      // single-select: assignee name
};

// All 6 Kanban columns in order
const COLUMNS = ['logged', 'open', 'in_progress', 'blocked', 'ready', 'closed'];

// Auto-refresh interval (ms)
const REFRESH_INTERVAL = 5000;
let refreshTimer = null;
let isDragging = false;

// Initialize all states visible by default
COLUMNS.forEach(s => activeFilters.state.add(s));

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

document.addEventListener('DOMContentLoaded', () => {
    initBoard();
});

async function initBoard() {
    await fetchFilterOptions();
    await fetchTasks();
    renderAll();
    setupAutoRefresh();
    setupDragAndDrop();
}

// ---------------------------------------------------------------------------
// Data fetching
// ---------------------------------------------------------------------------

/** Build query string from active filters (excluding state visibility). */
function buildQueryParams() {
    const params = new URLSearchParams();
    if (activeFilters.portfolio) params.set('portfolio', activeFilters.portfolio);
    // Project is multi-select but API takes single — use first if only one
    if (activeFilters.project.size === 1) {
        params.set('project', Array.from(activeFilters.project)[0]);
    }
    if (activeFilters.app) params.set('app', activeFilters.app);
    if (activeFilters.story) params.set('story', activeFilters.story);
    if (activeFilters.requirement) params.set('requirement', activeFilters.requirement);
    // Tag is multi-select but API takes single — use first if only one
    if (activeFilters.tag.size === 1) {
        params.set('tag', Array.from(activeFilters.tag)[0]);
    }
    if (activeFilters.assignee) params.set('assignee', activeFilters.assignee);
    return params;
}

/** Fetch tasks from GET /api/tasks with active filters. */
async function fetchTasks() {
    const params = buildQueryParams();
    const qs = params.toString();
    const url = '/api/tasks' + (qs ? '?' + qs : '');
    try {
        const resp = await fetch(url);
        if (!resp.ok) {
            showToast('Failed to fetch tasks: ' + resp.status);
            return;
        }
        allTasks = await resp.json();
    } catch (err) {
        console.error('fetchTasks error:', err);
        showToast('Network error fetching tasks');
    }
}

/** Fetch filter options from the various list endpoints. */
async function fetchFilterOptions() {
    try {
        const [portfolios, projects, apps, stories, requirements, tags] =
            await Promise.all([
                fetchJson('/api/portfolios'),
                fetchJson('/api/projects'),
                fetchJson('/api/apps'),
                fetchJson('/api/stories'),
                fetchJson('/api/requirements'),
                fetchJson('/api/tags'),
            ]);
        allFilterOptions = { portfolios, projects, apps, stories, requirements, tags };
    } catch (err) {
        console.error('fetchFilterOptions error:', err);
        showToast('Network error fetching filter options');
    }
}

/** Helper: fetch JSON from URL. */
async function fetchJson(url) {
    const resp = await fetch(url);
    if (!resp.ok) return [];
    return resp.json();
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/** Render everything: board, filters, counts, active filter chips. */
function renderAll() {
    renderBoard();
    renderFilters();
    renderActiveFilterChips();
    updateDragState();
}

/** Render the Kanban board with cards in columns. */
function renderBoard() {
    // Clear all columns
    COLUMNS.forEach(state => {
        const body = document.getElementById('col-body-' + state);
        if (body) body.innerHTML = '';
    });

    // Show/hide columns based on state filter
    COLUMNS.forEach(state => {
        const col = document.getElementById('col-' + state);
        if (col) {
            if (activeFilters.state.has(state)) {
                col.classList.remove('hidden');
            } else {
                col.classList.add('hidden');
            }
        }
    });

    // Count tasks per state
    const counts = {};
    COLUMNS.forEach(s => counts[s] = 0);

    // Render cards
    allTasks.forEach(task => {
        const state = task.state || 'logged';
        if (!counts.hasOwnProperty(state)) counts[state] = 0;
        counts[state] = (counts[state] || 0) + 1;
        if (!activeFilters.state.has(state)) return;
        const card = createCard(task);
        const body = document.getElementById('col-body-' + state);
        if (body) body.appendChild(card);
    });

    // Update column count badges
    COLUMNS.forEach(state => {
        const el = document.getElementById('count-' + state);
        if (el) el.textContent = counts[state] || 0;
    });

    // Update total count
    const totalEl = document.getElementById('totalCount');
    const visibleCount = allTasks.filter(t => activeFilters.state.has(t.state || 'logged')).length;
    totalEl.textContent = 'Showing ' + visibleCount + ' of ' + allTasks.length + ' tasks';
}

/** Create a card element for a task. */
function createCard(task) {
    const card = document.createElement('div');
    card.className = 'card';
    card.draggable = true;
    card.dataset.taskId = task.id;
    card.dataset.state = task.state || 'logged';
    card.addEventListener('click', () => openDetailPanel(task.id));

    // Title
    const title = document.createElement('div');
    title.className = 'card-title';
    title.textContent = task.title;
    card.appendChild(title);

    // Badges
    const badges = document.createElement('div');
    badges.className = 'card-badges';
    if (task.project) {
        const b = document.createElement('span');
        b.className = 'card-badge badge-project';
        b.textContent = task.project;
        badges.appendChild(b);
    }
    if (task.app) {
        const b = document.createElement('span');
        b.className = 'card-badge badge-app';
        b.textContent = task.app;
        badges.appendChild(b);
    }
    if (task.tags && task.tags.length > 0) {
        task.tags.forEach(tag => {
            const b = document.createElement('span');
            b.className = 'card-badge badge-tag';
            b.textContent = '#' + tag;
            badges.appendChild(b);
        });
    }
    card.appendChild(badges);

    // Footer: priority, deps, blocked indicator
    const footer = document.createElement('div');
    footer.className = 'card-footer';

    const priority = document.createElement('span');
    priority.className = 'card-priority';
    priority.textContent = 'P' + (task.priority || 0);
    footer.appendChild(priority);

    if (task.state === 'blocked') {
        const blocked = document.createElement('span');
        blocked.className = 'card-blocked';
        blocked.textContent = '\u26A0 blocked';
        footer.appendChild(blocked);
    }

    card.appendChild(footer);
    return card;
}

// ---------------------------------------------------------------------------
// Auto-refresh
// ---------------------------------------------------------------------------

function setupAutoRefresh() {
    if (refreshTimer) clearInterval(refreshTimer);
    refreshTimer = setInterval(async () => {
        if (isDragging) return; // Don't refresh during drag
        await fetchTasks();
        renderBoard();
        renderFilters();
    }, REFRESH_INTERVAL);
}

// ---------------------------------------------------------------------------
// Toast notifications
// ---------------------------------------------------------------------------

let toastTimer = null;
function showToast(message) {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.classList.remove('hidden');
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
        toast.classList.add('hidden');
    }, 3000);
}

// ---------------------------------------------------------------------------
// Filter sidebar toggle
// ---------------------------------------------------------------------------

function toggleFilterSidebar() {
    const sidebar = document.getElementById('filterSidebar');
    sidebar.classList.toggle('hidden');
}

// ---------------------------------------------------------------------------
// Cross-altitude rule
// ---------------------------------------------------------------------------

/** Check if a single story filter is active. */
function isStoryFiltered() {
    return activeFilters.story !== null && activeFilters.story !== '';
}

/** Update drag state based on cross-altitude rule. */
function updateDragState() {
    const storyFiltered = isStoryFiltered();
    COLUMNS.forEach(state => {
        const col = document.getElementById('col-' + state);
        if (!col) return;
        if (storyFiltered) {
            col.classList.remove('drag-disabled');
        } else {
            col.classList.add('drag-disabled');
        }
    });
}
