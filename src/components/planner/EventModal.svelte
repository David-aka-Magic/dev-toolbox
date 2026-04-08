<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from '../ui/Modal.svelte';
  import { createEvent, updateEvent, deleteEvent } from '$lib/stores/plannerStore';
  import type { Event, EventInput } from '$lib/stores/plannerStore';

  let {
    open = false,
    event = null,
    initialDate = '',
    initialEndDate = '',
  }: {
    open?: boolean;
    event?: Event | null;
    initialDate?: string;
    initialEndDate?: string;
  } = $props();

  const dispatch = createEventDispatcher<{ close: void }>();

  const PRESET_COLORS = [
    '#3b82f6', '#ef4444', '#10b981', '#f59e0b',
    '#8b5cf6', '#ec4899', '#06b6d4', '#f97316',
    '#84cc16', '#6b7280',
  ];

  const WEEKDAYS = [
    { label: 'Mo', value: 1 }, { label: 'Tu', value: 2 },
    { label: 'We', value: 3 }, { label: 'Th', value: 4 },
    { label: 'Fr', value: 5 }, { label: 'Sa', value: 6 },
    { label: 'Su', value: 7 },
  ];

  let title = $state('');
  let description = $state('');
  let startDate = $state('');
  let startTime = $state('09:00');
  let endDate = $state('');
  let endTime = $state('10:00');
  let allDay = $state(false);
  let color = $state('#3b82f6');
  let recurrenceType = $state<'none' | 'daily' | 'weekly' | 'monthly' | 'yearly'>('none');
  let recurrenceInterval = $state(1);
  let recurrenceDays = $state<number[]>([]);
  let recurrenceEndDate = $state('');
  let saving = $state(false);
  let error = $state('');

  // Populate form when modal opens / event changes
  $effect(() => {
    if (!open) return;
    if (event) {
      title = event.title;
      description = event.description ?? '';
      allDay = event.all_day;
      color = event.color;
      const { date: sd, time: st } = splitDateTime(event.start_time);
      const { date: ed, time: et } = splitDateTime(event.end_time);
      startDate = sd; startTime = st;
      endDate   = ed; endTime   = et;
      parseRecurrenceRule(event.recurrence_rule);
    } else {
      title = ''; description = ''; allDay = false; color = '#3b82f6';
      const sdt = splitDateTime(initialDate || today());
      const edt = splitDateTime(initialEndDate || initialDate || today());
      startDate = sdt.date; startTime = sdt.time;
      endDate   = edt.date;
      endTime   = (edt.date === sdt.date && edt.time === sdt.time) ? addHour(sdt.time) : edt.time;
      recurrenceType = 'none'; recurrenceInterval = 1;
      recurrenceDays = []; recurrenceEndDate = '';
    }
    error = '';
  });

  function today() {
    return new Date().toISOString().split('T')[0];
  }

  function addHour(time: string): string {
    const [h, m] = time.split(':').map(Number);
    const next = (h + 1) % 24;
    return `${String(next).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
  }

  function splitDateTime(iso: string): { date: string; time: string } {
    if (iso.includes('T')) {
      const [d, t] = iso.split('T');
      return { date: d, time: t.replace('Z', '').slice(0, 5) };
    }
    return { date: iso, time: '09:00' };
  }

  function parseRecurrenceRule(ruleStr: string | null) {
    if (!ruleStr) { recurrenceType = 'none'; recurrenceInterval = 1; recurrenceDays = []; recurrenceEndDate = ''; return; }
    try {
      const r = JSON.parse(ruleStr);
      recurrenceType = r.type ?? 'none';
      recurrenceInterval = r.interval ?? 1;
      recurrenceDays = r.days ?? [];
      recurrenceEndDate = r.end_date ?? '';
    } catch { recurrenceType = 'none'; }
  }

  function buildRecurrenceRule(): string | null {
    if (recurrenceType === 'none') return null;
    return JSON.stringify({
      type: recurrenceType,
      interval: recurrenceInterval,
      days: recurrenceType === 'weekly' ? recurrenceDays : null,
      end_date: recurrenceEndDate || null,
    });
  }

  function toggleDay(v: number) {
    recurrenceDays = recurrenceDays.includes(v)
      ? recurrenceDays.filter(d => d !== v)
      : [...recurrenceDays, v].sort((a, b) => a - b);
  }

  function intervalLabel(): string {
    const map: Record<string, string> = { daily: 'days', weekly: 'weeks', monthly: 'months', yearly: 'years' };
    return map[recurrenceType] ?? '';
  }

  async function handleSave() {
    if (!title.trim()) { error = 'Title is required.'; return; }
    if (startDate > endDate) { error = 'End date must be on or after start date.'; return; }
    saving = true; error = '';
    const input: EventInput = {
      title: title.trim(),
      description: description.trim() || null,
      start_time: allDay ? startDate : `${startDate}T${startTime}:00`,
      end_time:   allDay ? endDate   : `${endDate}T${endTime}:00`,
      all_day: allDay,
      color,
      recurrence_rule: buildRecurrenceRule(),
    };
    try {
      if (event) await updateEvent(event.id, input);
      else       await createEvent(input);
      dispatch('close');
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!event) return;
    saving = true;
    try {
      await deleteEvent(event.id);
      dispatch('close');
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function handleClose() { dispatch('close'); }
</script>

<Modal {open} title={event ? 'Edit Event' : 'New Event'} width="480px" on:close={handleClose}>
  <div class="form">

    <!-- Title -->
    <div class="field">
      <label>Title</label>
      <input class="text-in" type="text" bind:value={title} placeholder="Event title" autofocus />
    </div>

    <!-- Description -->
    <div class="field">
      <label>Description</label>
      <textarea class="text-in textarea" bind:value={description} placeholder="Optional description" rows="2"></textarea>
    </div>

    <!-- All day toggle -->
    <div class="field inline">
      <label>All day</label>
      <label class="toggle">
        <input type="checkbox" bind:checked={allDay} />
        <span class="track"><span class="thumb"></span></span>
      </label>
    </div>

    <!-- Dates / Times -->
    <div class="field">
      <label>Start</label>
      <div class="dt-row">
        <input class="text-in date-in" type="date" bind:value={startDate} />
        {#if !allDay}
          <input class="text-in time-in" type="time" bind:value={startTime} />
        {/if}
      </div>
    </div>

    <div class="field">
      <label>End</label>
      <div class="dt-row">
        <input class="text-in date-in" type="date" bind:value={endDate} />
        {#if !allDay}
          <input class="text-in time-in" type="time" bind:value={endTime} />
        {/if}
      </div>
    </div>

    <!-- Color -->
    <div class="field">
      <label>Color</label>
      <div class="swatches">
        {#each PRESET_COLORS as c}
          <button
            class="swatch"
            class:active={color === c}
            style:background={c}
            onclick={() => (color = c)}
            title={c}
          ></button>
        {/each}
      </div>
    </div>

    <!-- Recurrence -->
    <div class="field">
      <label>Repeat</label>
      <select class="text-in select-in" bind:value={recurrenceType}>
        <option value="none">Does not repeat</option>
        <option value="daily">Daily</option>
        <option value="weekly">Weekly</option>
        <option value="monthly">Monthly</option>
        <option value="yearly">Yearly</option>
      </select>
    </div>

    {#if recurrenceType !== 'none'}
      <div class="field">
        <label>Interval</label>
        <div class="interval-row">
          <span class="muted">Every</span>
          <input class="text-in interval-in" type="number" min="1" max="365" bind:value={recurrenceInterval} />
          <span class="muted">{intervalLabel()}</span>
        </div>
      </div>

      {#if recurrenceType === 'weekly'}
        <div class="field">
          <label>On days</label>
          <div class="weekday-row">
            {#each WEEKDAYS as { label, value }}
              <button
                class="day-btn"
                class:active={recurrenceDays.includes(value)}
                onclick={() => toggleDay(value)}
              >{label}</button>
            {/each}
          </div>
        </div>
      {/if}

      <div class="field">
        <label>Ends on</label>
        <input class="text-in date-in" type="date" bind:value={recurrenceEndDate} />
      </div>
    {/if}

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </div>

  <div slot="footer" class="footer-btns">
    {#if event}
      <button class="btn-delete" onclick={handleDelete} disabled={saving}>Delete</button>
    {/if}
    <div class="spacer"></div>
    <button class="btn-cancel" onclick={handleClose} disabled={saving}>Cancel</button>
    <button class="btn-save" onclick={handleSave} disabled={saving || !title.trim()}>
      {saving ? 'Saving…' : 'Save'}
    </button>
  </div>
</Modal>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    max-height: 60vh;
    padding-right: 4px;
  }
  .form::-webkit-scrollbar { width: 6px; }
  .form::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  .field { display: flex; flex-direction: column; gap: 6px; }
  .field.inline { flex-direction: row; align-items: center; justify-content: space-between; }
  .field label { font-size: 0.82rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }

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
  .textarea { resize: none; min-height: 56px; }
  .select-in { cursor: pointer; }
  .date-in { flex: 1; min-width: 0; }
  .time-in { width: 100px; flex-shrink: 0; }
  .interval-in { width: 64px; flex-shrink: 0; text-align: center; }

  .dt-row { display: flex; gap: 8px; }
  .interval-row { display: flex; align-items: center; gap: 8px; }
  .muted { color: var(--text-muted); font-size: 0.88rem; }

  /* Toggle switch */
  .toggle { display: flex; cursor: pointer; }
  .toggle input { display: none; }
  .track { width: 38px; height: 22px; background: var(--border); border-radius: 11px; position: relative; transition: background 0.2s; }
  .toggle input:checked + .track { background: var(--border-focus); }
  .thumb { position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; background: #fff; border-radius: 50%; transition: transform 0.2s; }
  .toggle input:checked + .track .thumb { transform: translateX(16px); }

  /* Color swatches */
  .swatches { display: flex; gap: 8px; flex-wrap: wrap; }
  .swatch {
    width: 26px; height: 26px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; transition: transform 0.1s, border-color 0.1s;
  }
  .swatch:hover { transform: scale(1.15); }
  .swatch.active { border-color: var(--text-main); box-shadow: 0 0 0 2px var(--bg-header); }

  /* Weekday buttons */
  .weekday-row { display: flex; gap: 6px; }
  .day-btn {
    width: 34px; height: 34px; border-radius: 50%; border: 1px solid var(--border);
    background: transparent; color: var(--text-muted); font-size: 0.78rem; font-weight: 600;
    cursor: pointer; transition: all 0.15s;
  }
  .day-btn:hover { background: var(--hover-bg); color: var(--text-main); }
  .day-btn.active { background: var(--border-focus); border-color: var(--border-focus); color: #fff; }

  .error { color: #ef4444; font-size: 0.85rem; margin: 0; }

  /* Footer */
  .footer-btns { display: flex; align-items: center; gap: 8px; width: 100%; }
  .spacer { flex: 1; }
  button { padding: 7px 16px; border-radius: 5px; border: none; cursor: pointer; font-size: 0.88rem; font-weight: 500; font-family: inherit; transition: background 0.15s; }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-cancel { background: transparent; color: var(--text-muted); }
  .btn-cancel:hover:not(:disabled) { background: var(--hover-bg); color: var(--text-main); }
  .btn-save { background: var(--border-focus); color: #fff; }
  .btn-save:hover:not(:disabled) { opacity: 0.85; }
  .btn-delete { background: transparent; color: #ef4444; border: 1px solid #ef4444; }
  .btn-delete:hover:not(:disabled) { background: #ef4444; color: #fff; }
</style>
