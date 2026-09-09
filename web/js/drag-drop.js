/**
 * drag-drop.js — Drag-and-drop within and between Kanban columns.
 *
 * Uses the HTML5 Drag and Drop API (no external library).
 * - Drag within a column: reorders priority → PUT /api/priority
 * - Drag between columns: changes task state → PUT /api/tasks/:id
 * - Cross-altitude rule: drag-reorder disabled when no single story filter
 */

let draggedCard = null;
let draggedFromColumn = null;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

function setupDragAndDrop() {
    COLUMNS.forEach(state => {
        const body = document.getElementById('col-body-' + state);
        if (!body) return;
        body.addEventListener('dragover', handleDragOver);
        body.addEventListener('drop', handleDrop);
        body.addEventListener('dragleave', handleDragLeave);
        body.addEventListener('dragenter', handleDragEnter);
    });

    // Delegate card dragstart/dragend on the board
    document.getElementById('board').addEventListener('dragstart', handleDragStart);
    document.getElementById('board').addEventListener('dragend', handleDragEnd);
}

// ---------------------------------------------------------------------------
// Drag start / end
// ---------------------------------------------------------------------------

function handleDragStart(e) {
    if (!e.target.classList || !e.target.classList.contains('card')) return;

    // Check cross-altitude rule for within-column reorder
    const col = e.target.closest('.column');
    if (col && col.classList.contains('drag-disabled')) {
        // Allow drag start but we'll prevent reorder on drop
        // State change (between columns) is still allowed
    }

    draggedCard = e.target;
    draggedFromColumn = col ? col.dataset.state : null;
    draggedCard.classList.add('dragging');
    isDragging = true;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', draggedCard.dataset.taskId);
}

function handleDragEnd(e) {
    if (draggedCard) {
        draggedCard.classList.remove('dragging');
    }
    // Remove all drag-over highlights
    document.querySelectorAll('.column-body.drag-over').forEach(el => {
        el.classList.remove('drag-over');
    });
    draggedCard = null;
    draggedFromColumn = null;
    isDragging = false;
}

// ---------------------------------------------------------------------------
// Drag over / enter / leave
// ---------------------------------------------------------------------------

function handleDragEnter(e) {
    e.preventDefault();
    if (e.currentTarget.classList.contains('column-body')) {
        e.currentTarget.classList.add('drag-over');
    }
}

function handleDragOver(e) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
}

function handleDragLeave(e) {
    // Only remove highlight if leaving the column body entirely
    if (e.currentTarget === e.target) {
        e.currentTarget.classList.remove('drag-over');
    }
}

// ---------------------------------------------------------------------------
// Drop handler
// ---------------------------------------------------------------------------

function handleDrop(e) {
    e.preventDefault();
    const dropBody = e.currentTarget;
    dropBody.classList.remove('drag-over');

    if (!draggedCard) return;

    const taskId = draggedCard.dataset.taskId;
    const fromState = draggedFromColumn;
    const toColumn = dropBody.closest('.column');
    const toState = toColumn ? toColumn.dataset.state : null;

    if (!fromState || !toState) return;

    if (fromState === toState) {
        // Within-column reorder (priority change)
        handleWithinColumnDrop(taskId, fromState, dropBody, e);
    } else {
        // Between-column drop (state change)
        handleBetweenColumnDrop(taskId, fromState, toState, dropBody, e);
    }
}

// ---------------------------------------------------------------------------
// Within-column drop: reorder priority
// ---------------------------------------------------------------------------

function handleWithinColumnDrop(taskId, state, dropBody, e) {
    // Cross-altitude rule: disable reorder if no story filter
    if (!isStoryFiltered()) {
        showCrossAltitudeTooltip(e);
        // Revert: re-render board
        renderBoard();
        return;
    }

    // Determine new position based on where the card was dropped
    const cards = Array.from(dropBody.querySelectorAll('.card:not(.dragging)'));
    let newPosition = cards.length; // default: end

    // Find insertion point based on mouse Y position
    const dropY = e.clientY;
    for (let i = 0; i < cards.length; i++) {
        const rect = cards[i].getBoundingClientRect();
        if (dropY < rect.top + rect.height / 2) {
            newPosition = i;
            break;
        }
    }

    // Optimistically reorder DOM
    if (newPosition === cards.length) {
        dropBody.appendChild(draggedCard);
    } else {
        dropBody.insertBefore(draggedCard, cards[newPosition]);
    }

    // Call PUT /api/priority
    const scopeType = 'story';
    const scopeId = activeFilters.story;
    reorderPriority(scopeType, scopeId, taskId, newPosition);
}

async function reorderPriority(scopeType, scopeId, taskId, newPosition) {
    try {
        const resp = await fetch('/api/priority', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                scope_type: scopeType,
                scope_id: scopeId,
                task_id: taskId,
                new_position: newPosition,
            }),
        });
        if (!resp.ok) {
            showToast('Failed to reorder priority');
            renderBoard(); // Revert
        }
    } catch (err) {
        console.error('reorderPriority error:', err);
        showToast('Network error reordering priority');
        renderBoard(); // Revert
    }
}

// ---------------------------------------------------------------------------
// Between-column drop: change task state
// ---------------------------------------------------------------------------

function handleBetweenColumnDrop(taskId, fromState, toState, dropBody, e) {
    // Optimistically move card to new column
    dropBody.appendChild(draggedCard);

    // Call PUT /api/tasks/:id with new state
    updateTaskState(taskId, toState);
}

async function updateTaskState(taskId, newState) {
    try {
        const resp = await fetch('/api/tasks/' + taskId, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ status: newState }),
        });
        if (!resp.ok) {
            showToast('Failed to update task state');
            renderBoard(); // Revert
        } else {
            // Update local task state
            const task = allTasks.find(t => t.id === taskId);
            if (task) task.state = newState;
            renderBoard();
        }
    } catch (err) {
        console.error('updateTaskState error:', err);
        showToast('Network error updating task state');
        renderBoard(); // Revert
    }
}

// ---------------------------------------------------------------------------
// Cross-altitude tooltip
// ---------------------------------------------------------------------------

let tooltipTimer = null;

function showCrossAltitudeTooltip(e) {
    const tooltip = document.getElementById('crossAltitudeTooltip');
    tooltip.classList.remove('hidden');
    tooltip.style.left = (e.clientX + 10) + 'px';
    tooltip.style.top = (e.clientY + 10) + 'px';
    if (tooltipTimer) clearTimeout(tooltipTimer);
    tooltipTimer = setTimeout(() => {
        tooltip.classList.add('hidden');
    }, 3000);
}
