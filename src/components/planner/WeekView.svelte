<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { events } from '$lib/stores/plannerStore';
  import EventModal from './EventModal.svelte';
  import type { Event as PlannerEvent } from '$lib/stores/plannerStore';

  let { startDate }: { startDate: Date } = $props();

  // ─── Constants ──────────────────────────────────────────────────────────────

  const HOUR_PX    = 60;
  const GUTTER_W   = 52;
  const DAY_NAMES  = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const MONTH_ABBR = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const HOURS      = Array.from({ length: 24 }, (_, i) => i);

  // ─── Types ───────────────────────────────────────────────────────────────────

  type Event = PlannerEvent;

  interface LayoutEvent {
    event: Event;
    top: number;
    height: number;
    col: number;
    maxCols: number;
    dayIndex: number;
  }

  // ─── Derived week days ───────────────────────────────────────────────────────

  let days = $derived(buildDays(startDate));

  function buildDays(start: Date): Date[] {
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(start);
      d.setDate(d.getDate() + i);
      return d;
    });
  }

  function fmtDate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  function isToday(d: Date): boolean {
    return fmtDate(d) === fmtDate(new Date());
  }

  function isRecurring(id: string): boolean {
    return id.length > 36;
  }

  // ─── Event layout ────────────────────────────────────────────────────────────

  let layoutByDay = $derived(buildLayout($events, days));
  let allDayByDay = $derived(buildAllDay($events, days));

  function eventStartMins(evt: Event): number {
    if (!evt.start_time.includes('T')) return 0;
    const t = evt.start_time.split('T')[1].slice(0, 5);
    const [h, m] = t.split(':').map(Number);
    return h * 60 + m;
  }

  function eventEndMins(evt: Event): number {
    if (!evt.end_time.includes('T')) return 24 * 60;
    const t = evt.end_time.split('T')[1].slice(0, 5);
    const [h, m] = t.split(':').map(Number);
    return h * 60 + m;
  }

  function buildLayout(evts: Event[], weekDays: Date[]): LayoutEvent[][] {
    return weekDays.map((day, dayIndex) => {
      const dateStr = fmtDate(day);
      const dayEvts = evts.filter(e => {
        if (e.all_day) return false;
        const eDate = e.start_time.includes('T') ? e.start_time.split('T')[0] : e.start_time;
        return eDate === dateStr;
      });

      if (dayEvts.length === 0) return [];

      // Sweep-line column assignment
      const sorted = [...dayEvts].sort((a, b) => eventStartMins(a) - eventStartMins(b));
      const cols: number[] = [];
      const ends: number[] = [];

      for (const evt of sorted) {
        const start = eventStartMins(evt);
        const end   = Math.max(start + 30, eventEndMins(evt));
        let c = 0;
        while (ends[c] !== undefined && ends[c] > start) c++;
        cols.push(c);
        ends[c] = end;
      }

      const maxCols = Math.max(...cols) + 1;

      return sorted.map((evt, i) => {
        const startM = eventStartMins(evt);
        const endM   = Math.max(startM + 30, eventEndMins(evt));
        return {
          event: evt,
          top:      startM * (HOUR_PX / 60),
          height:   Math.max(20, (endM - startM) * (HOUR_PX / 60)),
          col:      cols[i],
          maxCols,
          dayIndex,
        };
      });
    });
  }

  function buildAllDay(evts: Event[], weekDays: Date[]): Event[][] {
    return weekDays.map(day => {
      const dateStr = fmtDate(day);
      return evts.filter(e => {
        if (!e.all_day) return false;
        const eStart = e.start_time.split('T')[0];
        const eEnd   = e.end_time.split('T')[0];
        return dateStr >= eStart && dateStr <= eEnd;
      });
    });
  }

  function fmtTime12(mins: number): string {
    const h = Math.floor(mins / 60);
    const ampm = h < 12 ? 'AM' : 'PM';
    const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${h12} ${ampm}`;
  }

  // ─── Current time ────────────────────────────────────────────────────────────

  let currentMins = $state(getCurrentMins());
  let timeInterval: ReturnType<typeof setInterval>;

  function getCurrentMins(): number {
    const now = new Date();
    return now.getHours() * 60 + now.getMinutes();
  }

  // ─── Scroll to current time on mount ────────────────────────────────────────

  let bodyEl = $state<HTMLElement | null>(null);

  $effect(() => {
    if (bodyEl) {
      const scrollTo = Math.max(0, currentMins * (HOUR_PX / 60) - 120);
      bodyEl.scrollTop = scrollTo;
    }
  });

  onMount(() => {
    timeInterval = setInterval(() => { currentMins = getCurrentMins(); }, 60_000);
  });

  onDestroy(() => {
    clearInterval(timeInterval);
  });

  // ─── Drag-to-create ──────────────────────────────────────────────────────────

  let dragDayIndex  = $state<number | null>(null);
  let dragStartMins = $state<number | null>(null);
  let dragEndMins   = $state<number | null>(null);
  let isDragging    = $state(false);

  function minsFromMouseY(e: MouseEvent, column: HTMLElement): number {
    const rect = column.getBoundingClientRect();
    const y = e.clientY - rect.top; // rect.top is viewport-relative (already reflects scroll)
    const raw = Math.floor((y / HOUR_PX) * 60);
    return Math.max(0, Math.min(23 * 60 + 59, Math.round(raw / 15) * 15));
  }

  function onColumnMouseDown(e: MouseEvent, dayIndex: number) {
    if ((e.target as HTMLElement).closest('.evt-block')) return;
    const col = (e.currentTarget as HTMLElement);
    isDragging    = true;
    dragDayIndex  = dayIndex;
    dragStartMins = minsFromMouseY(e, col);
    dragEndMins   = dragStartMins + 60;
    e.preventDefault();
  }

  function onWindowMouseMove(e: MouseEvent) {
    if (!isDragging || dragDayIndex === null) return;
    const colEls = document.querySelectorAll<HTMLElement>('.day-column');
    const col = colEls[dragDayIndex];
    if (!col) return;
    const m = minsFromMouseY(e, col);
    if (m > (dragStartMins ?? 0)) dragEndMins = m;
    else { dragEndMins = dragStartMins; dragStartMins = m; }
  }

  function onWindowMouseUp() {
    if (!isDragging || dragDayIndex === null || dragStartMins === null) {
      isDragging = false; return;
    }
    const day = days[dragDayIndex];
    const dateStr = fmtDate(day);
    const pad = (n: number) => String(n).padStart(2, '0');
    const toISO = (mins: number) => `${dateStr}T${pad(Math.floor(mins / 60))}:${pad(mins % 60)}:00`;
    modalStart = toISO(dragStartMins);
    modalEnd   = toISO(dragEndMins ?? dragStartMins + 60);
    editEvent  = null;
    showModal  = true;
    isDragging = false; dragDayIndex = null; dragStartMins = null; dragEndMins = null;
  }

  function dragHighlight(dayIndex: number): { top: number; height: number } | null {
    if (!isDragging || dragDayIndex !== dayIndex || dragStartMins === null) return null;
    const end = dragEndMins ?? dragStartMins + 60;
    return {
      top:    dragStartMins * (HOUR_PX / 60),
      height: Math.max(20, (end - dragStartMins) * (HOUR_PX / 60)),
    };
  }

  // ─── Modal ───────────────────────────────────────────────────────────────────

  let showModal  = $state(false);
  let editEvent  = $state<Event | null>(null);
  let modalStart = $state('');
  let modalEnd   = $state('');

  function openEdit(evt: Event, e: MouseEvent) {
    e.stopPropagation();
    editEvent  = evt;
    modalStart = evt.start_time;
    modalEnd   = evt.end_time;
    showModal  = true;
  }

  function closeModal() { showModal = false; editEvent = null; }
</script>

<svelte:window onmousemove={onWindowMouseMove} onmouseup={onWindowMouseUp} />

<EventModal
  open={showModal}
  event={editEvent}
  initialDate={modalStart}
  initialEndDate={modalEnd}
  on:close={closeModal}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="week-view" bind:this={bodyEl}>

  <!-- ── Sticky header band (header + optional all-day row) ── -->
  <div class="top-sticky">
    <div class="header">
      <div class="gutter-spacer"></div>
      {#each days as day, i}
        <div class="col-header" class:today={isToday(day)}>
          <span class="col-dow">{DAY_NAMES[day.getDay()]}</span>
          <span class="col-date" class:today-badge={isToday(day)}>{day.getDate()}</span>
          <span class="col-month">{MONTH_ABBR[day.getMonth()]}</span>
        </div>
      {/each}
    </div>

    {#if allDayByDay.some(d => d.length > 0)}
      <div class="allday-row">
        <div class="gutter-allday">all-day</div>
        {#each allDayByDay as dayEvts, i}
          <div class="allday-col">
            {#each dayEvts as evt (evt.id)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="allday-evt"
                style:background={evt.color}
                onclick={(e) => openEdit(evt, e)}
                title={evt.title}
              >
                {#if isRecurring(evt.id)}
                  <svg class="recur-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>
                  </svg>
                {/if}
                {evt.title}
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- ── Time grid (scrolls under the sticky header) ── -->
  <div class="time-grid" style:height="{24 * HOUR_PX}px">

    <!-- Time gutter -->
    <div class="gutter">
      {#each HOURS as h}
        <div class="gutter-label" style:top="{h * HOUR_PX}px">
          {#if h > 0}{fmtTime12(h * 60)}{/if}
        </div>
      {/each}
    </div>

    <!-- Hour lines -->
    <div class="lines-layer">
      {#each HOURS as h}
        <div class="hour-line" style:top="{h * HOUR_PX}px"></div>
      {/each}
    </div>

    <!-- Day columns -->
    <div class="columns">
      {#each days as day, i}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="day-column"
          onmousedown={(e) => onColumnMouseDown(e, i)}
        >
          <!-- Drag highlight -->
          {#if dragHighlight(i)}
            <div
              class="drag-highlight"
              style:top="{dragHighlight(i)!.top}px"
              style:height="{dragHighlight(i)!.height}px"
            ></div>
          {/if}

          <!-- Current time indicator -->
          {#if isToday(day)}
            <div class="now-line" style:top="{currentMins * (HOUR_PX / 60)}px">
              <div class="now-dot"></div>
            </div>
          {/if}

          <!-- Events -->
          {#each layoutByDay[i] as le (le.event.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="evt-block"
              style:top="{le.top}px"
              style:height="{le.height}px"
              style:left="calc({le.col} / {le.maxCols} * 100% + 2px)"
              style:width="calc(100% / {le.maxCols} - 4px)"
              style:background={le.event.color}
              onclick={(e) => openEdit(le.event, e)}
              title={le.event.title}
            >
              <div class="evt-title">
                {#if isRecurring(le.event.id)}
                  <svg class="recur-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>
                  </svg>
                {/if}
                {le.event.title}
              </div>
              {#if le.height >= 40}
                <div class="evt-time">
                  {fmtTime12(Math.floor(le.top / HOUR_PX * 60))}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    </div>

  </div>

</div>

<style>
  .week-view {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-main);
  }
  .week-view::-webkit-scrollbar { width: 6px; }
  .week-view::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  /* ── Sticky band (header + all-day) ────────────────────────────────── */
  .top-sticky {
    position: sticky;
    top: 0;
    z-index: 20;
  }

  /* ── Header ─────────────────────────────────────────────────────────── */
  .header {
    display: flex;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
  }

  .gutter-spacer {
    width: 52px;
    flex-shrink: 0;
  }

  .col-header {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 6px 0;
    gap: 2px;
    border-left: 1px solid var(--border);
  }

  .col-dow {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }

  .col-date {
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--text-main);
    line-height: 1;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
  }

  .today-badge {
    background: var(--border-focus);
    color: #fff !important;
  }

  .col-month {
    font-size: 0.68rem;
    color: var(--text-muted);
  }

  .col-header.today .col-dow,
  .col-header.today .col-month {
    color: var(--border-focus);
  }

  /* ── All-day row ─────────────────────────────────────────────────────── */
  .allday-row {
    display: flex;
    border-bottom: 1px solid var(--border);
    min-height: 28px;
    background: var(--bg-panel);
  }

  .gutter-allday {
    width: 52px;
    flex-shrink: 0;
    box-sizing: border-box; /* padding must not add to the 52px gutter width */
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding-right: 6px;
    font-size: 0.62rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .allday-col {
    flex: 1;
    border-left: 1px solid var(--border);
    padding: 2px 3px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .allday-evt {
    font-size: 0.7rem;
    font-weight: 500;
    color: #fff;
    padding: 1px 5px;
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 3px;
    opacity: 0.9;
    transition: opacity 0.1s;
  }
  .allday-evt:hover { opacity: 1; }

  /* ── Time grid ───────────────────────────────────────────────────────── */
  .time-grid {
    display: flex;
    position: relative;
    flex-shrink: 0;
  }

  /* ── Gutter ──────────────────────────────────────────────────────────── */
  .gutter {
    width: 52px;
    flex-shrink: 0;
    position: relative;
  }

  .gutter-label {
    position: absolute;
    right: 8px;
    transform: translateY(-50%);
    font-size: 0.65rem;
    color: var(--text-muted);
    white-space: nowrap;
    user-select: none;
  }

  /* ── Hour lines layer ────────────────────────────────────────────────── */
  .lines-layer {
    position: absolute;
    left: 52px;
    right: 0;
    top: 0;
    bottom: 0;
    pointer-events: none;
  }

  .hour-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--border);
    opacity: 0.5;
  }

  /* ── Day columns ─────────────────────────────────────────────────────── */
  .columns {
    flex: 1;
    display: flex;
    min-width: 0;
  }

  .day-column {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--border);
    cursor: crosshair;
    min-width: 0;
  }

  /* ── Event block ─────────────────────────────────────────────────────── */
  .evt-block {
    position: absolute;
    border-radius: 4px;
    padding: 2px 5px;
    overflow: hidden;
    cursor: pointer;
    opacity: 0.92;
    transition: opacity 0.1s, box-shadow 0.1s;
    box-sizing: border-box;
    z-index: 1;
  }
  .evt-block:hover {
    opacity: 1;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
    z-index: 2;
  }

  .evt-title {
    font-size: 0.72rem;
    font-weight: 600;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .evt-time {
    font-size: 0.65rem;
    color: rgba(255,255,255,0.8);
    margin-top: 1px;
  }

  /* ── Recurrence icon ─────────────────────────────────────────────────── */
  .recur-icon {
    flex-shrink: 0;
    opacity: 0.85;
  }

  /* ── Current time indicator ──────────────────────────────────────────── */
  .now-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: #ef4444;
    pointer-events: none;
    z-index: 3;
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

  /* ── Drag highlight ──────────────────────────────────────────────────── */
  .drag-highlight {
    position: absolute;
    left: 2px;
    right: 2px;
    background: color-mix(in srgb, var(--border-focus) 30%, transparent);
    border: 1px solid var(--border-focus);
    border-radius: 4px;
    pointer-events: none;
    z-index: 1;
  }
</style>
