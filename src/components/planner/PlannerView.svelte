<script lang="ts">
  import { calendarView, selectedDate, loadEvents } from '$lib/stores/plannerStore';
  import CalendarView from './CalendarView.svelte';
  import WeekView from './WeekView.svelte';
  import DayView from './DayView.svelte';
  import TaskPanel from './TaskPanel.svelte';
  import TimeBlockView from './TimeBlockView.svelte';

  let showTaskPanel  = $state(true);
  let showTimeBlocks = $state(false);

  const MONTHS = [
    'January','February','March','April','May','June',
    'July','August','September','October','November','December'
  ];
  const MONTHS_SHORT = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const DAY_NAMES_LONG = ['Sunday','Monday','Tuesday','Wednesday','Thursday','Friday','Saturday'];

  let viewDate = $state(new Date());

  // ─── Derived props for each sub-view ────────────────────────────────────────

  let viewYear  = $derived(viewDate.getFullYear());
  let viewMonth = $derived(viewDate.getMonth());

  /** Sunday that starts the displayed week */
  let weekStart = $derived(getWeekStart(viewDate));

  function getWeekStart(d: Date): Date {
    const s = new Date(d);
    s.setDate(s.getDate() - s.getDay());
    s.setHours(0, 0, 0, 0);
    return s;
  }

  // ─── Reload events whenever viewDate or view mode changes ───────────────────

  $effect(() => {
    const [start, end] = visibleRange($calendarView, viewDate);
    loadEvents(start, end);
  });

  function fmt(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  function visibleRange(mode: string, d: Date): [string, string] {
    if (mode === 'month') {
      const first = new Date(d.getFullYear(), d.getMonth(), 1);
      const last  = new Date(d.getFullYear(), d.getMonth() + 1, 0);
      const start = new Date(first);
      start.setDate(start.getDate() - start.getDay());
      const end = new Date(last);
      end.setDate(end.getDate() + (6 - end.getDay()));
      return [fmt(start), fmt(end)];
    }
    if (mode === 'week') {
      const start = getWeekStart(d);
      const end   = new Date(start);
      end.setDate(end.getDate() + 6);
      return [fmt(start), fmt(end)];
    }
    // day
    return [fmt(d), fmt(d)];
  }

  // ─── Navigation ─────────────────────────────────────────────────────────────

  function prevPeriod() {
    const d = new Date(viewDate);
    if ($calendarView === 'month') {
      d.setDate(1);
      d.setMonth(d.getMonth() - 1);
    } else if ($calendarView === 'week') {
      d.setDate(d.getDate() - 7);
    } else {
      d.setDate(d.getDate() - 1);
    }
    viewDate = d;
  }

  function nextPeriod() {
    const d = new Date(viewDate);
    if ($calendarView === 'month') {
      d.setDate(1);
      d.setMonth(d.getMonth() + 1);
    } else if ($calendarView === 'week') {
      d.setDate(d.getDate() + 7);
    } else {
      d.setDate(d.getDate() + 1);
    }
    viewDate = d;
  }

  function goToday() {
    const t = new Date();
    viewDate = t;
    $selectedDate = fmt(t);
  }

  // ─── Title ───────────────────────────────────────────────────────────────────

  let title = $derived(buildTitle($calendarView, viewDate));

  function buildTitle(mode: string, d: Date): string {
    if (mode === 'month') {
      return `${MONTHS[d.getMonth()]} ${d.getFullYear()}`;
    }
    if (mode === 'week') {
      const ws = getWeekStart(d);
      const we = new Date(ws);
      we.setDate(we.getDate() + 6);
      if (ws.getMonth() === we.getMonth()) {
        return `${MONTHS_SHORT[ws.getMonth()]} ${ws.getDate()} – ${we.getDate()}, ${we.getFullYear()}`;
      }
      return `${MONTHS_SHORT[ws.getMonth()]} ${ws.getDate()} – ${MONTHS_SHORT[we.getMonth()]} ${we.getDate()}, ${we.getFullYear()}`;
    }
    // day
    return `${DAY_NAMES_LONG[d.getDay()]}, ${MONTHS_SHORT[d.getMonth()]} ${d.getDate()}, ${d.getFullYear()}`;
  }
</script>

<div class="planner">

  <!-- ── Toolbar ── -->
  <div class="toolbar">

    <!-- View switcher -->
    <div class="view-switcher">
      <button class:active={$calendarView === 'month'} onclick={() => { $calendarView = 'month'; showTimeBlocks = false; }}>Month</button>
      <button class:active={$calendarView === 'week'}  onclick={() => { $calendarView = 'week';  showTimeBlocks = false; }}>Week</button>
      <button class:active={$calendarView === 'day' && !showTimeBlocks} onclick={() => { $calendarView = 'day'; showTimeBlocks = false; }}>Day</button>
      <button class:active={showTimeBlocks} onclick={() => { $calendarView = 'day'; showTimeBlocks = true; }}>Time Blocks</button>
    </div>

    <!-- Navigation -->
    <div class="nav-group">
      <button class="nav-btn" onclick={prevPeriod} title="Previous">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="15 18 9 12 15 6"/>
        </svg>
      </button>

      <h2 class="period-title">{title}</h2>

      <button class="nav-btn" onclick={nextPeriod} title="Next">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
      </button>

      <button class="today-btn" onclick={goToday}>Today</button>
    </div>

    <!-- Task panel toggle -->
    <button
      class="tasks-toggle"
      class:active={showTaskPanel}
      onclick={() => (showTaskPanel = !showTaskPanel)}
      title={showTaskPanel ? 'Hide tasks' : 'Show tasks'}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
        <path d="M9 11l3 3L22 4"/>
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
      </svg>
      Tasks
    </button>

  </div>

  <!-- ── Body: calendar + task panel ── -->
  <div class="body">

    <!-- Calendar/week/day view -->
    <div class="content">
      {#if $calendarView === 'month'}
        <CalendarView year={viewYear} month={viewMonth} />
      {:else if $calendarView === 'week'}
        <WeekView startDate={weekStart} />
      {:else if showTimeBlocks}
        <TimeBlockView date={viewDate} />
      {:else}
        <DayView date={viewDate} />
      {/if}
    </div>

    <!-- Task panel -->
    {#if showTaskPanel}
      <TaskPanel />
    {/if}

  </div>

</div>

<style>
  .planner {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-main);
  }

  /* ── Toolbar ───────────────────────────────────────────────────────────── */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    height: 48px;
    flex-shrink: 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    gap: 16px;
  }

  /* View switcher */
  .view-switcher {
    display: flex;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }

  .view-switcher button {
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 5px 14px;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    font-family: inherit;
  }
  .view-switcher button:not(:last-child) { border-right: 1px solid var(--border); }
  .view-switcher button:hover { background: var(--hover-bg); color: var(--text-main); }
  .view-switcher button.active { background: var(--border-focus); color: #fff; }

  /* Navigation group */
  .nav-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .period-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-main);
    min-width: 220px;
    text-align: center;
    white-space: nowrap;
  }

  .nav-btn {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 5px;
    padding: 5px 7px;
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }
  .nav-btn:hover { background: var(--hover-bg); color: var(--text-main); border-color: var(--border-focus); }

  .today-btn {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 5px;
    padding: 5px 12px;
    font-size: 0.82rem;
    font-weight: 500;
    font-family: inherit;
    transition: all 0.15s;
    margin-left: 4px;
  }
  .today-btn:hover { background: var(--hover-bg); color: var(--text-main); border-color: var(--border-focus); }

  /* ── Tasks toggle button ───────────────────────────────────────────────── */
  .tasks-toggle {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 5px;
    padding: 5px 10px;
    font-size: 0.82rem;
    font-weight: 500;
    font-family: inherit;
    display: flex;
    align-items: center;
    gap: 5px;
    transition: all 0.15s;
    margin-left: auto;
  }
  .tasks-toggle:hover { background: var(--hover-bg); color: var(--text-main); border-color: var(--border-focus); }
  .tasks-toggle.active { background: color-mix(in srgb, var(--border-focus) 15%, transparent); color: var(--border-focus); border-color: var(--border-focus); }

  /* ── Body ───────────────────────────────────────────────────────────────── */
  .body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: row;
  }

  /* ── Content ───────────────────────────────────────────────────────────── */
  .content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
</style>
