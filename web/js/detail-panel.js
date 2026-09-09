/**
 * detail-panel.js — Card detail panel for editing task metadata.
 *
 * Opens a slide-in panel when a card is clicked. Fetches full task
 * details, allows editing title, description, tags, story link, and
 * dependencies. Saves changes via PUT /api/tasks/:id.
 */

let currentDetailTaskId = null;

// ---------------------------------------------------------------------------
// Open / close panel
// ---------------------------------------------------------------------------

async function openDetailPanel(taskId) {
    currentDetailTaskId = taskId;
    const panel = document.getElementById('detailPanel');
    panel.classList.remove('hidden');

    // Show loading state
    document.getElementById('detailIdDisplay').textContent = taskId;

    // Fetch full task details
    try {
        const resp = await fetch('/api/tasks/' + taskId);
        if (!resp.ok) {
            showToast('Failed to load task details');
            return;
        }
        const task = await resp.json();
        populateDetailForm(task);
    } catch (err) {
        console.error('openDetailPanel error:', err);
        showToast('Network error loading task details');
    }
}

function closeDetailPanel() {
    const panel = document.getElementById('detailPanel');
    panel.classList.add('hidden');
    currentDetailTaskId = null;
}

// ---------------------------------------------------------------------------
// Populate form
// ---------------------------------------------------------------------------

function populateDetailForm(task) {
    document.getElementById('detailId').value = task.id;
    document.getElementById('detailIdDisplay').textContent = task.id;
    document.getElementById('detailTitle').value = task.title || '';
    document.getElementById('detailState').value = task.state || 'logged';
    document.getElementById('detailTags').value =
        (task.tags || []).join(', ');
    document.getElementById('detailPriority').value = task.priority || 0;
    document.getElementById('detailDescription').value = task.description || '';
    document.getElementById('detailAssignee').value = task.assignee || '';
    document.getElementById('detailDeps').value =
        (task.deps || []).join(', ');

    // Populate story dropdown
    const storySelect = document.getElementById('detailStory');
    storySelect.innerHTML = '<option value="">(none)</option>';
    const stories = allFilterOptions.stories || [];
    stories.forEach(s => {
        const opt = document.createElement('option');
        opt.value = s.id;
        opt.textContent = s.title;
        if (task.story === s.id) opt.selected = true;
        storySelect.appendChild(opt);
    });
}

// ---------------------------------------------------------------------------
// Save task
// ---------------------------------------------------------------------------

async function saveTaskDetail(event) {
    event.preventDefault();

    const taskId = document.getElementById('detailId').value;
    if (!taskId) return;

    const tags = document.getElementById('detailTags').value
        .split(',').map(t => t.trim()).filter(t => t.length > 0);
    const deps = document.getElementById('detailDeps').value
        .split(',').map(d => d.trim()).filter(d => d.length > 0);

    const update = {
        title: document.getElementById('detailTitle').value,
        status: document.getElementById('detailState').value,
        story: document.getElementById('detailStory').value || null,
        tags: tags,
        priority: parseInt(document.getElementById('detailPriority').value, 10) || 0,
        assignee: document.getElementById('detailAssignee').value || null,
        description: document.getElementById('detailDescription').value || null,
    };

    try {
        const resp = await fetch('/api/tasks/' + taskId, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(update),
        });

        if (!resp.ok) {
            showToast('Failed to save task');
            return;
        }

        const updatedTask = await resp.json();

        // Update local task
        const idx = allTasks.findIndex(t => t.id === taskId);
        if (idx >= 0) {
            allTasks[idx] = updatedTask;
        }

        closeDetailPanel();
        renderAll();
    } catch (err) {
        console.error('saveTaskDetail error:', err);
        showToast('Network error saving task');
    }
}
