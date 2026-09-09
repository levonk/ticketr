/**
 * filters.js — Filter composition, count annotations, and filter UI rendering.
 *
 * Handles rendering the filter sidebar with superscript count badges,
 * computing scoped counts, and managing filter selection/deselection.
 */

// Superscript digit mapping for count badges
const SUPERSCRIPTS = ['\u2070', '\u00B9', '\u00B2', '\u00B3', '\u2074',
                       '\u2075', '\u2076', '\u2077', '\u2078', '\u2079'];

/** Convert a number to superscript string (e.g. 12 → ¹²). */
function toSuperscript(n) {
    if (n === 0) return SUPERSCRIPTS[0];
    let result = '';
    const str = String(n);
    for (const ch of str) {
        const digit = parseInt(ch, 10);
        if (digit >= 0 && digit <= 9) {
            result += SUPERSCRIPTS[digit];
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// Count computation (scoped to active filters)
// ---------------------------------------------------------------------------

/** Count tasks matching a given filter value, scoped to current active filters. */
function computeScopedCount(filterType, filterValue) {
    return allTasks.filter(task => {
        // Check if task matches this filter value
        let matches = false;
        switch (filterType) {
            case 'portfolio':
                // Task's project's portfolio matches
                matches = taskProjectPortfolio(task) === filterValue;
                break;
            case 'project':
                matches = task.project === filterValue;
                break;
            case 'app':
                matches = task.app === filterValue;
                break;
            case 'story':
                matches = task.story === filterValue;
                break;
            case 'tag':
                matches = task.tags && task.tags.includes(filterValue);
                break;
            case 'state':
                matches = (task.state || 'logged') === filterValue;
                break;
            case 'assignee':
                // Assignee not in task response — always 0 for now
                matches = false;
                break;
            default:
                matches = false;
        }
        if (!matches) return false;

        // Apply other active filters (exclude the current filter type)
        return matchesOtherFilters(task, filterType);
    }).length;
}

/** Check if task matches all active filters except the given filter type. */
function matchesOtherFilters(task, excludeFilterType) {
    if (excludeFilterType !== 'portfolio' && activeFilters.portfolio) {
        if (taskProjectPortfolio(task) !== activeFilters.portfolio) return false;
    }
    if (excludeFilterType !== 'project' && activeFilters.project.size > 0) {
        if (!activeFilters.project.has(task.project)) return false;
    }
    if (excludeFilterType !== 'app' && activeFilters.app) {
        if (task.app !== activeFilters.app) return false;
    }
    if (excludeFilterType !== 'story' && activeFilters.story) {
        if (task.story !== activeFilters.story) return false;
    }
    if (excludeFilterType !== 'tag' && activeFilters.tag.size > 0) {
        if (!task.tags || !task.tags.some(t => activeFilters.tag.has(t))) return false;
    }
    if (excludeFilterType !== 'assignee' && activeFilters.assignee) {
        // Assignee not in task response — can't filter
    }
    return true;
}

/** Look up a task's project's portfolio id from filter options. */
function taskProjectPortfolio(task) {
    if (!task.project) return null;
    const proj = allFilterOptions.projects &&
        allFilterOptions.projects.find(p => p.name === task.project);
    return proj ? proj.portfolio_id : null;
}

// ---------------------------------------------------------------------------
// Filter rendering
// ---------------------------------------------------------------------------

/** Render all filter sections in the sidebar. */
function renderFilters() {
    renderPortfolioFilter();
    renderProjectFilter();
    renderAppFilter();
    renderStoryFilter();
    renderRequirementFilter();
    renderTagFilter();
    renderStateFilter();
    renderAssigneeFilter();
}

/** Create a filter list item with count badge. */
function createFilterItem(value, label, filterType, count, isSelected, isMulti) {
    const li = document.createElement('li');
    if (isSelected) li.classList.add('selected');
    if (count === 0) li.classList.add('zero-count');

    const text = document.createElement('span');
    text.textContent = label;
    li.appendChild(text);

    const badge = document.createElement('span');
    badge.className = 'filter-count';
    badge.textContent = toSuperscript(count);
    li.appendChild(badge);

    li.addEventListener('click', () => {
        if (isMulti) {
            toggleMultiFilter(filterType, value);
        } else {
            toggleSingleFilter(filterType, value);
        }
    });

    return li;
}

function renderPortfolioFilter() {
    const list = document.getElementById('portfolioList');
    list.innerHTML = '';
    const portfolios = allFilterOptions.portfolios || [];
    portfolios.forEach(p => {
        const count = computeScopedCount('portfolio', p.id);
        const isSelected = activeFilters.portfolio === p.id;
        list.appendChild(createFilterItem(p.id, p.name, 'portfolio', count, isSelected, false));
    });
}

function renderProjectFilter() {
    const list = document.getElementById('projectList');
    list.innerHTML = '';
    const projects = allFilterOptions.projects || [];
    // Filter projects by selected portfolio
    const visibleProjects = activeFilters.portfolio
        ? projects.filter(p => p.portfolio_id === activeFilters.portfolio)
        : projects;
    visibleProjects.forEach(p => {
        const count = computeScopedCount('project', p.name);
        const isSelected = activeFilters.project.has(p.name);
        list.appendChild(createFilterItem(p.name, p.name, 'project', count, isSelected, true));
    });
}

function renderAppFilter() {
    const list = document.getElementById('appList');
    list.innerHTML = '';
    const apps = allFilterOptions.apps || [];
    // Filter apps by selected project(s)
    let visibleApps = apps;
    if (activeFilters.project.size > 0) {
        const projectIds = new Set();
        (allFilterOptions.projects || []).forEach(p => {
            if (activeFilters.project.has(p.name)) projectIds.add(p.id);
        });
        visibleApps = apps.filter(a => projectIds.has(a.project_id));
    }
    apps.forEach(a => {
        const count = computeScopedCount('app', a.name);
        const isSelected = activeFilters.app === a.name;
        list.appendChild(createFilterItem(a.name, a.name, 'app', count, isSelected, false));
    });
}

function renderStoryFilter() {
    const list = document.getElementById('storyList');
    list.innerHTML = '';
    const stories = allFilterOptions.stories || [];
    // Add "(none)" option for tasks without a story
    const noneCount = allTasks.filter(t => !t.story && matchesOtherFilters(t, 'story')).length;
    list.appendChild(createFilterItem('', '(none)', 'story', noneCount,
        activeFilters.story === '', false));
    stories.forEach(s => {
        const count = computeScopedCount('story', s.id);
        const isSelected = activeFilters.story === s.id;
        list.appendChild(createFilterItem(s.id, s.title, 'story', count, isSelected, false));
    });
}

function renderRequirementFilter() {
    const list = document.getElementById('requirementList');
    list.innerHTML = '';
    const requirements = allFilterOptions.requirements || [];
    // Filter requirements by selected portfolio
    const visibleReqs = activeFilters.portfolio
        ? requirements.filter(r => r.portfolio_id === activeFilters.portfolio)
        : requirements;
    visibleReqs.forEach(r => {
        // Count tasks whose story belongs to this requirement
        const storyIds = (allFilterOptions.stories || [])
            .filter(s => s.requirement_id === r.id)
            .map(s => s.id);
        const count = allTasks.filter(t =>
            storyIds.includes(t.story) && matchesOtherFilters(t, 'requirement')
        ).length;
        const isSelected = activeFilters.requirement === r.id;
        list.appendChild(createFilterItem(r.id, r.title, 'requirement', count, isSelected, false));
    });
}

function renderTagFilter() {
    const list = document.getElementById('tagList');
    list.innerHTML = '';
    const tags = allFilterOptions.tags || [];
    tags.forEach(t => {
        const tagName = t.name || t;
        const count = computeScopedCount('tag', tagName);
        const isSelected = activeFilters.tag.has(tagName);
        list.appendChild(createFilterItem(tagName, tagName, 'tag', count, isSelected, true));
    });
}

function renderStateFilter() {
    const list = document.getElementById('stateList');
    list.innerHTML = '';
    const stateLabels = {
        logged: 'Logged', open: 'Open', in_progress: 'In Progress',
        blocked: 'Blocked', ready: 'Ready', closed: 'Closed'
    };
    COLUMNS.forEach(state => {
        const count = allTasks.filter(t => (t.state || 'logged') === state).length;
        const isSelected = activeFilters.state.has(state);
        const li = createFilterItem(state, stateLabels[state], 'state', count, isSelected, true);
        list.appendChild(li);
    });
}

function renderAssigneeFilter() {
    const list = document.getElementById('assigneeList');
    list.innerHTML = '';
    // Assignees not available in task response — show placeholder
    const li = document.createElement('li');
    li.className = 'zero-count';
    li.textContent = '(not available)';
    const badge = document.createElement('span');
    badge.className = 'filter-count';
    badge.textContent = toSuperscript(0);
    li.appendChild(badge);
    list.appendChild(li);
}

// ---------------------------------------------------------------------------
// Filter toggle handlers
// ---------------------------------------------------------------------------

function toggleSingleFilter(filterType, value) {
    if (activeFilters[filterType] === value) {
        activeFilters[filterType] = null;
    } else {
        activeFilters[filterType] = value;
    }
    onFilterChange();
}

function toggleMultiFilter(filterType, value) {
    const set = activeFilters[filterType];
    if (set.has(value)) {
        set.delete(value);
    } else {
        set.add(value);
    }
    onFilterChange();
}

/** Called when any filter changes — re-fetch and re-render. */
async function onFilterChange() {
    await fetchTasks();
    renderAll();
}

// ---------------------------------------------------------------------------
// Active filter chips
// ---------------------------------------------------------------------------

function renderActiveFilterChips() {
    const container = document.getElementById('activeFilters');
    container.innerHTML = '';

    if (activeFilters.portfolio) {
        addFilterChip(container, 'portfolio', activeFilters.portfolio,
            portfolioName(activeFilters.portfolio));
    }
    activeFilters.project.forEach(p => {
        addFilterChip(container, 'project', p, p, true);
    });
    if (activeFilters.app) {
        addFilterChip(container, 'app', activeFilters.app, activeFilters.app);
    }
    if (activeFilters.story !== null) {
        const label = activeFilters.story === '' ? '(none)' : storyTitle(activeFilters.story);
        addFilterChip(container, 'story', activeFilters.story, label);
    }
    if (activeFilters.requirement) {
        addFilterChip(container, 'requirement', activeFilters.requirement,
            requirementTitle(activeFilters.requirement));
    }
    activeFilters.tag.forEach(t => {
        addFilterChip(container, 'tag', t, t, true);
    });
    // State visibility is not shown as chips (it's column show/hide)
}

function addFilterChip(container, filterType, value, label, isMulti) {
    const chip = document.createElement('span');
    chip.className = 'filter-chip';
    chip.textContent = label;

    const removeBtn = document.createElement('span');
    removeBtn.className = 'chip-remove';
    removeBtn.textContent = '\u00D7';
    removeBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        if (isMulti) {
            activeFilters[filterType].delete(value);
        } else {
            activeFilters[filterType] = null;
        }
        onFilterChange();
    });
    chip.appendChild(removeBtn);
    container.appendChild(chip);
}

function portfolioName(id) {
    const p = (allFilterOptions.portfolios || []).find(p => p.id === id);
    return p ? p.name : id;
}

function storyTitle(id) {
    const s = (allFilterOptions.stories || []).find(s => s.id === id);
    return s ? s.title : id;
}

function requirementTitle(id) {
    const r = (allFilterOptions.requirements || []).find(r => r.id === id);
    return r ? r.title : id;
}

// ---------------------------------------------------------------------------
// Clear all filters
// ---------------------------------------------------------------------------

function clearAllFilters() {
    activeFilters.portfolio = null;
    activeFilters.project = new Set();
    activeFilters.app = null;
    activeFilters.story = null;
    activeFilters.requirement = null;
    activeFilters.tag = new Set();
    activeFilters.assignee = null;
    // Reset state visibility to all
    activeFilters.state = new Set(COLUMNS);
    onFilterChange();
}
