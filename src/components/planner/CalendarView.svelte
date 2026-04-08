<script lang="ts">
  import { slide } from 'svelte/transition';
  import { events, selectedDate } from '$lib/stores/plannerStore';
  import EventModal from './EventModal.svelte';
  import type { Event as PlannerEvent } from '$lib/stores/plannerStore';

  let { year, month }: { year: number; month: number } = $props();

  const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const MONTH_NAMES = ['January','February','March','April','May','June',
                       'July','August','September','October','November','December'];

  // ─── Types ──────────────────────────────────────────────────────────────────

  interface CalendarDay {
    date: Date;
    dateStr: string;
    dayNum: number;
    isCurrentMonth: boolean;
    isToday: boolean;
  }

  type Event = PlannerEvent;

  // ─── Utilities ───────────────────────────────────────────────────────────────

  function fmtDate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  function fmtTime(iso: string): string {
    if (!iso.includes('T')) return 'All day';
    const t = iso.split('T')[1].replace('Z', '').slice(0, 5);
    const [h, m] = t.split(':').map(Number);
    const ampm = h < 12 ? 'AM' : 'PM';
    const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${h12}:${String(m).padStart(2, '0')} ${ampm}`;
  }

  function fmtDisplayDate(dateStr: string): string {
    const [y, mo, d] = dateStr.split('-').map(Number);
    return `${MONTH_NAMES[mo - 1]} ${d}, ${y}`;
  }

  // ─── Calendar grid ───────────────────────────────────────────────────────────

  let grid = $derived(buildGrid(year, month));

  function buildGrid(y: number, m: number): CalendarDay[][] {
    const todayStr = fmtDate(new Date());
    const firstOfMonth = new Date(y, m, 1);
    const lastOfMonth  = new Date(y, m + 1, 0);

    const start = new Date(firstOfMonth);
    start.setDate(start.getDate() - start.getDay()); // back to Sunday

    const end = new Date(lastOfMonth);
    end.setDate(end.getDate() + (6 - end.getDay())); // forward to Saturday

    const weeks: CalendarDay[][] = [];
    const cur = new Date(start);
    while (cur <= end) {
      const week: CalendarDay[] = [];
      for (let i = 0; i < 7; i++) {
        const d = new Date(cur);
        const str = fmtDate(d);
        week.push({
          date: d, dateStr: str,
          dayNum: d.getDate(),
          isCurrentMonth: d.getMonth() === m,
          isToday: str === todayStr,
        });
        cur.setDate(cur.getDate() + 1);
      }
      weeks.push(week);
    }
    return weeks;
  }

  // ─── Event map ───────────────────────────────────────────────────────────────

  let eventMap = $derived(buildEventMap($events));

  function buildEventMap(evts: Event[]): Map<string, Event[]> {
    const map = new Map<string, Event[]>();
    for (const evt of evts) {
      const startStr = evt.start_time.split('T')[0];
      const endStr   = evt.end_time.split('T')[0];
      const cur = new Date(startStr + 'T00:00:00');
      const end = new Date(endStr   + 'T00:00:00');
      while (cur <= end) {
        const s = fmtDate(cur);
        if (!map.has(s)) map.set(s, []);
        map.get(s)!.push(evt);
        cur.setDate(cur.getDate() + 1);
      }
    }
    return map;
  }

  function eventsForDay(dateStr: string): Event[] {
    return eventMap.get(dateStr) ?? [];
  }

  // ─── Drag-to-create ──────────────────────────────────────────────────────────

  let isMouseDown = $state(false);
  let dragStart   = $state<string | null>(null);
  let dragEnd     = $state<string | null>(null);

  function inDragRange(dateStr: string): boolean {
    if (!isMouseDown || !dragStart) return false;
    const d = dragEnd ?? dragStart;
    const [mn, mx] = dragStart <= d ? [dragStart, d] : [d, dragStart];
    return dateStr >= mn && dateStr <= mx;
  }

  function onCellMouseDown(dateStr: string) {
    isMouseDown = true;
    dragStart = dateStr;
    dragEnd   = dateStr;
  }

  function onCellMouseEnter(dateStr: string) {
    if (isMouseDown) dragEnd = dateStr;
  }

  function onWindowMouseUp() {
    if (!isMouseDown || !dragStart) return;
    const d = dragEnd ?? dragStart;
    const [mn, mx] = dragStart <= d ? [dragStart, d] : [d, dragStart];
    isMouseDown = false;
    if (mn !== mx) {
      // Multi-day drag → open create modal
      openNewModal(mn, mx);
    }
    dragStart = null;
    dragEnd   = null;
  }

  // ─── Day click / selection ───────────────────────────────────────────────────

  function onCellClick(dateStr: string) {
    $selectedDate = $selectedDate === dateStr ? null : dateStr;
  }

  function onCellDblClick(dateStr: string) {
    openNewModal(dateStr, dateStr);
  }

  // ─── Event modal ─────────────────────────────────────────────────────────────

  let showModal  = $state(false);
  let editEvent  = $state<Event | null>(null);
  let modalStart = $state('');
  let modalEnd   = $state('');

  function openNewModal(start: string, end: string) {
    editEvent  = null;
    modalStart = start;
    modalEnd   = end;
    showModal  = true;
  }

  function openEditModal(evt: Event, e: MouseEvent) {
    e.stopPropagation();
    editEvent  = evt;
    modalStart = evt.start_time.split('T')[0];
    modalEnd   = evt.end_time.split('T')[0];
    showModal  = true;
  }

  function closeModal() { showModal = false; editEvent = null; }
</script>

<svelte:window on:mouseup={onWindowMouseUp} />

<EventModal
  open={showModal}
  event={editEvent}
  initialDate={modalStart}
  initialEndDate={modalEnd}
  on:close={closeModal}
/>

<div class="wrapper">

  <!-- ── Calendar area ── -->
  <div class="cal-area">

    <!-- Day-of-week header -->
    <div class="day-headers">
      {#each DAY_NAMES as name}
        <div class="day-header">{name}</div>
      {/each}
    </div>

    <!-- Grid -->
    <div class="grid" style:--rows={grid.length}>
      {#each grid.flat() as day (day.dateStr)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="day-cell"
          class:other-month={!day.isCurrentMonth}
          class:is-today={day.isToday}
          class:is-selected={$selectedDate === day.dateStr}
          class:in-drag={inDragRange(day.dateStr)}
          on:mousedown={() => onCellMouseDown(day.dateStr)}
          on:mouseenter={() => onCellMouseEnter(day.dateStr)}
          on:click={() => onCellClick(day.dateStr)}
          on:dblclick={() => onCellDblClick(day.dateStr)}
        >
          <!-- Date number -->
          <div class="day-num-row">
            <span class="day-num" class:today-badge={day.isToday}>{day.dayNum}</span>
          </div>

          <!-- Event bars -->
          <div class="event-list">
            {#each eventsForDay(day.dateStr).slice(0, 3) as evt (evt.id)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="event-bar"
                style:--evt-color={evt.color}
                on:click={(e) => openEditModal(evt, e)}
                title={evt.title}
              >
                {evt.title}
              </div>
            {/each}
            {#if eventsForDay(day.dateStr).length > 3}
              <div class="more-label">+{eventsForDay(day.dateStr).length - 3} more</div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

  </div>

  <!-- ── Day panel ── -->
  {#if $selectedDate}
    <div class="day-panel" transition:slide={{ axis: 'x', duration: 180 }}>
      <div class="panel-header">
        <span class="panel-date">{fmtDisplayDate($selectedDate)}</span>
        <div class="panel-actions">
          <button
            class="panel-add"
            on:click={() => openNewModal($selectedDate!, $selectedDate!)}
            title="New event"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
          </button>
          <button class="panel-close" on:click={() => ($selectedDate = null)} title="Close">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
      </div>

      <div class="panel-body">
        {#if eventsForDay($selectedDate).length === 0}
          <p class="no-events">No events.<br/>Double-click the day to add one.</p>
        {:else}
          {#each eventsForDay($selectedDate) as evt (evt.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="panel-event" on:click={(e) => openEditModal(evt, e)}>
              <div class="panel-dot" style:background={evt.color}></div>
              <div class="panel-info">
                <span class="panel-title">{evt.title}</span>
                <span class="panel-time">
                  {evt.all_day ? 'All day' : `${fmtTime(evt.start_time)} – ${fmtTime(evt.end_time)}`}
                </span>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}

</div>

<style>
  /* ── Outer wrapper ─────────────────────────────────────────────────── */
  .wrapper {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--bg-main);
  }

  /* ── Calendar column ───────────────────────────────────────────────── */
  .cal-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  /* ── Day-of-week header row ────────────────────────────────────────── */
  .day-headers {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    flex-shrink: 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
  }

  .day-header {
    padding: 8px 0;
    text-align: center;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* ── Main grid ─────────────────────────────────────────────────────── */
  .grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-template-rows: repeat(var(--rows, 6), 1fr);
    overflow: hidden;
  }

  /* ── Day cell ──────────────────────────────────────────────────────── */
  .day-cell {
    border-right: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    padding: 4px 5px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    cursor: pointer;
    overflow: hidden;
    user-select: none;
    transition: background 0.1s;
    min-height: 0;
  }

  .day-cell:nth-child(7n) { border-right: none; }

  .day-cell:hover { background: var(--hover-bg); }

  .day-cell.other-month { opacity: 0.35; }

  .day-cell.is-selected { background: color-mix(in srgb, var(--border-focus) 10%, transparent); }

  .day-cell.in-drag { background: color-mix(in srgb, var(--border-focus) 18%, transparent); }

  /* Date number */
  .day-num-row {
    display: flex;
    justify-content: flex-end;
    padding: 1px 2px;
  }

  .day-num {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-muted);
    line-height: 1;
  }

  .is-today .day-num { color: var(--text-main); }

  .today-badge {
    background: var(--border-focus);
    color: #fff !important;
    border-radius: 50%;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.78rem;
    font-weight: 700;
    line-height: 1;
  }

  /* Event bars */
  .event-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
    flex: 1;
    min-height: 0;
  }

  .event-bar {
    background: var(--evt-color, var(--border-focus));
    color: #fff;
    font-size: 0.68rem;
    font-weight: 500;
    padding: 1px 5px;
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    line-height: 1.6;
    transition: opacity 0.1s;
  }

  .event-bar:hover { opacity: 0.85; }

  .more-label {
    font-size: 0.68rem;
    color: var(--text-muted);
    padding: 0 4px;
    line-height: 1.4;
  }

  /* ── Day panel ─────────────────────────────────────────────────────── */
  .day-panel {
    width: 260px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--bg-panel);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: 8px;
  }

  .panel-date {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-main);
    flex: 1;
    min-width: 0;
  }

  .panel-actions { display: flex; gap: 4px; }

  .panel-add, .panel-close {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }
  .panel-add:hover  { background: var(--hover-bg); color: var(--border-focus); }
  .panel-close:hover { background: var(--hover-bg); color: var(--text-main); }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .panel-body::-webkit-scrollbar { width: 6px; }
  .panel-body::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  .no-events {
    color: var(--text-muted);
    font-size: 0.82rem;
    text-align: center;
    margin: auto;
    line-height: 1.6;
  }

  .panel-event {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.1s;
    border: 1px solid transparent;
  }
  .panel-event:hover {
    background: var(--hover-bg);
    border-color: var(--border);
  }

  .panel-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 3px;
  }

  .panel-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .panel-title {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-main);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .panel-time {
    font-size: 0.75rem;
    color: var(--text-muted);
  }
</style>
