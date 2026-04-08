<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    tasks, timeBlocks,
    loadTimeBlocks, loadTasks,
    createTimeBlock, updateTimeBlock, deleteTimeBlock,
  } from '$lib/stores/plannerStore';
  import type { TimeBlock, TimeBlockInput, Task } from '$lib/stores/plannerStore';

  let { date }: { date: Date } = $props();

  // ─── Constants ───────────────────────────────────────────────────────────────

  const HOUR_PX  = 64;
  const SNAP     = 15;
  const HOURS    = Array.from({ length: 24 }, (_, i) => i);
  const EDGE_PX  = 9; // resize handle zone height

  const PRESET_COLORS = [
    '#3b82f6', '#ef4444', '#10b981', '#f59e0b',
    '#8b5cf6', '#ec4899', '#06b6d4', '#f97316',
  ];

  const PRIORITY_COLORS: Record<number, string> = {
    1: '#3b82f6',
    2: '#f59e0b',
    3: '#ef4444',
  };

  // ─── Utilities ───────────────────────────────────────────────────────────────

  function fmtDate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  function toISO(ds: string, mins: number): string {
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return `${ds}T${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:00`;
  }

  function minsFromISO(iso: string): number {
    if (!iso.includes('T')) return 0;
    const [h, m] = iso.split('T')[1].slice(0, 5).split(':').map(Number);
    return h * 60 + m;
  }

  function fmtTime(mins: number): string {
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    const ampm = h < 12 ? 'AM' : 'PM';
    const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${h12}:${String(m).padStart(2, '0')} ${ampm}`;
  }

  function fmtTimeInput(mins: number): string {
    return `${String(Math.floor(mins / 60)).padStart(2, '0')}:${String(mins % 60).padStart(2, '0')}`;
  }

  function timeInputToMins(s: string): number {
    const [h, m] = s.split(':').map(Number);
    return (h || 0) * 60 + (m || 0);
  }

  function snapTo(mins: number): number {
    return Math.round(mins / SNAP) * SNAP;
  }

  function clamp(x: number, lo: number, hi: number): number {
    return Math.max(lo, Math.min(hi, x));
  }

  // ─── Derived data ─────────────────────────────────────────────────────────────

  let dateStr = $derived(fmtDate(date));

  let dayBlocks = $derived(
    $timeBlocks
      .filter(b => b.start_time.split('T')[0] === dateStr)
      .sort((a, b) => minsFromISO(a.start_time) - minsFromISO(b.start_time))
  );

  let taskMap = $derived(new Map($tasks.map(t => [t.id, t])));

  let scheduledTaskIds = $derived(
    new Set(dayBlocks.filter(b => b.task_id).map(b => b.task_id!))
  );

  let unscheduledTasks = $derived(
    $tasks
      .filter(t => !t.completed && !scheduledTaskIds.has(t.id))
      .sort((a, b) => (a.due_date ?? '9999').localeCompare(b.due_date ?? '9999'))
  );

  // ─── Reload when date changes ─────────────────────────────────────────────────

  $effect(() => {
    loadTimeBlocks(dateStr);
  });

  // ─── Current time ─────────────────────────────────────────────────────────────

  let currentMins = $state(getNowMins());
  let ticker: ReturnType<typeof setInterval>;

  function getNowMins(): number {
    const n = new Date();
    return n.getHours() * 60 + n.getMinutes();
  }

  function isToday(): boolean {
    return dateStr === fmtDate(new Date());
  }

  let bodyEl = $state<HTMLElement | null>(null);
  let colEl  = $state<HTMLElement | null>(null);

  $effect(() => {
    if (bodyEl) bodyEl.scrollTop = Math.max(0, currentMins * (HOUR_PX / 60) - 160);
  });

  onMount(async () => {
    ticker = setInterval(() => { currentMins = getNowMins(); }, 60_000);
    if ($tasks.length === 0) await loadTasks(null, true);
  });

  onDestroy(() => clearInterval(ticker));

  // ─── Y → minutes conversion ───────────────────────────────────────────────────

  function minsFromClientY(clientY: number): number {
    if (!colEl) return 0;
    const y   = clientY - colEl.getBoundingClientRect().top;
    return clamp(snapTo((y / HOUR_PX) * 60), 0, 23 * 60 + 45);
  }

  // ─── Drag state ───────────────────────────────────────────────────────────────

  type DragMode = 'create' | 'move' | 'resize-top' | 'resize-bottom';

  interface Drag {
    mode:       DragMode;
    startMins:  number;
    endMins:    number;
    blockId?:   string;
    origDur?:   number;   // for move: preserve duration
    offsetMins?: number;  // for move: click offset within block
    anchorMins?: number;  // for resize: fixed edge
  }

  let drag = $state<Drag | null>(null);
  let saving = $state(false);

  // Tracks whether mouse moved enough to count as a drag (not a click)
  let mouseDownPos = { x: 0, y: 0 };
  let mouseDownBlock: TimeBlock | null = null;

  // ─── Column mousedown (create) ────────────────────────────────────────────────

  function onColMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest('.tb-block')) return;
    const mins = minsFromClientY(e.clientY);
    drag = { mode: 'create', startMins: mins, endMins: mins + 60 };
    mouseDownPos = { x: e.clientX, y: e.clientY };
    mouseDownBlock = null;
    e.preventDefault();
  }

  // ─── Block mousedown (move / resize) ─────────────────────────────────────────

  function onBlockMouseDown(e: MouseEvent, block: TimeBlock) {
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();
    mouseDownPos   = { x: e.clientX, y: e.clientY };
    mouseDownBlock = block;
    const target  = e.target as HTMLElement;
    const startM  = minsFromISO(block.start_time);
    const endM    = minsFromISO(block.end_time);
    if (target.classList.contains('rh-top')) {
      drag = { mode: 'resize-top', blockId: block.id, startMins: startM, endMins: endM, anchorMins: endM };
    } else if (target.classList.contains('rh-bot')) {
      drag = { mode: 'resize-bottom', blockId: block.id, startMins: startM, endMins: endM, anchorMins: startM };
    } else {
      const clickMins = minsFromClientY(e.clientY);
      drag = {
        mode: 'move', blockId: block.id,
        startMins: startM, endMins: endM,
        origDur: endM - startM,
        offsetMins: clamp(clickMins - startM, 0, endM - startM),
      };
    }
  }

  // ─── Window mousemove ─────────────────────────────────────────────────────────

  function onMouseMove(e: MouseEvent) {
    if (!drag) return;
    const mins = minsFromClientY(e.clientY);
    if (drag.mode === 'create') {
      drag = { ...drag, endMins: Math.max(drag.startMins + SNAP, mins) };
    } else if (drag.mode === 'resize-top') {
      drag = { ...drag, startMins: clamp(mins, 0, drag.anchorMins! - SNAP) };
    } else if (drag.mode === 'resize-bottom') {
      drag = { ...drag, endMins: clamp(mins, drag.anchorMins! + SNAP, 24 * 60) };
    } else if (drag.mode === 'move') {
      const dur   = drag.origDur!;
      const start = clamp(mins - drag.offsetMins!, 0, 24 * 60 - dur);
      drag = { ...drag, startMins: start, endMins: start + dur };
    }
  }

  // ─── Window mouseup ───────────────────────────────────────────────────────────

  async function onMouseUp(e: MouseEvent) {
    if (!drag) return;
    const d = { ...drag };
    drag = null;

    const moveDist = Math.hypot(e.clientX - mouseDownPos.x, e.clientY - mouseDownPos.y);

    if (d.mode === 'create') {
      const lo = d.startMins;
      const hi = Math.max(d.endMins, lo + SNAP);
      if (hi - lo < SNAP) return;
      pendingStart = lo;
      pendingEnd   = hi;
      newTitle     = '';
      newColor     = PRESET_COLORS[0];
      placePopover(e.clientX, e.clientY, 260, 200, 'create');
      showCreatePop = true;
      showEditPop   = false;
    } else if (moveDist < 5 && mouseDownBlock) {
      // click on block — open edit
      openEditPop(mouseDownBlock, e.clientX, e.clientY);
    } else if (d.blockId) {
      // drag finished — persist
      const block = dayBlocks.find(b => b.id === d.blockId);
      if (!block) return;
      saving = true;
      try {
        await updateTimeBlock(block.id, {
          title:      block.title,
          start_time: toISO(dateStr, d.startMins),
          end_time:   toISO(dateStr, d.endMins),
          color:      block.color,
          event_id:   block.event_id,
          task_id:    block.task_id,
        });
      } finally {
        saving = false;
      }
    }

    mouseDownBlock = null;
  }

  // ─── Popover positioning helper ───────────────────────────────────────────────

  let popoverX = $state(0);
  let popoverY = $state(0);

  function placePopover(cx: number, cy: number, w: number, h: number, side: 'create' | 'block') {
    popoverX = clamp(cx + (side === 'block' ? 8 : 4), 8, window.innerWidth  - w - 8);
    popoverY = clamp(cy - 20,                          8, window.innerHeight - h - 8);
  }

  // ─── Create popover ───────────────────────────────────────────────────────────

  let showCreatePop = $state(false);
  let pendingStart  = $state(0);
  let pendingEnd    = $state(60);
  let newTitle      = $state('');
  let newColor      = $state(PRESET_COLORS[0]);

  async function confirmCreate() {
    if (!newTitle.trim()) return;
    saving = true;
    try {
      await createTimeBlock({
        title:      newTitle.trim(),
        start_time: toISO(dateStr, pendingStart),
        end_time:   toISO(dateStr, pendingEnd),
        color:      newColor,
      });
      showCreatePop = false;
    } finally {
      saving = false;
    }
  }

  // ─── Edit popover ─────────────────────────────────────────────────────────────

  let showEditPop  = $state(false);
  let editBlockId  = $state('');
  let editTitle    = $state('');
  let editColor    = $state(PRESET_COLORS[0]);
  let editStart    = $state(0);
  let editEnd      = $state(60);

  function openEditPop(block: TimeBlock, cx: number, cy: number) {
    editBlockId = block.id;
    editTitle   = block.title;
    editColor   = block.color;
    editStart   = minsFromISO(block.start_time);
    editEnd     = minsFromISO(block.end_time);
    placePopover(cx, cy, 260, 240, 'block');
    showEditPop   = true;
    showCreatePop = false;
    closeCtx();
  }

  async function confirmEdit() {
    const block = dayBlocks.find(b => b.id === editBlockId);
    if (!block || !editTitle.trim()) return;
    saving = true;
    try {
      await updateTimeBlock(block.id, {
        title:      editTitle.trim(),
        start_time: toISO(dateStr, editStart),
        end_time:   toISO(dateStr, Math.max(editEnd, editStart + SNAP)),
        color:      editColor,
        event_id:   block.event_id,
        task_id:    block.task_id,
      });
      showEditPop = false;
    } finally {
      saving = false;
    }
  }

  // ─── Context menu ─────────────────────────────────────────────────────────────

  let ctxBlockId    = $state<string | null>(null);
  let ctxX          = $state(0);
  let ctxY          = $state(0);
  let showLinkPicker = $state(false);

  function openCtx(block: TimeBlock, e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    ctxBlockId    = block.id;
    ctxX          = clamp(e.clientX, 8, window.innerWidth  - 190);
    ctxY          = clamp(e.clientY, 8, window.innerHeight - 200);
    showLinkPicker = false;
    showCreatePop  = false;
    showEditPop    = false;
  }

  function closeCtx() {
    ctxBlockId     = null;
    showLinkPicker = false;
  }

  function ctxEdit() {
    const block = dayBlocks.find(b => b.id === ctxBlockId);
    if (block) openEditPop(block, ctxX, ctxY);
    closeCtx();
  }

  async function ctxDelete() {
    if (!ctxBlockId) return;
    saving = true;
    try {
      await deleteTimeBlock(ctxBlockId);
      closeCtx();
    } finally {
      saving = false;
    }
  }

  async function ctxUnlink() {
    const block = dayBlocks.find(b => b.id === ctxBlockId);
    if (!block) return;
    saving = true;
    try {
      await updateTimeBlock(block.id, {
        title: block.title, start_time: block.start_time,
        end_time: block.end_time, color: block.color,
        event_id: block.event_id, task_id: null,
      });
      closeCtx();
    } finally {
      saving = false;
    }
  }

  async function ctxLinkTask(task: Task) {
    const block = dayBlocks.find(b => b.id === ctxBlockId);
    if (!block) return;
    saving = true;
    try {
      await updateTimeBlock(block.id, {
        title:      block.title,
        start_time: block.start_time,
        end_time:   block.end_time,
        color:      task.priority > 0 ? PRIORITY_COLORS[task.priority] : block.color,
        event_id:   block.event_id,
        task_id:    task.id,
      });
      closeCtx();
    } finally {
      saving = false;
    }
  }

  // ─── Task drag → grid (HTML5 DnD) ────────────────────────────────────────────

  let taskDragId      = $state<string | null>(null);
  let taskDropMins    = $state<number | null>(null);

  function onTaskDragStart(e: DragEvent, taskId: string) {
    taskDragId = taskId;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'copy';
      e.dataTransfer.setData('text/plain', taskId);
    }
  }

  function onGridDragOver(e: DragEvent) {
    if (!taskDragId) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
    taskDropMins = minsFromClientY(e.clientY);
  }

  function onGridDragLeave(e: DragEvent) {
    // only clear if leaving the column entirely
    if (!(e.relatedTarget as HTMLElement)?.closest('.day-col')) {
      taskDropMins = null;
    }
  }

  async function onGridDrop(e: DragEvent) {
    e.preventDefault();
    if (!taskDragId) return;
    const task = taskMap.get(taskDragId);
    if (!task) { taskDragId = null; taskDropMins = null; return; }
    const startM = taskDropMins ?? minsFromClientY(e.clientY);
    const endM   = clamp(startM + 60, startM + SNAP, 24 * 60);
    saving = true;
    try {
      await createTimeBlock({
        title:      task.title,
        start_time: toISO(dateStr, startM),
        end_time:   toISO(dateStr, endM),
        color:      task.priority > 0 ? PRIORITY_COLORS[task.priority] : PRESET_COLORS[0],
        task_id:    task.id,
      });
    } finally {
      saving = false;
      taskDragId   = null;
      taskDropMins = null;
    }
  }

  // ─── Close overlays on outside click ─────────────────────────────────────────

  function onWindowClick(e: MouseEvent) {
    const t = e.target as HTMLElement;
    if (showCreatePop && !t.closest('.pop-overlay')) showCreatePop = false;
    if (showEditPop   && !t.closest('.pop-overlay')) showEditPop   = false;
    if (ctxBlockId    && !t.closest('.ctx-menu'))    closeCtx();
  }
</script>

<svelte:window
  onmousemove={onMouseMove}
  onmouseup={onMouseUp}
  onclick={onWindowClick}
/>

<div class="tbv-wrap" class:is-dragging={drag !== null}>

  <!-- ── Unscheduled tasks panel ── -->
  <div class="unsch-panel">
    <div class="unsch-head">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M9 11l3 3L22 4"/>
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
      </svg>
      Unscheduled
    </div>
    <div class="unsch-list">
      {#if unscheduledTasks.length === 0}
        <p class="unsch-empty">All scheduled!</p>
      {:else}
        {#each unscheduledTasks as task (task.id)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="task-chip"
            draggable="true"
            ondragstart={(e) => onTaskDragStart(e, task.id)}
            ondragend={() => { taskDragId = null; taskDropMins = null; }}
            title="Drag to grid to schedule"
          >
            {#if task.priority > 0}
              <span class="chip-dot" style:background={PRIORITY_COLORS[task.priority]}></span>
            {/if}
            <span class="chip-title">{task.title}</span>
            {#if task.due_date}
              <span class="chip-due" class:overdue={task.due_date < dateStr}>{task.due_date.slice(5)}</span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- ── Time grid ── -->
  <div class="grid-area">

    <!-- Scrollable body -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="body" bind:this={bodyEl}>
      <div class="body-inner" style:height="{24 * HOUR_PX}px">

        <!-- Gutter -->
        <div class="gutter">
          {#each HOURS as h}
            <div class="g-label" style:top="{h * HOUR_PX}px">
              {#if h > 0}{fmtTime(h * 60)}{/if}
            </div>
          {/each}
        </div>

        <!-- Hour lines -->
        <div class="lines-layer">
          {#each HOURS as h}
            <div class="h-line" style:top="{h * HOUR_PX}px"></div>
          {/each}
        </div>

        <!-- Block column -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="day-col"
          bind:this={colEl}
          onmousedown={onColMouseDown}
          ondragover={onGridDragOver}
          ondragleave={onGridDragLeave}
          ondrop={onGridDrop}
        >
          <!-- Current time line -->
          {#if isToday()}
            <div class="now-line" style:top="{currentMins * (HOUR_PX / 60)}px">
              <div class="now-dot"></div>
            </div>
          {/if}

          <!-- Create drag preview -->
          {#if drag?.mode === 'create'}
            <div
              class="tb-preview"
              style:top="{drag.startMins * (HOUR_PX / 60)}px"
              style:height="{Math.max(SNAP, drag.endMins - drag.startMins) * (HOUR_PX / 60)}px"
            >
              <span>{fmtTime(drag.startMins)} – {fmtTime(drag.endMins)}</span>
            </div>
          {/if}

          <!-- Task drop preview -->
          {#if taskDropMins !== null}
            <div
              class="tb-preview task-drop"
              style:top="{taskDropMins * (HOUR_PX / 60)}px"
              style:height="{60 * (HOUR_PX / 60)}px"
            ></div>
          {/if}

          <!-- Time blocks -->
          {#each dayBlocks as block (block.id)}
            {@const isBlockDragging = drag?.blockId === block.id}
            {@const dispStart = isBlockDragging ? drag!.startMins : minsFromISO(block.start_time)}
            {@const dispEnd   = isBlockDragging ? drag!.endMins   : minsFromISO(block.end_time)}
            {@const bTop = dispStart * (HOUR_PX / 60)}
            {@const bH   = Math.max(24, (dispEnd - dispStart) * (HOUR_PX / 60))}
            {@const linkedTask = block.task_id ? taskMap.get(block.task_id) : null}
            {@const isDone = linkedTask?.completed ?? false}

            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="tb-block"
              class:is-done={isDone}
              class:is-dragging-block={isBlockDragging}
              style:top="{bTop}px"
              style:height="{bH}px"
              style:--bc={block.color}
              onmousedown={(e) => onBlockMouseDown(e, block)}
              oncontextmenu={(e) => openCtx(block, e)}
            >
              <!-- Resize handle: top -->
              <div class="rh-top"></div>

              <!-- Content -->
              <div class="block-inner">
                <div class="block-title">{block.title}</div>

                {#if linkedTask}
                  <div class="block-task-link">
                    <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                      <polyline points="20 6 9 12 4 12"/>
                    </svg>
                    {linkedTask.title}
                  </div>
                {/if}

                {#if bH >= 36}
                  <div class="block-time">
                    {fmtTime(dispStart)} – {fmtTime(dispEnd)}
                  </div>
                {/if}

                {#if isDone}
                  <div class="done-overlay">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                      <polyline points="20 6 9 12 4 12"/>
                      <circle cx="12" cy="12" r="10"/>
                    </svg>
                  </div>
                {/if}
              </div>

              <!-- Resize handle: bottom -->
              <div class="rh-bot"></div>
            </div>
          {/each}

        </div><!-- end day-col -->
      </div><!-- end body-inner -->
    </div><!-- end body -->

    <!-- ── Create popover ── -->
    {#if showCreatePop}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="pop-overlay"
        style:left="{popoverX}px"
        style:top="{popoverY}px"
        onclick={(e) => e.stopPropagation()}
      >
        <div class="pop-time-badge">{fmtTime(pendingStart)} – {fmtTime(pendingEnd)}</div>
        <input
          class="pop-input"
          type="text"
          bind:value={newTitle}
          placeholder="Block title"
          autofocus
          onkeydown={(e) => {
            if (e.key === 'Enter') confirmCreate();
            if (e.key === 'Escape') showCreatePop = false;
          }}
        />
        <div class="pop-swatches">
          {#each PRESET_COLORS as c}
            <button class="sw" class:active={newColor === c} style:background={c} onclick={() => (newColor = c)}></button>
          {/each}
        </div>
        <div class="pop-footer">
          <button class="btn-cancel" onclick={() => (showCreatePop = false)}>Cancel</button>
          <button class="btn-save" onclick={confirmCreate} disabled={!newTitle.trim() || saving}>
            {saving ? '…' : 'Create'}
          </button>
        </div>
      </div>
    {/if}

    <!-- ── Edit popover ── -->
    {#if showEditPop}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="pop-overlay"
        style:left="{popoverX}px"
        style:top="{popoverY}px"
        onclick={(e) => e.stopPropagation()}
      >
        <div class="pop-header">
          <span>Edit Block</span>
          <button class="pop-x" onclick={() => (showEditPop = false)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <input class="pop-input" type="text" bind:value={editTitle} autofocus />
        <div class="pop-times">
          <div class="pop-time-field">
            <label>Start</label>
            <input
              class="pop-time-in"
              type="time"
              value={fmtTimeInput(editStart)}
              onchange={(e) => (editStart = timeInputToMins(e.currentTarget.value))}
            />
          </div>
          <div class="pop-time-field">
            <label>End</label>
            <input
              class="pop-time-in"
              type="time"
              value={fmtTimeInput(editEnd)}
              onchange={(e) => (editEnd = timeInputToMins(e.currentTarget.value))}
            />
          </div>
        </div>
        <div class="pop-swatches">
          {#each PRESET_COLORS as c}
            <button class="sw" class:active={editColor === c} style:background={c} onclick={() => (editColor = c)}></button>
          {/each}
        </div>
        <div class="pop-footer">
          <button class="btn-cancel" onclick={() => (showEditPop = false)}>Cancel</button>
          <button class="btn-save" onclick={confirmEdit} disabled={!editTitle.trim() || saving}>
            {saving ? '…' : 'Save'}
          </button>
        </div>
      </div>
    {/if}

    <!-- ── Context menu ── -->
    {#if ctxBlockId}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="ctx-menu"
        style:left="{ctxX}px"
        style:top="{ctxY}px"
        onclick={(e) => e.stopPropagation()}
      >
        {#if !showLinkPicker}
          <button onclick={ctxEdit}>Edit</button>
          {#if dayBlocks.find(b => b.id === ctxBlockId)?.task_id}
            <button onclick={ctxUnlink}>Unlink Task</button>
          {:else}
            <button onclick={() => (showLinkPicker = true)}>
              Link to Task
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" style="margin-left:auto">
                <polyline points="9 18 15 12 9 6"/>
              </svg>
            </button>
          {/if}
          <div class="ctx-sep"></div>
          <button class="ctx-danger" onclick={ctxDelete}>Delete</button>
        {:else}
          <div class="ctx-back">
            <button onclick={() => (showLinkPicker = false)}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="15 18 9 12 15 6"/>
              </svg>
              Link to Task
            </button>
          </div>
          <div class="link-picker">
            {#each $tasks.filter(t => !t.completed) as task (task.id)}
              <button onclick={() => ctxLinkTask(task)}>
                {#if task.priority > 0}
                  <span class="lp-dot" style:background={PRIORITY_COLORS[task.priority]}></span>
                {/if}
                <span class="lp-title">{task.title}</span>
              </button>
            {:else}
              <p class="lp-empty">No active tasks</p>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

  </div><!-- end grid-area -->
</div><!-- end tbv-wrap -->

<style>
  /* ── Wrapper ─────────────────────────────────────────────────────────────── */
  .tbv-wrap {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--bg-main);
  }

  .tbv-wrap.is-dragging { user-select: none; cursor: grabbing; }

  /* ── Unscheduled panel ───────────────────────────────────────────────────── */
  .unsch-panel {
    width: 176px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    background: var(--bg-panel);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .unsch-head {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 12px;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .unsch-list {
    flex: 1;
    overflow-y: auto;
    padding: 6px 6px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .unsch-list::-webkit-scrollbar { width: 4px; }
  .unsch-list::-webkit-scrollbar-thumb { background: var(--border); border-radius: 2px; }

  .unsch-empty {
    color: var(--text-muted);
    font-size: 0.75rem;
    text-align: center;
    margin: 16px 0;
    line-height: 1.5;
  }

  .task-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 8px;
    border-radius: 5px;
    background: var(--bg-main);
    border: 1px solid var(--border);
    cursor: grab;
    transition: all 0.1s;
    user-select: none;
  }
  .task-chip:hover { border-color: var(--border-focus); background: var(--hover-bg); }
  .task-chip:active { cursor: grabbing; }

  .chip-dot {
    width: 7px; height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .chip-title {
    font-size: 0.75rem;
    color: var(--text-main);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .chip-due {
    font-size: 0.65rem;
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .chip-due.overdue { color: #ef4444; }

  /* ── Grid area ───────────────────────────────────────────────────────────── */
  .grid-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
    min-width: 0;
  }

  /* ── Scrollable body ─────────────────────────────────────────────────────── */
  .body {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
  .body::-webkit-scrollbar { width: 6px; }
  .body::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  .body-inner {
    display: flex;
    position: relative;
  }

  /* ── Gutter ──────────────────────────────────────────────────────────────── */
  .gutter {
    width: 52px;
    flex-shrink: 0;
    position: relative;
  }

  .g-label {
    position: absolute;
    right: 8px;
    transform: translateY(-50%);
    font-size: 0.65rem;
    color: var(--text-muted);
    white-space: nowrap;
    user-select: none;
    pointer-events: none;
  }

  /* ── Hour lines ──────────────────────────────────────────────────────────── */
  .lines-layer {
    position: absolute;
    left: 52px;
    right: 0;
    top: 0;
    bottom: 0;
    pointer-events: none;
  }

  .h-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--border);
    opacity: 0.45;
  }

  /* ── Day column ──────────────────────────────────────────────────────────── */
  .day-col {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--border);
    cursor: crosshair;
    min-width: 0;
  }

  /* ── Current time ────────────────────────────────────────────────────────── */
  .now-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: #ef4444;
    pointer-events: none;
    z-index: 4;
  }
  .now-dot {
    position: absolute;
    left: -4px;
    top: -4px;
    width: 10px;
    height: 10px;
    background: #ef4444;
    border-radius: 50%;
  }

  /* ── Drag preview ────────────────────────────────────────────────────────── */
  .tb-preview {
    position: absolute;
    left: 4px;
    right: 4px;
    background: color-mix(in srgb, var(--border-focus) 25%, transparent);
    border: 1.5px dashed var(--border-focus);
    border-radius: 5px;
    pointer-events: none;
    z-index: 2;
    display: flex;
    align-items: flex-start;
    padding: 3px 6px;
  }

  .tb-preview span {
    font-size: 0.68rem;
    color: var(--border-focus);
    font-weight: 600;
  }

  .tb-preview.task-drop {
    background: color-mix(in srgb, #10b981 22%, transparent);
    border-color: #10b981;
  }

  /* ── Time block ──────────────────────────────────────────────────────────── */
  .tb-block {
    position: absolute;
    left: 4px;
    right: 4px;
    border-radius: 6px;
    background: color-mix(in srgb, var(--bc, #3b82f6) 85%, black 15%);
    border-left: 3px solid var(--bc, #3b82f6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 1;
    box-sizing: border-box;
    transition: box-shadow 0.1s, opacity 0.15s;
  }

  .tb-block:hover { box-shadow: 0 2px 10px rgba(0,0,0,0.4); z-index: 2; }

  .tb-block.is-done { opacity: 0.55; }

  .tb-block.is-dragging-block {
    opacity: 0.75;
    box-shadow: 0 6px 20px rgba(0,0,0,0.5);
    z-index: 5;
  }

  /* Resize handles */
  .rh-top, .rh-bot {
    height: 8px;
    flex-shrink: 0;
    cursor: ns-resize;
    position: relative;
    z-index: 1;
  }
  .rh-top { border-radius: 6px 6px 0 0; }
  .rh-bot { border-radius: 0 0 6px 6px; margin-top: auto; }

  .tb-block:hover .rh-top,
  .tb-block:hover .rh-bot {
    background: rgba(255,255,255,0.15);
  }

  /* Block content */
  .block-inner {
    flex: 1;
    padding: 0 6px 3px;
    overflow: hidden;
    cursor: grab;
    position: relative;
    min-height: 0;
  }
  .block-inner:active { cursor: grabbing; }

  .block-title {
    font-size: 0.76rem;
    font-weight: 700;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .block-task-link {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 0.66rem;
    color: rgba(255,255,255,0.8);
    margin-top: 1px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .block-time {
    font-size: 0.63rem;
    color: rgba(255,255,255,0.7);
    margin-top: 1px;
  }

  /* Completed overlay */
  .done-overlay {
    position: absolute;
    bottom: 4px;
    right: 6px;
    color: rgba(255,255,255,0.6);
    display: flex;
    align-items: center;
    pointer-events: none;
  }

  /* ── Popovers (fixed position) ───────────────────────────────────────────── */
  .pop-overlay {
    position: fixed;
    width: 256px;
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(0,0,0,0.4);
    padding: 12px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 9px;
  }

  .pop-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .pop-header span {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-main);
  }

  .pop-x {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    display: flex;
    border-radius: 3px;
    transition: all 0.1s;
  }
  .pop-x:hover { background: var(--hover-bg); color: var(--text-main); }

  .pop-time-badge {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--border-focus);
  }

  .pop-input {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 7px 9px;
    border-radius: 5px;
    font-size: 0.88rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
    box-sizing: border-box;
  }
  .pop-input:focus { border-color: var(--border-focus); }

  .pop-times {
    display: flex;
    gap: 8px;
  }

  .pop-time-field {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .pop-time-field label {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .pop-time-in {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 5px 7px;
    border-radius: 4px;
    font-size: 0.82rem;
    font-family: inherit;
    outline: none;
    width: 100%;
    box-sizing: border-box;
  }
  .pop-time-in:focus { border-color: var(--border-focus); }

  .pop-swatches {
    display: flex;
    gap: 6px;
  }

  .sw {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.1s, border-color 0.1s;
    flex-shrink: 0;
  }
  .sw:hover { transform: scale(1.15); }
  .sw.active { border-color: var(--text-main); box-shadow: 0 0 0 2px var(--bg-header); }

  .pop-footer {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  .btn-cancel {
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 5px 10px;
    border-radius: 4px;
    font-size: 0.82rem;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.1s;
  }
  .btn-cancel:hover { background: var(--hover-bg); color: var(--text-main); }

  .btn-save {
    background: var(--border-focus);
    border: none;
    color: #fff;
    padding: 5px 12px;
    border-radius: 4px;
    font-size: 0.82rem;
    font-family: inherit;
    cursor: pointer;
    transition: opacity 0.1s;
  }
  .btn-save:hover:not(:disabled) { opacity: 0.85; }
  .btn-save:disabled { opacity: 0.45; cursor: not-allowed; }

  /* ── Context menu ────────────────────────────────────────────────────────── */
  .ctx-menu {
    position: fixed;
    min-width: 170px;
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 7px;
    box-shadow: 0 5px 16px rgba(0,0,0,0.4);
    z-index: 100;
    overflow: hidden;
  }

  .ctx-menu > button,
  .ctx-back > button {
    display: flex;
    align-items: center;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 8px 13px;
    font-size: 0.82rem;
    font-family: inherit;
    color: var(--text-main);
    cursor: pointer;
    transition: background 0.1s;
    gap: 6px;
  }

  .ctx-menu > button:hover,
  .ctx-back > button:hover { background: var(--hover-bg); }

  .ctx-danger { color: #ef4444 !important; }
  .ctx-danger:hover { background: rgba(239,68,68,0.1) !important; }

  .ctx-sep {
    height: 1px;
    background: var(--border);
    margin: 2px 0;
  }

  .ctx-back {
    border-bottom: 1px solid var(--border);
  }

  .ctx-back button {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  /* Link picker */
  .link-picker {
    max-height: 200px;
    overflow-y: auto;
  }
  .link-picker::-webkit-scrollbar { width: 4px; }
  .link-picker::-webkit-scrollbar-thumb { background: var(--border); border-radius: 2px; }

  .link-picker button {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 7px 13px;
    font-size: 0.82rem;
    font-family: inherit;
    color: var(--text-main);
    cursor: pointer;
    transition: background 0.1s;
  }
  .link-picker button:hover { background: var(--hover-bg); }

  .lp-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .lp-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lp-empty {
    font-size: 0.78rem;
    color: var(--text-muted);
    padding: 8px 13px;
    margin: 0;
  }
</style>
