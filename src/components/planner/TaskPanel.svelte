<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from '../ui/Modal.svelte';
  import TaskListModal from './TaskListModal.svelte';
  import {
    tasks, taskLists,
    loadTasks, loadTaskLists,
    createTask, updateTask, deleteTask, toggleTask, reorderTasks,
  } from '$lib/stores/plannerStore';
  import type { Task, TaskList } from '$lib/stores/plannerStore';

  // ─── Props ───────────────────────────────────────────────────────────────────

  let { initialWidth = 320 }: { initialWidth?: number } = $props();

  // ─── Constants ───────────────────────────────────────────────────────────────

  const PRIORITY_COLORS: Record<number, string> = {
    1: '#3b82f6',
    2: '#f59e0b',
    3: '#ef4444',
  };
  const PRIORITY_LABELS: Record<number, string> = {
    0: 'None', 1: 'Low', 2: 'Medium', 3: 'High',
  };
  const MONTHS_ABR = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];

  // ─── Panel resize ────────────────────────────────────────────────────────────

  let panelWidth     = $state(initialWidth);
  let isResizing     = $state(false);
  let resizeStartX   = 0;
  let resizeStartW   = 0;

  function onResizeMouseDown(e: MouseEvent) {
    isResizing   = true;
    resizeStartX = e.clientX;
    resizeStartW = panelWidth;
    e.preventDefault();
  }

  function onWindowMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    const delta = resizeStartX - e.clientX;
    panelWidth = Math.max(220, Math.min(560, resizeStartW + delta));
  }

  function onWindowMouseUp() { isResizing = false; }

  // ─── List selection ──────────────────────────────────────────────────────────

  let selectedListId = $state<string | null>(null);

  let filteredTasks = $derived(
    selectedListId === null
      ? $tasks
      : $tasks.filter(t => t.list_id === selectedListId)
  );

  let activeTasks = $derived(
    [...filteredTasks.filter(t => !t.completed)]
      .sort((a, b) => a.sort_order - b.sort_order)
  );

  let completedTasks = $derived(
    [...filteredTasks.filter(t => t.completed)]
      .sort((a, b) => (b.completed_at ?? '').localeCompare(a.completed_at ?? ''))
  );

  let selectedList = $derived($taskLists.find(l => l.id === selectedListId) ?? null);

  // ─── Quick-add ───────────────────────────────────────────────────────────────

  let quickTitle = $state('');

  async function handleQuickAdd(e: KeyboardEvent) {
    if (e.key !== 'Enter' || !quickTitle.trim()) return;
    const maxOrder = activeTasks.reduce((m, t) => Math.max(m, t.sort_order), -1);
    await createTask({
      title: quickTitle.trim(),
      priority: 0,
      completed: false,
      list_id: selectedListId,
      sort_order: maxOrder + 1,
    });
    quickTitle = '';
  }

  // ─── Drag-to-reorder ─────────────────────────────────────────────────────────

  let dragIndex     = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  function onDragStart(e: DragEvent, index: number) {
    dragIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(index));
    }
  }

  function onDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dragOverIndex = index;
  }

  function onDragLeave() { dragOverIndex = null; }

  function onDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    if (dragIndex === null || dragIndex === targetIndex) {
      dragIndex = null; dragOverIndex = null; return;
    }
    const newActiveIds = activeTasks.map(t => t.id);
    const [moved] = newActiveIds.splice(dragIndex, 1);
    newActiveIds.splice(targetIndex, 0, moved);

    const filteredSet = new Set(filteredTasks.map(t => t.id));
    const completedIds = completedTasks.map(t => t.id);
    const otherIds = $tasks.filter(t => !filteredSet.has(t.id)).map(t => t.id);
    reorderTasks([...newActiveIds, ...completedIds, ...otherIds]);

    dragIndex = null; dragOverIndex = null;
  }

  function onDragEnd() { dragIndex = null; dragOverIndex = null; }

  // ─── Context menu ────────────────────────────────────────────────────────────

  let menuTaskId = $state<string | null>(null);

  function toggleMenu(taskId: string, e: MouseEvent) {
    e.stopPropagation();
    menuTaskId = menuTaskId === taskId ? null : taskId;
  }

  function onWindowClick() { menuTaskId = null; }

  // ─── Completed section ───────────────────────────────────────────────────────

  let showCompleted = $state(false);

  // ─── Task list modal ─────────────────────────────────────────────────────────

  let showListModal = $state(false);
  let editList      = $state<TaskList | null>(null);

  function openNewList() { editList = null; showListModal = true; }
  function openEditList() { editList = selectedList; showListModal = true; }
  function closeListModal() { showListModal = false; editList = null; }

  // ─── Task detail modal ───────────────────────────────────────────────────────

  let showDetail    = $state(false);
  let detailTask    = $state<Task | null>(null);
  let detailTitle   = $state('');
  let detailDesc    = $state('');
  let detailDueDate = $state('');
  let detailDueTime = $state('');
  let detailPriority = $state(0);
  let detailListId  = $state<string | null>(null);
  let detailSaving  = $state(false);
  let detailError   = $state('');

  function openDetail(task: Task) {
    menuTaskId    = null;
    detailTask    = task;
    detailTitle   = task.title;
    detailDesc    = task.description ?? '';
    detailDueDate = task.due_date ?? '';
    detailDueTime = task.due_time ?? '';
    detailPriority = task.priority;
    detailListId  = task.list_id;
    detailError   = '';
    showDetail    = true;
  }

  function closeDetail() { showDetail = false; detailTask = null; }

  async function saveDetail() {
    if (!detailTask || !detailTitle.trim()) { detailError = 'Title is required.'; return; }
    detailSaving = true; detailError = '';
    try {
      await updateTask(detailTask.id, {
        title:       detailTitle.trim(),
        description: detailDesc.trim() || null,
        due_date:    detailDueDate || null,
        due_time:    detailDueTime || null,
        priority:    detailPriority,
        completed:   detailTask.completed,
        list_id:     detailListId,
        sort_order:  detailTask.sort_order,
      });
      closeDetail();
    } catch (e) {
      detailError = String(e);
    } finally {
      detailSaving = false;
    }
  }

  async function deleteFromDetail() {
    if (!detailTask) return;
    detailSaving = true;
    try {
      await deleteTask(detailTask.id);
      closeDetail();
    } catch (e) {
      detailError = String(e);
    } finally {
      detailSaving = false;
    }
  }

  // ─── Due badge ───────────────────────────────────────────────────────────────

  function getDueBadge(task: Task): { text: string; cls: string } | null {
    if (!task.due_date) return null;
    const todayStr    = new Date().toISOString().split('T')[0];
    const tomorrow    = new Date(Date.now() + 86_400_000);
    const tomorrowStr = tomorrow.toISOString().split('T')[0];
    if (task.due_date < todayStr)    return { text: 'Overdue',  cls: 'overdue'  };
    if (task.due_date === todayStr)  return { text: 'Today',    cls: 'today'    };
    if (task.due_date === tomorrowStr) return { text: 'Tomorrow', cls: 'tomorrow' };
    const [, m, d] = task.due_date.split('-').map(Number);
    return { text: `${MONTHS_ABR[m - 1]} ${d}`, cls: 'future' };
  }

  // ─── Toggle task completion ───────────────────────────────────────────────────

  async function handleToggle(task: Task) {
    await toggleTask(task.id);
  }

  async function handleDelete(taskId: string) {
    menuTaskId = null;
    await deleteTask(taskId);
  }

  // ─── Mount ───────────────────────────────────────────────────────────────────

  onMount(async () => {
    await Promise.all([loadTaskLists(), loadTasks(null, true)]);
  });
</script>

<svelte:window
  onmousemove={onWindowMouseMove}
  onmouseup={onWindowMouseUp}
  onclick={onWindowClick}
/>

<!-- Task list modal -->
<TaskListModal open={showListModal} list={editList} on:close={closeListModal} />

<!-- Task detail modal -->
<Modal open={showDetail} title={detailTask ? detailTask.title : 'Task'} width="440px" on:close={closeDetail}>
  <div class="detail-form">

    <div class="field">
      <label>Title</label>
      <input class="text-in" type="text" bind:value={detailTitle} placeholder="Task title" autofocus />
    </div>

    <div class="field">
      <label>Description</label>
      <textarea class="text-in textarea" bind:value={detailDesc} placeholder="Optional description" rows="3"></textarea>
    </div>

    <div class="field-row">
      <div class="field">
        <label>Due date</label>
        <input class="text-in" type="date" bind:value={detailDueDate} />
      </div>
      <div class="field">
        <label>Due time</label>
        <input class="text-in" type="time" bind:value={detailDueTime} />
      </div>
    </div>

    <div class="field-row">
      <div class="field">
        <label>Priority</label>
        <select class="text-in" bind:value={detailPriority}>
          <option value={0}>None</option>
          <option value={1}>Low</option>
          <option value={2}>Medium</option>
          <option value={3}>High</option>
        </select>
      </div>
      <div class="field">
        <label>List</label>
        <select class="text-in" bind:value={detailListId}>
          <option value={null}>No list</option>
          {#each $taskLists as list}
            <option value={list.id}>{list.name}</option>
          {/each}
        </select>
      </div>
    </div>

    {#if detailError}
      <p class="error">{detailError}</p>
    {/if}
  </div>

  <div slot="footer" class="footer-btns">
    <button class="btn-delete" onclick={deleteFromDetail} disabled={detailSaving}>Delete</button>
    <div class="spacer"></div>
    <button class="btn-cancel" onclick={closeDetail} disabled={detailSaving}>Cancel</button>
    <button class="btn-save" onclick={saveDetail} disabled={detailSaving || !detailTitle.trim()}>
      {detailSaving ? 'Saving…' : 'Save'}
    </button>
  </div>
</Modal>

<!-- ── Panel ── -->
<div class="panel" style:width="{panelWidth}px" class:resizing={isResizing}>

  <!-- Resize handle -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="resize-handle" onmousedown={onResizeMouseDown}></div>

  <div class="panel-inner">

    <!-- ── List selector row ── -->
    <div class="list-row">
      <div class="list-select-wrap">
        <select
          class="list-select"
          bind:value={selectedListId}
          onchange={() => { /* filtering is derived */ }}
        >
          <option value={null}>All Tasks</option>
          {#each $taskLists as list}
            <option value={list.id}>
              {list.name}
            </option>
          {/each}
        </select>
        {#if selectedListId && selectedList}
          <div class="list-color-dot" style:background={selectedList.color}></div>
        {/if}
      </div>

      <div class="list-actions">
        {#if selectedListId}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <button class="icon-btn" onclick={openEditList} title="Edit list">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
        {/if}
        <button class="icon-btn add-list-btn" onclick={openNewList} title="New list">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- ── Quick-add input ── -->
    <div class="quick-add">
      <svg class="quick-add-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      <input
        class="quick-input"
        type="text"
        placeholder="Add task…"
        bind:value={quickTitle}
        onkeydown={handleQuickAdd}
      />
    </div>

    <!-- ── Task list ── -->
    <div class="task-list">

      {#if activeTasks.length === 0 && completedTasks.length === 0}
        <div class="empty-state">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" opacity="0.35">
            <path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
          </svg>
          <span>No tasks. Type above to add one.</span>
        </div>
      {/if}

      <!-- Active tasks -->
      {#each activeTasks as task, i (task.id)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="task-item"
          class:drag-over={dragOverIndex === i && dragIndex !== i}
          class:dragging={dragIndex === i}
          draggable="true"
          ondragstart={(e) => onDragStart(e, i)}
          ondragover={(e) => onDragOver(e, i)}
          ondragleave={onDragLeave}
          ondrop={(e) => onDrop(e, i)}
          ondragend={onDragEnd}
        >
          <!-- Checkbox -->
          <!-- svelte-ignore a11y_interactive_supports_focus -->
          <div
            class="checkbox"
            role="checkbox"
            aria-checked="false"
            onclick={() => handleToggle(task)}
            onkeydown={(e) => e.key === ' ' && handleToggle(task)}
            tabindex="0"
          ></div>

          <!-- Task body -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="task-body" onclick={() => openDetail(task)}>
            <span class="task-title">{task.title}</span>
            <div class="task-meta">
              {#if task.priority > 0}
                <span class="priority-dot" style:background={PRIORITY_COLORS[task.priority]} title={PRIORITY_LABELS[task.priority]}></span>
              {/if}
              {#if getDueBadge(task)}
                {@const badge = getDueBadge(task)!}
                <span class="due-badge due-{badge.cls}">{badge.text}</span>
              {/if}
            </div>
          </div>

          <!-- Context menu button -->
          <div class="menu-wrap">
            <button class="menu-btn" onclick={(e) => toggleMenu(task.id, e)} title="More options">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
              </svg>
            </button>
            {#if menuTaskId === task.id}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="task-menu" onclick={(e) => e.stopPropagation()}>
                <button onclick={() => openDetail(task)}>Edit</button>
                <button class="danger" onclick={() => handleDelete(task.id)}>Delete</button>
              </div>
            {/if}
          </div>
        </div>
      {/each}

      <!-- Completed section -->
      {#if completedTasks.length > 0}
        <div class="completed-header">
          <button class="completed-toggle" onclick={() => (showCompleted = !showCompleted)}>
            <svg
              width="12" height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              style:transform={showCompleted ? 'rotate(90deg)' : 'rotate(0deg)'}
              style:transition="transform 0.15s"
            >
              <polyline points="9 18 15 12 9 6"/>
            </svg>
            Completed ({completedTasks.length})
          </button>
        </div>

        {#if showCompleted}
          {#each completedTasks as task (task.id)}
            <div class="task-item done">
              <!-- Checkbox (checked) -->
              <!-- svelte-ignore a11y_interactive_supports_focus -->
              <div
                class="checkbox checked"
                role="checkbox"
                aria-checked="true"
                onclick={() => handleToggle(task)}
                onkeydown={(e) => e.key === ' ' && handleToggle(task)}
                tabindex="0"
              >
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.5">
                  <polyline points="20 6 9 12 4 12"/>
                </svg>
              </div>

              <!-- Task body -->
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="task-body" onclick={() => openDetail(task)}>
                <span class="task-title done-title">{task.title}</span>
              </div>

              <!-- Context menu -->
              <div class="menu-wrap">
                <button class="menu-btn" onclick={(e) => toggleMenu(task.id, e)}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                  </svg>
                </button>
                {#if menuTaskId === task.id}
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="task-menu" onclick={(e) => e.stopPropagation()}>
                    <button onclick={() => openDetail(task)}>Edit</button>
                    <button class="danger" onclick={() => handleDelete(task.id)}>Delete</button>
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      {/if}

    </div>
  </div>
</div>

<style>
  /* ── Panel shell ─────────────────────────────────────────────────────────── */
  .panel {
    display: flex;
    flex-direction: row;
    height: 100%;
    flex-shrink: 0;
    position: relative;
    background: var(--bg-panel);
    border-left: 1px solid var(--border);
  }

  .panel.resizing { user-select: none; }

  /* ── Resize handle ───────────────────────────────────────────────────────── */
  .resize-handle {
    width: 5px;
    height: 100%;
    cursor: col-resize;
    flex-shrink: 0;
    position: relative;
    z-index: 2;
    transition: background 0.15s;
  }
  .resize-handle:hover,
  .panel.resizing .resize-handle {
    background: var(--border-focus);
  }

  /* ── Panel inner ─────────────────────────────────────────────────────────── */
  .panel-inner {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  /* ── List selector row ───────────────────────────────────────────────────── */
  .list-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .list-select-wrap {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .list-select {
    width: 100%;
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 5px 8px;
    padding-left: 22px;
    border-radius: 5px;
    font-size: 0.82rem;
    font-family: inherit;
    outline: none;
    cursor: pointer;
    appearance: none;
    transition: border-color 0.15s;
  }
  .list-select:focus { border-color: var(--border-focus); }

  .list-color-dot {
    position: absolute;
    left: 7px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    pointer-events: none;
    flex-shrink: 0;
  }

  .list-actions {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 5px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }
  .icon-btn:hover { background: var(--hover-bg); color: var(--text-main); }
  .add-list-btn:hover { color: var(--border-focus); }

  /* ── Quick-add ───────────────────────────────────────────────────────────── */
  .quick-add {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .quick-add-icon {
    color: var(--text-muted);
    flex-shrink: 0;
    opacity: 0.6;
  }

  .quick-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-main);
    font-size: 0.88rem;
    font-family: inherit;
    outline: none;
    min-width: 0;
  }
  .quick-input::placeholder { color: var(--text-muted); opacity: 0.7; }

  /* ── Task list ───────────────────────────────────────────────────────────── */
  .task-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px 0;
  }
  .task-list::-webkit-scrollbar { width: 5px; }
  .task-list::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  /* ── Empty state ─────────────────────────────────────────────────────────── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 40px 16px;
    color: var(--text-muted);
    font-size: 0.82rem;
    text-align: center;
  }

  /* ── Task item ───────────────────────────────────────────────────────────── */
  .task-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 7px 10px;
    position: relative;
    cursor: default;
    border-radius: 0;
    transition: background 0.1s;
  }
  .task-item:hover { background: var(--hover-bg); }
  .task-item:hover .menu-btn { opacity: 1; }

  .task-item.done { opacity: 0.6; }

  .task-item.drag-over::before {
    content: '';
    position: absolute;
    top: 0; left: 8px; right: 8px;
    height: 2px;
    background: var(--border-focus);
    border-radius: 1px;
  }

  .task-item.dragging { opacity: 0.4; }

  /* ── Checkbox ────────────────────────────────────────────────────────────── */
  .checkbox {
    width: 16px;
    height: 16px;
    border: 1.5px solid var(--border);
    border-radius: 4px;
    flex-shrink: 0;
    margin-top: 2px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    background: transparent;
  }
  .checkbox:hover { border-color: var(--border-focus); background: color-mix(in srgb, var(--border-focus) 12%, transparent); }
  .checkbox:focus { outline: none; border-color: var(--border-focus); }

  .checkbox.checked {
    background: var(--border-focus);
    border-color: var(--border-focus);
    color: #fff;
  }
  .checkbox.checked:hover { opacity: 0.8; }

  /* ── Task body ───────────────────────────────────────────────────────────── */
  .task-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    cursor: pointer;
  }

  .task-title {
    font-size: 0.85rem;
    color: var(--text-main);
    line-height: 1.4;
    word-break: break-word;
  }

  .done-title {
    text-decoration: line-through;
    color: var(--text-muted);
    opacity: 0.75;
    transition: opacity 0.2s;
  }

  .task-meta {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
  }

  /* ── Priority dot ────────────────────────────────────────────────────────── */
  .priority-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  /* ── Due badge ───────────────────────────────────────────────────────────── */
  .due-badge {
    font-size: 0.68rem;
    font-weight: 500;
    padding: 1px 5px;
    border-radius: 3px;
    line-height: 1.5;
  }
  .due-overdue  { background: rgba(239,68,68,0.15); color: #ef4444; }
  .due-today    { background: rgba(245,158,11,0.15); color: #f59e0b; }
  .due-tomorrow { background: color-mix(in srgb, var(--border) 50%, transparent); color: var(--text-muted); }
  .due-future   { background: color-mix(in srgb, var(--border) 40%, transparent); color: var(--text-muted); }

  /* ── Context menu button ─────────────────────────────────────────────────── */
  .menu-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .menu-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 3px 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    opacity: 0;
    transition: all 0.1s;
  }
  .menu-btn:hover { background: var(--hover-bg); color: var(--text-main); }

  .task-menu {
    position: absolute;
    right: 0;
    top: calc(100% + 2px);
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    z-index: 50;
    min-width: 110px;
    overflow: hidden;
  }

  .task-menu button {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 8px 12px;
    font-size: 0.82rem;
    font-family: inherit;
    color: var(--text-main);
    cursor: pointer;
    transition: background 0.1s;
  }
  .task-menu button:hover { background: var(--hover-bg); }
  .task-menu button.danger { color: #ef4444; }
  .task-menu button.danger:hover { background: rgba(239,68,68,0.1); }

  /* ── Completed section header ────────────────────────────────────────────── */
  .completed-header {
    padding: 8px 10px 2px;
  }

  .completed-toggle {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 600;
    font-family: inherit;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 4px;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    transition: all 0.15s;
  }
  .completed-toggle:hover { background: var(--hover-bg); color: var(--text-main); }

  /* ── Detail modal form ───────────────────────────────────────────────────── */
  .detail-form {
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    max-height: 60vh;
    padding-right: 4px;
  }
  .detail-form::-webkit-scrollbar { width: 6px; }
  .detail-form::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  .field { display: flex; flex-direction: column; gap: 6px; }
  .field label { font-size: 0.82rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }

  .field-row { display: flex; gap: 12px; }
  .field-row .field { flex: 1; min-width: 0; }

  .text-in {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px 10px;
    border-radius: 5px;
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
    box-sizing: border-box;
  }
  .text-in:focus { border-color: var(--border-focus); }
  .textarea { resize: none; min-height: 70px; }

  .error { color: #ef4444; font-size: 0.85rem; margin: 0; }

  .footer-btns { display: flex; align-items: center; gap: 8px; width: 100%; }
  .spacer { flex: 1; }
  button:not(.task-menu button):not(.menu-btn):not(.icon-btn):not(.quick-input):not(.completed-toggle):not(.checkbox) {
    padding: 7px 16px;
    border-radius: 5px;
    border: none;
    cursor: pointer;
    font-size: 0.88rem;
    font-weight: 500;
    font-family: inherit;
    transition: background 0.15s;
  }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-cancel { background: transparent; color: var(--text-muted); }
  .btn-cancel:hover:not(:disabled) { background: var(--hover-bg); color: var(--text-main); }
  .btn-save { background: var(--border-focus); color: #fff; }
  .btn-save:hover:not(:disabled) { opacity: 0.85; }
  .btn-delete { background: transparent; color: #ef4444; border: 1px solid #ef4444 !important; }
  .btn-delete:hover:not(:disabled) { background: #ef4444; color: #fff; }
</style>
