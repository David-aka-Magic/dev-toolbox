<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { events } from '$lib/stores/plannerStore';
  import EventModal from './EventModal.svelte';
  import type { Event as PlannerEvent } from '$lib/stores/plannerStore';

  let { date }: { date: Date } = $props();

  // ─── Constants ──────────────────────────────────────────────────────────────

  const HOUR_PX   = 64;
  const GUTTER_W  = 52;
  const MONTH_ABR = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const DAY_NAMES = ['Sunday','Monday','Tuesday','Wednesday','Thursday','Friday','Saturday'];
  const HOURS     = Array.from({ length: 24 }, (_, i) => i);

  // ─── Types ───────────────────────────────────────────────────────────────────

  type Event = PlannerEvent;

  interface LayoutEvent {
    event: Event;
    top: number;
    height: number;
    col: number;
    maxCols: number;
  }

  // ─── Utilities ───────────────────────────────────────────────────────────────

  function fmtDate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  function isToday(d: Date): boolean {
    return fmtDate(d) === fmtDate(new Date());
  }

  function isRecurring(id: string): boolean {
    return id.length > 36;
  }

  function fmtTime12(mins: number): string {
    const h = Math.floor(mins / 60);
    const ampm = h < 12 ? 'AM' : 'PM';
    const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${h12} ${ampm}`;
  }

  function fmtTimeFull(iso: string): string {
    if (!iso.includes('T')) return 'All day';
    const t = iso.split('T')[1].slice(0, 5);
    const [h, m] = t.split(':').map(Number);
    const ampm = h < 12 ? 'AM' : 'PM';
    const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${h12}:${String(m).padStart(2, '0')} ${ampm}`;
  }

  // ─── Event layout ────────────────────────────────────────────────────────────

  let layout    = $derived(buildLayout($events, date));
  let allDayEvt = $derived(buildAllDay($events, date));

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

  function buildLayout(evts: Event[], d: Date): LayoutEvent[] {
    const dateStr = fmtDate(d);
    const dayEvts = evts.filter(e => {
      if (e.all_day) return false;
      const eDate = e.start_time.includes('T') ? e.start_time.split('T')[0] : e.start_time;
      return eDate === dateStr;
    });
    if (dayEvts.length === 0) return [];

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
        event:    evt,
        top:      startM * (HOUR_PX / 60),
        height:   Math.max(24, (endM - startM) * (HOUR_PX / 60)),
        col:      cols[i],
        maxCols,
      };
    });
  }

  function buildAllDay(evts: Event[], d: Date): Event[] {
    const dateStr = fmtDate(d);
    return evts.filter(e => {
      if (!e.all_day) return false;
      const eStart = e.start_time.split('T')[0];
      const eEnd   = e.end_time.split('T')[0];
      return dateStr >= eStart && dateStr <= eEnd;
    });
  }

  // ─── Current time ────────────────────────────────────────────────────────────

  let currentMins = $state(getCurrentMins());
  let timeInterval: ReturnType<typeof setInterval>;

  function getCurrentMins(): number {
    const now = new Date();
    return now.getHours() * 60 + now.getMinutes();
  }

  let bodyEl = $state<HTMLElement | null>(null);

  $effect(() => {
    if (bodyEl) {
      bodyEl.scrollTop = Math.max(0, currentMins * (HOUR_PX / 60) - 150);
    }
  });

  onMount(() => {
    timeInterval = setInterval(() => { currentMins = getCurrentMins(); }, 60_000);
  });

  onDestroy(() => clearInterval(timeInterval));

  // ─── Drag-to-create ──────────────────────────────────────────────────────────

  let dragStartMins = $state<number | null>(null);
  let dragEndMins   = $state<number | null>(null);
  let isDragging    = $state(false);

  function minsFromY(e: MouseEvent, col: HTMLElement): number {
    const rect = col.getBoundingClientRect();
    const y    = e.clientY - rect.top + (bodyEl?.scrollTop ?? 0);
    const raw  = Math.floor((y / HOUR_PX) * 60);
    return Math.max(0, Math.min(23 * 60 + 59, Math.round(raw / 15) * 15));
  }

  function onColumnMouseDown(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.evt-block')) return;
    const col = (e.currentTarget as HTMLElement);
    isDragging    = true;
    dragStartMins = minsFromY(e, col);
    dragEndMins   = dragStartMins + 60;
    e.preventDefault();
  }

  function onWindowMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    const col = document.querySelector<HTMLElement>('.day-col');
    if (!col) return;
    const m = minsFromY(e, col);
    if (m > (dragStartMins ?? 0)) dragEndMins = m;
    else { dragEndMins = dragStartMins; dragStartMins = m; }
  }

  function onWindowMouseUp() {
    if (!isDragging || dragStartMins === null) { isDragging = false; return; }
    const dateStr = fmtDate(date);
    const pad = (n: number) => String(n).padStart(2, '0');
    const toISO = (mins: number) => `${dateStr}T${pad(Math.floor(mins / 60))}:${pad(mins % 60)}:00`;
    modalStart = toISO(dragStartMins);
    modalEnd   = toISO(dragEndMins ?? dragStartMins + 60);
    editEvent  = null;
    showModal  = true;
    isDragging = false; dragStartMins = null; dragEndMins = null;
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

<div class="day-view">

  <!-- ── Date header ── -->
  <div class="date-header">
    <span class="dow" class:today-text={isToday(date)}>{DAY_NAMES[date.getDay()]}</span>
    <span class="date-num" class:today-badge={isToday(date)}>{date.getDate()}</span>
    <span class="month-yr">{MONTH_ABR[date.getMonth()]} {date.getFullYear()}</span>
  </div>

  <!-- ── All-day events ── -->
  {#if allDayEvt.length > 0}
    <div class="allday-row">
      <div class="allday-label">all-day</div>
      <div class="allday-events">
        {#each allDayEvt as evt (evt.id)}
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
    </div>
  {/if}

  <!-- ── Time grid ── -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="body" bind:this={bodyEl}>
    <div class="body-inner" style:height="{24 * HOUR_PX}px">

      <!-- Gutter -->
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

      <!-- Day column -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="day-col" onmousedown={onColumnMouseDown}>

        <!-- Drag highlight -->
        {#if isDragging && dragStartMins !== null}
          <div
            class="drag-highlight"
            style:top="{dragStartMins * (HOUR_PX / 60)}px"
            style:height="{Math.max(24, ((dragEndMins ?? dragStartMins + 60) - dragStartMins) * (HOUR_PX / 60))}px"
          ></div>
        {/if}

        <!-- Current time -->
        {#if isToday(date)}
          <div class="now-line" style:top="{currentMins * (HOUR_PX / 60)}px">
            <div class="now-dot"></div>
          </div>
        {/if}

        <!-- Events -->
        {#each layout as le (le.event.id)}
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
                {fmtTimeFull(le.event.start_time)} – {fmtTimeFull(le.event.end_time)}
              </div>
            {/if}
            {#if le.height >= 60 && le.event.description}
              <div class="evt-desc">{le.event.description}</div>
            {/if}
          </div>
        {/each}

      </div>
    </div>
  </div>

</div>

<style>
  .day-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-main);
  }

  /* ── Date header ─────────────────────────────────────────────────────── */
  .date-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-panel);
    flex-shrink: 0;
  }

  .dow {
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .dow.today-text { color: var(--border-focus); }

  .date-num {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-main);
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    line-height: 1;
  }

  .today-badge {
    background: var(--border-focus);
    color: #fff !important;
  }

  .month-yr {
    font-size: 0.9rem;
    color: var(--text-muted);
  }

  /* ── All-day row ─────────────────────────────────────────────────────── */
  .allday-row {
    display: flex;
    align-items: flex-start;
    gap: 0;
    border-bottom: 1px solid var(--border);
    min-height: 30px;
    flex-shrink: 0;
    background: var(--bg-panel);
  }

  .allday-label {
    width: 52px;
    flex-shrink: 0;
    text-align: right;
    padding: 6px 8px 6px 0;
    font-size: 0.62rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .allday-events {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    padding: 4px 6px;
  }

  .allday-evt {
    font-size: 0.75rem;
    font-weight: 500;
    color: #fff;
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    opacity: 0.9;
    transition: opacity 0.1s;
  }
  .allday-evt:hover { opacity: 1; }

  /* ── Body ────────────────────────────────────────────────────────────── */
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

  /* ── Lines ───────────────────────────────────────────────────────────── */
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

  /* ── Day column ──────────────────────────────────────────────────────── */
  .day-col {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--border);
    cursor: crosshair;
    min-width: 0;
  }

  /* ── Event block ─────────────────────────────────────────────────────── */
  .evt-block {
    position: absolute;
    border-radius: 5px;
    padding: 4px 8px;
    overflow: hidden;
    cursor: pointer;
    opacity: 0.93;
    transition: opacity 0.1s, box-shadow 0.1s;
    box-sizing: border-box;
    z-index: 1;
  }
  .evt-block:hover {
    opacity: 1;
    box-shadow: 0 2px 10px rgba(0,0,0,0.35);
    z-index: 2;
  }

  .evt-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: #fff;
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .evt-time {
    font-size: 0.7rem;
    color: rgba(255,255,255,0.85);
    margin-top: 2px;
  }

  .evt-desc {
    font-size: 0.7rem;
    color: rgba(255,255,255,0.75);
    margin-top: 2px;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  /* ── Recurrence icon ─────────────────────────────────────────────────── */
  .recur-icon {
    flex-shrink: 0;
    opacity: 0.85;
  }

  /* ── Current time ────────────────────────────────────────────────────── */
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
