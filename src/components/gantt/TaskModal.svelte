<script lang="ts">
  import Modal from '../ui/Modal.svelte';
  import {
    tasks, createTask, updateTask, deleteTask,
    type GanttTask, type GanttTaskInput,
  } from '$lib/stores/ganttStore';

  const COLORS = [
    '#3b82f6', '#8b5cf6', '#ec4899', '#ef4444',
    '#f97316', '#eab308', '#22c55e', '#14b8a6',
    '#64748b', '#06b6d4',
  ];

  let {
    open          = false,
    task          = null,
    projectId     = '',
    defaultParentId = '',
    onclose       = () => {},
    onsaved       = (_t: GanttTask) => {},
    ondeleted     = () => {},
  }: {
    open?:           boolean;
    task?:           GanttTask | null;
    projectId?:      string;
    defaultParentId?: string;
    onclose?:        () => void;
    onsaved?:        (t: GanttTask) => void;
    ondeleted?:      () => void;
  } = $props();

  // ─── Form state ──────────────────────────────────────────────────────────────

  let title      = $state('');
  let startDate  = $state('');
  let endDate    = $state('');
  let progress   = $state(0);
  let color      = $state('#3b82f6');
  let groupName  = $state('');
  let parentId   = $state('');
  let assignedTo = $state('');
  let notes      = $state('');
  let saving     = $state(false);
  let error      = $state('');

  // ─── Sync when opened ────────────────────────────────────────────────────────

  $effect(() => {
    if (open) {
      title      = task?.title          ?? 'New Task';
      startDate  = task?.start_date     ?? isoToday();
      endDate    = task?.end_date       ?? addDays(isoToday(), 7);
      progress   = task?.progress       ?? 0;
      color      = task?.color          ?? '#3b82f6';
      groupName  = task?.group_name     ?? '';
      parentId   = task?.parent_task_id ?? defaultParentId ?? '';
      assignedTo = task?.assigned_to    ?? '';
      notes      = task?.notes          ?? '';
      error      = '';
    }
  });

  // ─── Helpers ─────────────────────────────────────────────────────────────────

  function isoToday(): string {
    return new Date().toISOString().split('T')[0];
  }

  function addDays(iso: string, n: number): string {
    const [y, m, d] = iso.split('-').map(Number);
    const dt = new Date(y, m - 1, d + n);
    return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, '0')}-${String(dt.getDate()).padStart(2, '0')}`;
  }

  // ─── Derived ─────────────────────────────────────────────────────────────────

  let duration = $derived(calcDuration(startDate, endDate));

  function calcDuration(s: string, e: string): string {
    if (!s || !e) return '';
    const [sy, sm, sd] = s.split('-').map(Number);
    const [ey, em, ed] = e.split('-').map(Number);
    const ms = new Date(ey, em - 1, ed).getTime() - new Date(sy, sm - 1, sd).getTime();
    const days = Math.round(ms / 86400000);
    if (days < 0) return '⚠ End is before start';
    return `${days + 1} day${days === 0 ? '' : 's'}`;
  }

  let parentOptions = $derived(
    $tasks.filter(t =>
      t.project_id === projectId &&
      t.id !== task?.id &&
      !t.parent_task_id   // only root tasks as parents
    )
  );

  let groupNames = $derived(
    [...new Set($tasks.filter(t => t.group_name).map(t => t.group_name!))]
  );

  // ─── Actions ─────────────────────────────────────────────────────────────────

  async function save() {
    if (!title.trim())       { error = 'Title is required.'; return; }
    if (!startDate)          { error = 'Start date is required.'; return; }
    if (!endDate)            { error = 'End date is required.'; return; }
    if (endDate < startDate) { error = 'End date must be on or after start date.'; return; }

    saving = true;
    error  = '';
    try {
      const input: GanttTaskInput = {
        project_id:     projectId,
        title:          title.trim(),
        start_date:     startDate,
        end_date:       endDate,
        progress,
        color,
        group_name:     groupName.trim() || null,
        parent_task_id: parentId || null,
        assigned_to:    assignedTo.trim() || null,
        notes:          notes.trim() || null,
        sort_order:     task?.sort_order ?? $tasks.filter(t => t.project_id === projectId).length,
      };
      const saved = task
        ? await updateTask(task.id, input)
        : await createTask(input);
      onsaved(saved);
      onclose();
    } catch (e: any) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function remove() {
    if (!task) return;
    if (!confirm(`Delete "${task.title}"?`)) return;
    saving = true;
    try {
      await deleteTask(task.id);
      ondeleted();
      onclose();
    } catch (e: any) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<Modal
  open={open}
  title={task ? 'Edit Task' : 'New Task'}
  width="500px"
  on:close={onclose}
>
  <div class="form">

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <label>
      <span>Title</span>
      <input type="text" bind:value={title} placeholder="Task name" autofocus />
    </label>

    <div class="row-two">
      <label>
        <span>Start Date</span>
        <input type="date" bind:value={startDate} />
      </label>
      <label>
        <span>End Date</span>
        <input type="date" bind:value={endDate} />
      </label>
    </div>

    {#if duration}
      <p class="duration" class:duration-warn={duration.startsWith('⚠')}>{duration}</p>
    {/if}

    <label>
      <span>Progress — {progress}%</span>
      <div class="slider-row">
        <input type="range" min="0" max="100" bind:value={progress} class="slider" />
        <span class="slider-val">{progress}%</span>
      </div>
    </label>

    <div class="color-field">
      <span class="field-label">Color</span>
      <div class="swatches">
        {#each COLORS as c}
          <button
            class="swatch"
            class:swatch-selected={color === c}
            style:background={c}
            onclick={() => (color = c)}
            aria-label={c}
          ></button>
        {/each}
      </div>
    </div>

    <label>
      <span>Group / Swimlane</span>
      <input
        type="text"
        bind:value={groupName}
        placeholder="Optional group name"
        list="tm-group-opts"
      />
      <datalist id="tm-group-opts">
        {#each groupNames as g}<option value={g}></option>{/each}
      </datalist>
    </label>

    <label>
      <span>Parent Task</span>
      <select bind:value={parentId}>
        <option value="">— None —</option>
        {#each parentOptions as p}
          <option value={p.id}>{p.title}</option>
        {/each}
      </select>
    </label>

    <label>
      <span>Assigned To</span>
      <input type="text" bind:value={assignedTo} placeholder="Name or email" />
    </label>

    <label>
      <span>Notes</span>
      <textarea bind:value={notes} rows="3" placeholder="Optional notes…"></textarea>
    </label>

  </div>

  <div slot="footer" class="footer">
    {#if task}
      <button class="btn-danger" onclick={remove} disabled={saving}>Delete</button>
      <div style="flex:1"></div>
    {/if}
    <button class="btn-cancel" onclick={onclose} disabled={saving}>Cancel</button>
    <button class="btn-save" onclick={save} disabled={saving || duration.startsWith('⚠')}>
      {saving ? 'Saving…' : 'Save'}
    </button>
  </div>
</Modal>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 13px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  label span, .field-label {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  input[type="text"],
  input[type="date"],
  select,
  textarea {
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.85rem;
    padding: 6px 9px;
    outline: none;
    resize: vertical;
    width: 100%;
    box-sizing: border-box;
  }
  input:focus, select:focus, textarea:focus {
    border-color: var(--border-focus);
  }
  input[type="date"] { cursor: pointer; }

  select { cursor: pointer; }

  .row-two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .duration {
    margin: -6px 0 0;
    font-size: 0.78rem;
    color: var(--text-muted);
  }
  .duration-warn { color: #ef4444; }

  /* Slider */
  .slider-row { display: flex; align-items: center; gap: 10px; }
  .slider { flex: 1; cursor: pointer; accent-color: var(--border-focus); }
  .slider-val { font-size: 0.78rem; color: var(--text-muted); width: 32px; text-align: right; flex-shrink: 0; }

  /* Color swatches */
  .color-field { display: flex; flex-direction: column; gap: 6px; }

  .swatches {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s, border-color 0.1s;
  }
  .swatch:hover { transform: scale(1.15); }
  .swatch-selected {
    border-color: #fff;
    outline: 2px solid var(--border-focus);
    outline-offset: 1px;
  }

  /* Error */
  .error {
    margin: 0;
    font-size: 0.82rem;
    color: #ef4444;
    background: color-mix(in srgb, #ef4444 10%, transparent);
    border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
    border-radius: 5px;
    padding: 6px 10px;
  }

  /* Footer */
  .footer {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  button {
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.84rem;
    font-weight: 500;
    padding: 6px 16px;
    transition: opacity 0.15s;
  }
  button:disabled { opacity: 0.45; cursor: not-allowed; }

  .btn-save   { background: var(--border-focus); color: #fff; }
  .btn-save:hover:not(:disabled) { opacity: 0.85; }
  .btn-cancel { background: var(--hover-bg); color: var(--text-muted); border: 1px solid var(--border); }
  .btn-cancel:hover:not(:disabled) { color: var(--text-main); }
  .btn-danger {
    background: color-mix(in srgb, #ef4444 15%, transparent);
    color: #ef4444;
    border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
  }
  .btn-danger:hover:not(:disabled) { background: color-mix(in srgb, #ef4444 25%, transparent); }
</style>
