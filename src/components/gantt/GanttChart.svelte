<script lang="ts">
  import {
    currentProject, tasks, milestones, dependencies,
    zoomLevel, selectedTaskId, viewportStartDate,
    loadProjects, loadProjectData, updateProject, updateTask, batchUpdateTasks,
    createProject, createTask, deleteTask, createMilestone,
    createDependency, deleteDependency, validateDependencies, reorderTasks,
    type GanttTask, type GanttMilestone, type GanttDependency, type GanttTaskInput,
  } from '$lib/stores/ganttStore';
  import { settings } from '$lib/stores/settingsStore';
  import GanttTaskBar from './GanttTaskBar.svelte';
  import ProjectModal from './ProjectModal.svelte';
  import TaskModal    from './TaskModal.svelte';
  import html2canvas from 'html2canvas';
  import { save, open as openDialog } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';

  // ─── Constants ───────────────────────────────────────────────────────────────

  const ROW_H       = $derived(($settings.ganttRowHeight as number) ?? 36);
  const HEADER_H    = 56;
  const MIN_PANEL_W = 160;
  const MAX_PANEL_W = 520;
  const COL_W: Record<string, number> = { days: 36, weeks: 100, months: 120 };
  const TASK_COLORS = [
    '#3b82f6','#8b5cf6','#ec4899','#ef4444',
    '#f97316','#eab308','#22c55e','#14b8a6','#64748b','#06b6d4',
  ];

  // ─── Row model ───────────────────────────────────────────────────────────────

  type Row =
    | { kind: 'group'; name: string }
    | { kind: 'task';  task: GanttTask; indent: number; isParent: boolean }
    | { kind: 'milestone'; milestone: GanttMilestone };

  let rows = $derived(buildRows($tasks, $milestones));

  function buildRows(allTasks: GanttTask[], allMilestones: GanttMilestone[]): Row[] {
    const result: Row[] = [];

    // Build children map
    const childMap = new Map<string, GanttTask[]>();
    for (const t of allTasks) {
      if (t.parent_task_id) {
        const arr = childMap.get(t.parent_task_id) ?? [];
        arr.push(t);
        childMap.set(t.parent_task_id, arr);
      }
    }
    for (const [, arr] of childMap) arr.sort((a, b) => a.sort_order - b.sort_order);

    // Root tasks sorted
    const roots = allTasks
      .filter(t => !t.parent_task_id)
      .sort((a, b) => a.sort_order - b.sort_order);

    const seenGroups = new Set<string>();

    for (const task of roots) {
      if (task.group_name && !seenGroups.has(task.group_name)) {
        seenGroups.add(task.group_name);
        result.push({ kind: 'group', name: task.group_name });
      }
      const children = childMap.get(task.id) ?? [];
      result.push({ kind: 'task', task, indent: 0, isParent: children.length > 0 });
      for (const child of children) {
        result.push({ kind: 'task', task: child, indent: 1, isParent: false });
      }
    }

    // Milestones at the end
    for (const m of [...allMilestones].sort((a, b) => a.date.localeCompare(b.date))) {
      result.push({ kind: 'milestone', milestone: m });
    }

    return result;
  }

  function rowKey(row: Row, i: number): string {
    if (row.kind === 'task')      return row.task.id;
    if (row.kind === 'milestone') return 'ms_' + row.milestone.id;
    return 'grp_' + row.name + '_' + i;
  }

  // ─── Critical path ────────────────────────────────────────────────────────────

  let showCriticalPath = $state(false);

  let criticalPathIds = $derived(computeCriticalPath($tasks, $dependencies));

  function computeCriticalPath(allTasks: GanttTask[], allDeps: GanttDependency[]): Set<string> {
    if (!allTasks.length) return new Set();

    const dur = new Map<string, number>();
    for (const t of allTasks) dur.set(t.id, Math.max(1, dateDuration(t.start_date, t.end_date)));

    const succs = new Map<string, string[]>();
    const preds = new Map<string, string[]>();
    for (const t of allTasks) { succs.set(t.id, []); preds.set(t.id, []); }
    for (const dep of allDeps) {
      if (dep.dependency_type === 'finish_to_start') {
        succs.get(dep.from_task_id)?.push(dep.to_task_id);
        preds.get(dep.to_task_id)?.push(dep.from_task_id);
      }
    }

    // Kahn's topological sort
    const inDeg = new Map<string, number>();
    for (const t of allTasks) inDeg.set(t.id, preds.get(t.id)!.length);
    const queue = allTasks.filter(t => inDeg.get(t.id) === 0).map(t => t.id);
    const topo: string[] = [];
    while (queue.length) {
      const id = queue.shift()!;
      topo.push(id);
      for (const s of succs.get(id) ?? []) {
        const nd = (inDeg.get(s) ?? 1) - 1;
        inDeg.set(s, nd);
        if (nd === 0) queue.push(s);
      }
    }
    if (topo.length < allTasks.length) return new Set(); // cycle

    // Forward pass
    const ES = new Map<string, number>(); const EF = new Map<string, number>();
    for (const id of topo) {
      const ps = preds.get(id) ?? [];
      const es = ps.length ? Math.max(...ps.map(p => EF.get(p) ?? 0)) : 0;
      ES.set(id, es); EF.set(id, es + dur.get(id)!);
    }
    const projEnd = Math.max(...[...EF.values()]);

    // Backward pass
    const LF = new Map<string, number>(); const LS = new Map<string, number>();
    for (const id of [...topo].reverse()) {
      const ss = succs.get(id) ?? [];
      const lf = ss.length ? Math.min(...ss.map(s => LS.get(s) ?? projEnd)) : projEnd;
      LF.set(id, lf); LS.set(id, lf - dur.get(id)!);
    }

    // Critical: total float = LS - ES ≈ 0
    const crit = new Set<string>();
    for (const t of allTasks) {
      if (Math.abs((LS.get(t.id) ?? 0) - (ES.get(t.id) ?? 0)) < 0.01) crit.add(t.id);
    }
    return crit;
  }

  // ─── Project progress ─────────────────────────────────────────────────────────

  let projectProgress = $derived(computeProjectProgress($tasks));

  function computeProjectProgress(allTasks: GanttTask[]): number {
    if (!allTasks.length) return 0;
    const totalDur = allTasks.reduce((s, t) => s + Math.max(1, dateDuration(t.start_date, t.end_date)), 0);
    const weighted = allTasks.reduce((s, t) => s + t.progress * Math.max(1, dateDuration(t.start_date, t.end_date)), 0);
    return Math.round(weighted / totalDur);
  }

  // ─── Layout state ────────────────────────────────────────────────────────────

  let panelWidth   = $state(280);
  let resizing     = $state(false);
  let resizeStartX = 0;
  let resizeStartW = 0;

  let editProjectOpen = $state(false);
  let editingName     = $state(false);
  let nameInput       = $state('');

  let timelineEl:  HTMLElement;
  let leftEl:      HTMLElement;
  let headerInner: HTMLElement;

  // Minimap
  let minimapWidth        = $state(0);
  let timelineScrollLeft  = $state(0);
  let minimapDragging     = $state(false);
  let _mmDragStartX       = 0;
  let _mmDragStartScroll  = 0;

  // Undo stack
  interface UndoEntry { tasks: GanttTask[] }
  let undoStack   = $state<UndoEntry[]>([]);
  const MAX_UNDO  = 20;

  function pushUndo(changedTasks: GanttTask[]) {
    undoStack = [...undoStack.slice(-(MAX_UNDO - 1)), { tasks: changedTasks.map(t => ({ ...t })) }];
  }

  async function undoLast() {
    if (!undoStack.length) { showToast('Nothing to undo', 'warning'); return; }
    const entry = undoStack[undoStack.length - 1];
    undoStack = undoStack.slice(0, -1);
    await batchUpdateTasks(entry.tasks.map(t => ({
      id: t.id, start_date: t.start_date, end_date: t.end_date, progress: t.progress,
    })));
    showToast('Undo', 'success');
  }

  // Auto-schedule confirm
  interface AutoScheduleConfirm { affectedIds: string[]; deltaDays: number }
  let autoScheduleConfirm: AutoScheduleConfirm | null = $state(null);

  // ─── Timeline geometry ───────────────────────────────────────────────────────

  let columns = $derived(buildColumns($zoomLevel, $viewportStartDate, $tasks, $settings.ganttShowWeekends));

  function buildColumns(
    zoom: string, vpStart: Date, allTasks: GanttTask[], showWeekends = true,
  ): { date: Date; label: string; major?: boolean; isWeekend?: boolean }[] {
    const allDates = allTasks.flatMap(t => [parseDate(t.start_date), parseDate(t.end_date)]);
    const rangeEnd = allDates.length
      ? new Date(Math.max(...allDates.map(d => d.getTime())))
      : new Date(vpStart.getTime() + 90 * 86400000);
    const span   = rangeEnd.getTime() - vpStart.getTime();
    const extEnd = new Date(rangeEnd.getTime() + Math.max(span * 0.2, 14 * 86400000));

    const cols: { date: Date; label: string; major?: boolean; isWeekend?: boolean }[] = [];

    if (zoom === 'days') {
      const d = new Date(vpStart.getFullYear(), vpStart.getMonth(), vpStart.getDate());
      while (d <= extEnd) {
        const isWeekend = d.getDay() === 0 || d.getDay() === 6;
        if (!isWeekend || showWeekends) {
          cols.push({ date: new Date(d), label: String(d.getDate()), major: d.getDate() === 1, isWeekend });
        }
        d.setDate(d.getDate() + 1);
      }
    } else if (zoom === 'weeks') {
      const d = new Date(vpStart.getFullYear(), vpStart.getMonth(), vpStart.getDate());
      const dow = d.getDay(); d.setDate(d.getDate() - (dow === 0 ? 6 : dow - 1));
      while (d <= extEnd) {
        cols.push({
          date: new Date(d),
          label: `${d.getDate()} ${d.toLocaleString('default', { month: 'short' })}`,
          major: d.getDate() <= 7,
        });
        d.setDate(d.getDate() + 7);
      }
    } else {
      const d = new Date(vpStart.getFullYear(), vpStart.getMonth(), 1);
      while (d <= extEnd) {
        cols.push({
          date: new Date(d),
          label: d.toLocaleString('default', { month: 'short' }),
          major: d.getMonth() === 0,
        });
        d.setMonth(d.getMonth() + 1);
      }
    }
    return cols;
  }

  let majorGroups = $derived(buildMajorGroups($zoomLevel, columns));

  function buildMajorGroups(zoom: string, cols: typeof columns) {
    const groups: { label: string; span: number }[] = [];
    for (const col of cols) {
      const label = zoom === 'months'
        ? String(col.date.getFullYear())
        : col.date.toLocaleString('default', { month: 'long', year: 'numeric' });
      if (groups.length && groups[groups.length - 1].label === label) {
        groups[groups.length - 1].span++;
      } else {
        groups.push({ label, span: 1 });
      }
    }
    return groups;
  }

  const colW = $derived(COL_W[$zoomLevel] ?? 100);
  let minimapScale = $derived(minimapWidth / Math.max(1, columns.length * colW));

  // px per day at current zoom
  const pxPerDay = $derived(
    $zoomLevel === 'days'  ? colW :
    $zoomLevel === 'weeks' ? colW / 7 :
    colW / 30
  );

  function parseDate(iso: string): Date {
    const [y, m, d] = iso.split('-').map(Number);
    return new Date(y, m - 1, d);
  }

  function dateToX(date: Date): number {
    const d = new Date(date.getFullYear(), date.getMonth(), date.getDate());
    if ($zoomLevel === 'days') {
      const vp = new Date($viewportStartDate.getFullYear(), $viewportStartDate.getMonth(), $viewportStartDate.getDate());
      return ((d.getTime() - vp.getTime()) / 86400000) * colW;
    }
    const firstCol = columns[0]?.date;
    if (!firstCol) return 0;
    if ($zoomLevel === 'weeks') {
      return ((d.getTime() - firstCol.getTime()) / 86400000 / 7) * colW;
    }
    // months
    const months =
      (d.getFullYear() - firstCol.getFullYear()) * 12 +
      (d.getMonth()    - firstCol.getMonth()) +
      d.getDate() / daysInMonth(d);
    return months * colW;
  }

  function daysInMonth(d: Date) {
    return new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
  }

  function dateDuration(start: string, end: string): number {
    return Math.max(1, Math.round(
      (parseDate(end).getTime() - parseDate(start).getTime()) / 86400000
    ));
  }

  function barWidth(task: GanttTask): number {
    const days = dateDuration(task.start_date, task.end_date);
    if ($zoomLevel === 'days')  return days * colW;
    if ($zoomLevel === 'weeks') return (days / 7) * colW;
    const s = parseDate(task.start_date), e = parseDate(task.end_date);
    const months = (e.getFullYear() - s.getFullYear()) * 12 + (e.getMonth() - s.getMonth()) + days / 30;
    return months * colW;
  }

  function addDaysStr(iso: string, n: number): string {
    const [y, m, d] = iso.split('-').map(Number);
    const dt = new Date(y, m - 1, d + n);
    return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, '0')}-${String(dt.getDate()).padStart(2, '0')}`;
  }

  function snapDeltaDays(deltaPx: number): number {
    const raw = deltaPx / pxPerDay;
    if (!$settings.ganttSnapToGrid) return Math.round(raw);
    const unit = $zoomLevel === 'months' ? 7 : 1;
    return Math.round(raw / unit) * unit;
  }

  function getSummaryBounds(parentId: string): { x: number; w: number } | null {
    const children = $tasks.filter(t => t.parent_task_id === parentId);
    if (!children.length) return null;
    const xs = children.map(t => dateToX(parseDate(t.start_date)));
    const xe = children.map(t => dateToX(parseDate(t.end_date)));
    return { x: Math.min(...xs), w: Math.max(4, Math.max(...xe) - Math.min(...xs)) };
  }

  let todayX = $derived(dateToX(new Date()));

  // ─── Drag state ───────────────────────────────────────────────────────────────

  interface DragState {
    taskId:        string;
    mode:          'move' | 'resize-left' | 'resize-right';
    origMouseX:    number;
    origStartDate: string;
    origEndDate:   string;
    origBarX:      number;
    origBarW:      number;
    ghostX:        number;
    ghostW:        number;
    snapStart:     string;
    snapEnd:       string;
  }

  let drag: DragState | null = $state(null);

  let ghostRowIdx = $derived(
    drag ? rows.findIndex(r => r.kind === 'task' && r.task?.id === drag!.taskId) : -1
  );

  function onTaskBarDown(e: MouseEvent, task: GanttTask, mode: 'move'|'resize-left'|'resize-right') {
    if (linkMode) {
      if (!linkFromId) { linkFromId = task.id; }
      else if (linkFromId !== task.id) { void completeLinkTo(task.id); }
      return;
    }
    const bx = dateToX(parseDate(task.start_date));
    const bw = barWidth(task);
    drag = {
      taskId: task.id, mode,
      origMouseX:    e.clientX,
      origStartDate: task.start_date,
      origEndDate:   task.end_date,
      origBarX:      bx,
      origBarW:      bw,
      ghostX:        bx,
      ghostW:        bw,
      snapStart:     task.start_date,
      snapEnd:       task.end_date,
    };
    selectedTaskId.set(task.id);
  }

  // ─── Panel resize ─────────────────────────────────────────────────────────────

  function onResizeMouseDown(e: MouseEvent) {
    resizing     = true;
    resizeStartX = e.clientX;
    resizeStartW = panelWidth;
    e.preventDefault();
  }

  // ─── Window events ────────────────────────────────────────────────────────────

  function onMouseMove(e: MouseEvent) {
    if (minimapDragging && timelineEl) {
      const dx = e.clientX - _mmDragStartX;
      timelineEl.scrollLeft = Math.max(0, _mmDragStartScroll + dx / minimapScale);
      timelineScrollLeft = timelineEl.scrollLeft;
      return;
    }
    if (resizing) {
      panelWidth = Math.min(MAX_PANEL_W, Math.max(MIN_PANEL_W, resizeStartW + e.clientX - resizeStartX));
      return;
    }
    if (!drag) return;

    const deltaPx    = e.clientX - drag.origMouseX;
    const snappedDays = snapDeltaDays(deltaPx);

    if (drag.mode === 'move') {
      drag.snapStart = addDaysStr(drag.origStartDate, snappedDays);
      drag.snapEnd   = addDaysStr(drag.origEndDate,   snappedDays);
      drag.ghostX    = dateToX(parseDate(drag.snapStart));
      drag.ghostW    = drag.origBarW;
    } else if (drag.mode === 'resize-left') {
      const ns = addDaysStr(drag.origStartDate, snappedDays);
      if (ns <= drag.origEndDate) {
        drag.snapStart = ns;
        drag.ghostX    = dateToX(parseDate(ns));
        drag.ghostW    = dateToX(parseDate(drag.origEndDate)) - drag.ghostX;
      }
    } else {
      const ne = addDaysStr(drag.origEndDate, snappedDays);
      if (ne >= drag.origStartDate) {
        drag.snapEnd = ne;
        drag.ghostX  = drag.origBarX;
        drag.ghostW  = dateToX(parseDate(ne)) - drag.origBarX;
      }
    }
  }

  async function onMouseUp() {
    if (minimapDragging) { minimapDragging = false; return; }
    if (resizing) { resizing = false; return; }
    if (!drag) return;

    const d = drag;
    drag = null;

    if (d.snapStart !== d.origStartDate || d.snapEnd !== d.origEndDate) {
      const task = $tasks.find(t => t.id === d.taskId);
      if (task) pushUndo([task]);
      await batchUpdateTasks([{ id: d.taskId, start_date: d.snapStart, end_date: d.snapEnd }]);

      // Auto-expand: scroll to keep moved task in view
      setTimeout(() => {
        if (timelineEl) {
          const newEndX = dateToX(parseDate(d.snapEnd));
          const newStartX = dateToX(parseDate(d.snapStart));
          if (newEndX > timelineEl.scrollLeft + timelineEl.clientWidth - 40) {
            timelineEl.scrollLeft = Math.max(0, newEndX - timelineEl.clientWidth + 80);
          } else if (newStartX < timelineEl.scrollLeft + 20) {
            timelineEl.scrollLeft = Math.max(0, newStartX - 40);
          }
        }
      }, 30);

      // Check auto-scheduling: if task was pushed forward, offer to shift dependents
      const deltaDays = Math.round((parseDate(d.snapEnd).getTime() - parseDate(d.origEndDate).getTime()) / 86400000);
      if (deltaDays > 0) {
        const chain = getFinishToStartChain(d.taskId);
        if (chain.length > 0) {
          autoScheduleConfirm = { affectedIds: chain, deltaDays };
        }
      }
    }
  }

  function getFinishToStartChain(taskId: string): string[] {
    const visited = new Set<string>();
    const queue = [taskId];
    while (queue.length) {
      const id = queue.shift()!;
      for (const dep of $dependencies) {
        if (dep.from_task_id === id && dep.dependency_type === 'finish_to_start' && !visited.has(dep.to_task_id)) {
          visited.add(dep.to_task_id);
          queue.push(dep.to_task_id);
        }
      }
    }
    visited.delete(taskId);
    return [...visited];
  }

  async function confirmAutoSchedule() {
    if (!autoScheduleConfirm) return;
    const { affectedIds, deltaDays } = autoScheduleConfirm;
    autoScheduleConfirm = null;
    const tasksToMove = $tasks.filter(t => affectedIds.includes(t.id));
    pushUndo(tasksToMove);
    await batchUpdateTasks(tasksToMove.map(t => ({
      id: t.id,
      start_date: addDaysStr(t.start_date, deltaDays),
      end_date:   addDaysStr(t.end_date,   deltaDays),
    })));
    showToast(`Moved ${affectedIds.length} dependent task${affectedIds.length > 1 ? 's' : ''}`, 'success');
  }

  function onWindowClick() {
    if (contextMenu) contextMenu = null;
    if (depCtxMenu)  depCtxMenu  = null;
    if (exportDropdownOpen) exportDropdownOpen = false;
  }

  // ─── Scroll sync ─────────────────────────────────────────────────────────────

  let _syncingScroll = false;

  function onLeftScroll(e: Event) {
    if (_syncingScroll) return;
    _syncingScroll = true;
    const st = (e.target as HTMLElement).scrollTop;
    if (timelineEl) timelineEl.scrollTop = st;
    _syncingScroll = false;
  }

  function onTimelineScroll(e: Event) {
    if (_syncingScroll) return;
    _syncingScroll = true;
    const el = e.target as HTMLElement;
    if (leftEl)      leftEl.scrollTop      = el.scrollTop;
    if (headerInner) headerInner.scrollLeft = el.scrollLeft;
    timelineScrollLeft = el.scrollLeft;
    _syncingScroll = false;
  }

  // ─── Context menu ─────────────────────────────────────────────────────────────

  interface CtxMenu { x: number; y: number; taskId: string; page: 'main'|'colors'|'deps' }
  let contextMenu: CtxMenu | null = $state(null);

  function onBarContextMenu(e: MouseEvent, task: GanttTask) {
    e.preventDefault();
    e.stopPropagation();
    const cx = Math.min(e.clientX, window.innerWidth  - 172);
    const cy = Math.min(e.clientY, window.innerHeight - 240);
    contextMenu = { x: cx, y: cy, taskId: task.id, page: 'main' };
    selectedTaskId.set(task.id);
  }

  function ctxTask(): GanttTask | undefined {
    return $tasks.find(t => t.id === contextMenu?.taskId);
  }

  function ctxEdit() {
    const t = ctxTask(); if (!t) return;
    contextMenu = null;
    taskModalTask = t;
    taskModalOpen = true;
  }

  function ctxAddSubtask() {
    const t = ctxTask(); if (!t) return;
    contextMenu = null;
    taskModalTask       = null;
    newTaskParentId     = t.id;
    taskModalOpen       = true;
  }

  async function ctxSetColor(c: string) {
    const t = ctxTask(); if (!t) return;
    contextMenu = null;
    await updateTask(t.id, taskToInput(t, { color: c }));
  }

  async function ctxAddDep(fromId: string) {
    const t = ctxTask(); if (!t || !$currentProject) return;
    contextMenu = null;
    try {
      await createDependency({ project_id: $currentProject.id, from_task_id: fromId, to_task_id: t.id });
    } catch (err: any) {
      alert('Cannot create dependency: ' + err);
    }
  }

  async function ctxDelete() {
    const taskId = contextMenu?.taskId; if (!taskId) return;
    contextMenu = null;
    if (confirm('Delete this task?')) await deleteTask(taskId);
  }

  function taskToInput(t: GanttTask, overrides: Partial<GanttTaskInput> = {}): GanttTaskInput {
    return {
      project_id:     t.project_id,
      title:          t.title,
      start_date:     t.start_date,
      end_date:       t.end_date,
      progress:       t.progress,
      color:          t.color,
      group_name:     t.group_name,
      sort_order:     t.sort_order,
      collapsed:      t.collapsed,
      parent_task_id: t.parent_task_id,
      assigned_to:    t.assigned_to,
      notes:          t.notes,
      ...overrides,
    };
  }

  // ─── Task modal ───────────────────────────────────────────────────────────────

  let taskModalOpen   = $state(false);
  let taskModalTask:  GanttTask | null = $state(null);
  let newTaskParentId = $state('');

  function openTaskModal(task: GanttTask) {
    taskModalTask   = task;
    newTaskParentId = '';
    taskModalOpen   = true;
  }

  // ─── Milestone tooltip ────────────────────────────────────────────────────────

  interface Tooltip { x: number; y: number; text: string }
  let tooltip: Tooltip | null = $state(null);

  // ─── Row DnD reorder ─────────────────────────────────────────────────────────

  let dndRowId  = $state<string | null>(null);
  let dndOverId = $state<string | null>(null);

  async function handleDrop(toId: string) {
    if (!dndRowId || !dndOverId || dndRowId === dndOverId) {
      dndRowId = null; dndOverId = null; return;
    }
    const rootRows = rows
      .filter(r => r.kind === 'task' && !r.task!.parent_task_id)
      .map(r => (r as { kind: 'task'; task: GanttTask }).task);

    const fromIdx = rootRows.findIndex(t => t.id === dndRowId);
    const toIdx   = rootRows.findIndex(t => t.id === toId);
    if (fromIdx === -1 || toIdx === -1) { dndRowId = null; dndOverId = null; return; }

    const newOrder = [...rootRows.map(t => t.id)];
    const [moved]  = newOrder.splice(fromIdx, 1);
    newOrder.splice(toIdx, 0, moved);

    // Expand to include children after each parent
    const fullOrder: string[] = [];
    for (const id of newOrder) {
      fullOrder.push(id);
      $tasks
        .filter(t => t.parent_task_id === id)
        .sort((a, b) => a.sort_order - b.sort_order)
        .forEach(t => fullOrder.push(t.id));
    }

    dndRowId = null; dndOverId = null;
    await reorderTasks(fullOrder);
  }

  // ─── Dependency arrows ────────────────────────────────────────────────────

  let linkMode         = $state(false);
  let linkFromId       = $state<string | null>(null);
  let linkPreviewMouse = $state<{ x: number; y: number } | null>(null);
  let hoveredDepId     = $state<string | null>(null);

  interface DepCtxMenu { x: number; y: number; dep: GanttDependency; page: 'main'|'types' }
  let depCtxMenu: DepCtxMenu | null = $state(null);

  interface Toast { id: number; msg: string; type: 'success'|'warning'|'error' }
  let toasts = $state<Toast[]>([]);
  let _toastN = 0;

  function showToast(msg: string, type: Toast['type'] = 'success') {
    const id = _toastN++;
    toasts = [...toasts, { id, msg, type }];
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id); }, 3500);
  }

  function getTaskBarEdges(taskId: string): { leftX: number; rightX: number; centerY: number } | null {
    const ri = rows.findIndex(r => r.kind === 'task' && r.task?.id === taskId);
    if (ri === -1) return null;
    const row = rows[ri] as Extract<Row, { kind: 'task' }>;
    const task = row.task;
    if (row.isParent) {
      const sb = getSummaryBounds(task.id);
      if (!sb) return null;
      return { leftX: sb.x, rightX: sb.x + sb.w, centerY: ri * ROW_H + ROW_H / 2 };
    }
    const lx = dateToX(parseDate(task.start_date));
    const rx = lx + Math.max(barWidth(task), 6);
    return { leftX: lx, rightX: rx, centerY: ri * ROW_H + ROW_H / 2 };
  }

  function getDepEndpoints(dep: GanttDependency): [number, number, number, number] | null {
    const from = getTaskBarEdges(dep.from_task_id);
    const to   = getTaskBarEdges(dep.to_task_id);
    if (!from || !to) return null;
    switch (dep.dependency_type) {
      case 'finish_to_start':  return [from.rightX, from.centerY, to.leftX,  to.centerY];
      case 'start_to_start':   return [from.leftX,  from.centerY, to.leftX,  to.centerY];
      case 'finish_to_finish': return [from.rightX, from.centerY, to.rightX, to.centerY];
      case 'start_to_finish':  return [from.leftX,  from.centerY, to.rightX, to.centerY];
      default:                 return [from.rightX, from.centerY, to.leftX,  to.centerY];
    }
  }

  function fp(n: number): string { return (Math.round(n * 10) / 10).toString(); }

  function buildArrowPath(sx: number, sy: number, tx: number, ty: number, type: string): string {
    const M = 14;
    if (type === 'start_to_start' || type === 'start_to_finish') {
      const lx = Math.min(sx, tx) - M;
      return `M${fp(sx)},${fp(sy)} H${fp(lx)} V${fp(ty)} H${fp(tx)}`;
    }
    if (type === 'finish_to_finish') {
      const rx = Math.max(sx, tx) + M;
      return `M${fp(sx)},${fp(sy)} H${fp(rx)} V${fp(ty)} H${fp(tx)}`;
    }
    if (tx >= sx + M * 2) {
      return `M${fp(sx)},${fp(sy)} H${fp(sx + M)} V${fp(ty)} H${fp(tx)}`;
    }
    const midY = sy !== ty ? (sy + ty) / 2 : sy + ROW_H * 0.6;
    const lx   = Math.min(sx, tx) - M;
    return `M${fp(sx)},${fp(sy)} H${fp(sx + M)} V${fp(midY)} H${fp(lx)} V${fp(ty)} H${fp(tx)}`;
  }

  let arrowData = $derived(
    $dependencies.map(dep => {
      const pts = getDepEndpoints(dep);
      if (!pts) return null;
      const [sx, sy, tx, ty] = pts;
      const isHl = $selectedTaskId === dep.from_task_id ||
                   $selectedTaskId === dep.to_task_id   ||
                   hoveredDepId    === dep.id;
      return { dep, path: buildArrowPath(sx, sy, tx, ty, dep.dependency_type), isHl };
    }).filter((x): x is NonNullable<typeof x> => x !== null)
  );

  let linkPreviewLine = $derived(
    linkMode && linkFromId && linkPreviewMouse
      ? (() => {
          const from = getTaskBarEdges(linkFromId!);
          if (!from) return null;
          return { sx: from.rightX, sy: from.centerY, tx: linkPreviewMouse!.x, ty: linkPreviewMouse!.y };
        })()
      : null
  );

  async function completeLinkTo(toId: string) {
    const fromId = linkFromId!;
    linkFromId = null;
    if (fromId === toId) { showToast('A task cannot depend on itself', 'warning'); return; }
    if (!$currentProject) return;
    try {
      const dep = await createDependency({ project_id: $currentProject.id, from_task_id: fromId, to_task_id: toId });
      showToast('Dependency created', 'success');
      const warnings = await validateDependencies($currentProject.id);
      if (warnings.length > 0) {
        await deleteDependency(dep.id);
        showToast('Circular dependency detected — link removed', 'warning');
      }
    } catch (err: any) {
      showToast(String(err), 'error');
    }
  }

  function onTimelineBodyMouseMove(e: MouseEvent) {
    if (!linkMode || !linkFromId || !timelineEl) return;
    const rect = timelineEl.getBoundingClientRect();
    linkPreviewMouse = { x: e.clientX - rect.left + timelineEl.scrollLeft, y: e.clientY - rect.top + timelineEl.scrollTop };
  }

  function onDepContextMenu(e: MouseEvent, dep: GanttDependency) {
    e.preventDefault(); e.stopPropagation();
    depCtxMenu = { x: Math.min(e.clientX, window.innerWidth - 180), y: Math.min(e.clientY, window.innerHeight - 160), dep, page: 'main' };
  }

  async function depCtxDelete() {
    if (!depCtxMenu) return;
    const id = depCtxMenu.dep.id; depCtxMenu = null;
    await deleteDependency(id); showToast('Dependency deleted', 'success');
  }

  async function depCtxChangeType(type: string) {
    if (!depCtxMenu || !$currentProject) return;
    const dep = depCtxMenu.dep; depCtxMenu = null;
    await deleteDependency(dep.id);
    await createDependency({ project_id: dep.project_id, from_task_id: dep.from_task_id, to_task_id: dep.to_task_id, dependency_type: type });
    showToast('Dependency type updated', 'success');
  }

  function depTypeLabel(type: string): string {
    const map: Record<string, string> = {
      finish_to_start:  'Finish → Start',
      start_to_start:   'Start → Start',
      finish_to_finish: 'Finish → Finish',
      start_to_finish:  'Start → Finish',
    };
    return map[type] ?? type;
  }

  // ─── Inline project name ─────────────────────────────────────────────────────

  function startNameEdit() {
    if (!$currentProject) return;
    nameInput = $currentProject.name;
    editingName = true;
  }

  async function commitNameEdit() {
    editingName = false;
    if (!$currentProject || !nameInput.trim() || nameInput === $currentProject.name) return;
    await updateProject($currentProject.id, {
      name:         nameInput.trim(),
      description:  $currentProject.description,
      start_date:   $currentProject.start_date,
      color_scheme: $currentProject.color_scheme,
      zoom_level:   $currentProject.zoom_level,
    });
  }

  // ─── Toolbar actions ─────────────────────────────────────────────────────────

  function scrollToToday() {
    if (!timelineEl) return;
    timelineEl.scrollLeft = Math.max(0, todayX - timelineEl.clientWidth / 2);
  }

  function fitAll() {
    if (!$tasks.length) return;
    const dates = $tasks.flatMap(t => [parseDate(t.start_date), parseDate(t.end_date)]);
    viewportStartDate.set(new Date(Math.min(...dates.map(d => d.getTime()))));
  }

  async function addTask() {
    if (!$currentProject) return;
    taskModalTask   = null;
    newTaskParentId = '';
    taskModalOpen   = true;
  }

  async function addMilestone() {
    if (!$currentProject) return;
    await createMilestone({
      project_id: $currentProject.id,
      title:      'Milestone',
      date:       new Date().toISOString().split('T')[0],
    });
  }

  function goBack() {
    currentProject.set(null);
    loadProjects();
  }

  function scrollToEnd() {
    if (!timelineEl || !$tasks.length) return;
    const dates = $tasks.flatMap(t => [parseDate(t.start_date), parseDate(t.end_date)]);
    const endX = Math.max(...dates.map(d => dateToX(d)));
    timelineEl.scrollLeft = Math.max(0, endX - timelineEl.clientWidth + 60);
  }

  function cycleZoomIn() {
    const order = ['months', 'weeks', 'days'] as const;
    const idx = order.indexOf($zoomLevel);
    if (idx < order.length - 1) zoomLevel.set(order[idx + 1]);
  }

  function cycleZoomOut() {
    const order = ['months', 'weeks', 'days'] as const;
    const idx = order.indexOf($zoomLevel);
    if (idx > 0) zoomLevel.set(order[idx - 1]);
  }

  function navigateTasks(delta: number) {
    const taskRows = rows.filter(r => r.kind === 'task') as Extract<Row, { kind: 'task' }>[];
    if (!taskRows.length) return;
    const currIdx = taskRows.findIndex(r => r.task.id === $selectedTaskId);
    const newIdx  = Math.max(0, Math.min(taskRows.length - 1, (currIdx === -1 ? 0 : currIdx) + delta));
    const newId   = taskRows[newIdx].task.id;
    selectedTaskId.set(newId);
    const el = leftEl?.querySelector(`[data-task-id="${newId}"]`) as HTMLElement | null;
    el?.scrollIntoView({ block: 'nearest' });
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement) return;
    switch (e.key) {
      case 'Delete': case 'Backspace':
        if ($selectedTaskId) {
          e.preventDefault();
          const t = $tasks.find(t => t.id === $selectedTaskId);
          if (t && confirm(`Delete "${t.title}"?`)) { void deleteTask($selectedTaskId); }
        }
        break;
      case 'Enter':
        if ($selectedTaskId) {
          e.preventDefault();
          const t = $tasks.find(t => t.id === $selectedTaskId);
          if (t) openTaskModal(t);
        }
        break;
      case 'n': case 'N':
        if (!e.ctrlKey && !e.metaKey) { e.preventDefault(); void addTask(); }
        break;
      case 'm': case 'M':
        if (!e.ctrlKey && !e.metaKey) { e.preventDefault(); void addMilestone(); }
        break;
      case '=': case '+':
        e.preventDefault(); cycleZoomIn(); break;
      case '-':
        e.preventDefault(); cycleZoomOut(); break;
      case 'ArrowUp':
        e.preventDefault(); navigateTasks(-1); break;
      case 'ArrowDown':
        e.preventDefault(); navigateTasks(1); break;
      case 'z': case 'Z':
        if (e.ctrlKey || e.metaKey) { e.preventDefault(); void undoLast(); }
        break;
      case 'Escape':
        e.preventDefault();
        if (taskModalOpen) { taskModalOpen = false; taskModalTask = null; }
        else if (linkMode) { linkMode = false; linkFromId = null; }
        else if (contextMenu) { contextMenu = null; }
        else if (depCtxMenu)  { depCtxMenu  = null; }
        else if (autoScheduleConfirm) { autoScheduleConfirm = null; }
        else { selectedTaskId.set(null); }
        break;
      case 'Home':
        e.preventDefault(); fitAll(); if (timelineEl) timelineEl.scrollLeft = 0; break;
      case 'End':
        e.preventDefault(); scrollToEnd(); break;
    }
  }

  function onTimelineWheel(e: WheelEvent) {
    if (!e.ctrlKey) return;
    e.preventDefault();
    if (e.deltaY < 0) cycleZoomIn(); else cycleZoomOut();
  }

  // ─── Helpers ─────────────────────────────────────────────────────────────────

  function durationLabel(start: string, end: string): string {
    const d = dateDuration(start, end);
    return d === 1 ? '1d' : `${d}d`;
  }

  function formatDate(iso: string): string {
    const d = parseDate(iso);
    return `${d.getMonth() + 1}/${d.getDate()}`;
  }

  // ─── Loading / skeleton ───────────────────────────────────────────────────────

  let isLoading = $state(false);

  const SKELETON_ROWS = [
    { w: 180, ml: 20 }, { w: 120, ml: 80 }, { w: 200, ml: 10 },
    { w: 90,  ml: 60 }, { w: 150, ml: 30 }, { w: 110, ml: 100 },
  ];

  // ─── Export / Import ─────────────────────────────────────────────────────────

  let exportDropdownOpen = $state(false);
  let isExporting        = $state(false);
  let chartBodyEl: HTMLElement;

  function escXml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  async function exportPNG() {
    if (!chartBodyEl) return;
    isExporting = true; exportDropdownOpen = false;
    try {
      const canvas = await html2canvas(chartBodyEl, { scale: 2, useCORS: true, logging: false });
      const base64 = canvas.toDataURL('image/png').split(',')[1];
      const filePath = await save({
        filters: [{ name: 'PNG Image', extensions: ['png'] }],
        defaultPath: `${$currentProject?.name ?? 'gantt-chart'}.png`,
      });
      if (filePath) {
        await invoke('save_screenshot', { path: filePath, data: base64 });
        showToast('Exported as PNG', 'success');
      }
    } catch (err: any) { showToast('PNG export failed: ' + String(err), 'error'); }
    isExporting = false;
  }

  function buildExportSVG(): string {
    const HDR = 48;
    const totalW = panelWidth + columns.length * colW;
    const totalH = HDR + ROW_H + rows.length * ROW_H; // header row + data rows
    const tX = panelWidth; // timeline x offset
    const bY = HDR + ROW_H; // body y (below SVG header + col header row)

    const parts: string[] = [];
    parts.push(`<svg xmlns="http://www.w3.org/2000/svg" width="${totalW}" height="${totalH}" font-family="system-ui,sans-serif">`);
    parts.push(`<rect width="${totalW}" height="${totalH}" fill="#1a1a2e"/>`);

    // Project header bar
    parts.push(`<rect width="${totalW}" height="${HDR}" fill="#252540"/>`);
    const projName = $currentProject?.name ?? 'Gantt Chart';
    const dateRange = $tasks.length
      ? `${$tasks.map(t => t.start_date).sort()[0]} – ${$tasks.map(t => t.end_date).sort().at(-1)}`
      : new Date().toISOString().slice(0,10);
    parts.push(`<text x="16" y="${HDR/2+5}" font-size="15" font-weight="700" fill="#fff">${escXml(projName)}</text>`);
    parts.push(`<text x="${totalW-12}" y="${HDR/2+5}" font-size="10" fill="#888" text-anchor="end">${escXml(dateRange)}</text>`);

    // Column header row
    parts.push(`<rect x="0" y="${HDR}" width="${totalW}" height="${ROW_H}" fill="#20203a"/>`);
    parts.push(`<text x="12" y="${HDR+ROW_H/2+4}" font-size="9" font-weight="700" fill="#666" letter-spacing="0.5">TASK</text>`);
    let cx = tX;
    for (const col of columns) {
      if (col.major) parts.push(`<rect x="${cx}" y="${HDR}" width="${colW}" height="${ROW_H}" fill="#252540"/>`);
      parts.push(`<text x="${cx+colW/2}" y="${HDR+ROW_H/2+4}" font-size="9" fill="${col.isWeekend ? '#555' : '#666'}" text-anchor="middle">${col.label}</text>`);
      parts.push(`<line x1="${cx}" y1="${HDR}" x2="${cx}" y2="${totalH}" stroke="#2a2a44" stroke-width="1"/>`);
      cx += colW;
    }

    // Rows
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i];
      const ry = bY + i * ROW_H;
      const alt = i % 2 === 1;

      if (row.kind === 'group') {
        parts.push(`<rect x="0" y="${ry}" width="${totalW}" height="${ROW_H}" fill="#23233a"/>`);
        parts.push(`<text x="12" y="${ry+ROW_H/2+4}" font-size="9" font-weight="700" fill="#777" letter-spacing="0.5">${escXml(row.name.toUpperCase())}</text>`);
        continue;
      }

      // Left panel row bg
      parts.push(`<rect x="0" y="${ry}" width="${panelWidth}" height="${ROW_H}" fill="${alt ? '#1e1e35' : '#1a1a2e'}"/>`);
      // Timeline row bg
      if (alt) parts.push(`<rect x="${tX}" y="${ry}" width="${columns.length * colW}" height="${ROW_H}" fill="rgba(255,255,255,0.02)"/>`);

      if (row.kind === 'task') {
        const color = row.task.color;
        parts.push(`<circle cx="14" cy="${ry+ROW_H/2}" r="4" fill="${escXml(color)}"/>`);
        const title = row.task.title.length > 24 ? row.task.title.slice(0,22) + '…' : row.task.title;
        parts.push(`<text x="24" y="${ry+ROW_H/2+4}" font-size="11" fill="#ddd">${escXml(title)}</text>`);

        if (row.isParent) {
          const sb = getSummaryBounds(row.task.id);
          if (sb) parts.push(`<rect x="${tX+sb.x}" y="${ry+ROW_H/2-4}" width="${sb.w}" height="8" rx="2" fill="${escXml(color)}"/>`);
        } else if (row.task.start_date === row.task.end_date) {
          const dx = tX + dateToX(parseDate(row.task.start_date));
          parts.push(`<rect x="${dx-7}" y="${ry+ROW_H/2-7}" width="14" height="14" rx="2" fill="${escXml(color)}" transform="rotate(45,${dx},${ry+ROW_H/2})"/>`);
        } else {
          const bx = tX + dateToX(parseDate(row.task.start_date));
          const bw = Math.max(barWidth(row.task), 6);
          const bh = row.indent > 0 ? 20 : 26;
          const by2 = ry + (ROW_H - bh) / 2;
          parts.push(`<rect x="${bx}" y="${by2}" width="${bw}" height="${bh}" rx="4" fill="${escXml(color)}"/>`);
          if (row.task.progress > 0) parts.push(`<rect x="${bx}" y="${by2}" width="${bw * row.task.progress / 100}" height="${bh}" rx="4" fill="rgba(0,0,0,0.25)"/>`);
          if (bw > 36) {
            const lbl = row.task.title.length > 14 ? row.task.title.slice(0,12) + '…' : row.task.title;
            parts.push(`<text x="${bx+6}" y="${by2+bh/2+4}" font-size="9" font-weight="600" fill="white">${escXml(lbl)}</text>`);
          }
        }
      } else if (row.kind === 'milestone') {
        parts.push(`<text x="24" y="${ry+ROW_H/2+4}" font-size="11" fill="#ddd">${escXml(row.milestone.title)}</text>`);
        const mx = tX + dateToX(parseDate(row.milestone.date));
        parts.push(`<rect x="${mx-7}" y="${ry+ROW_H/2-7}" width="14" height="14" rx="2" fill="${escXml(row.milestone.color)}" transform="rotate(45,${mx},${ry+ROW_H/2})"/>`);
      }
    }

    // Dependency arrows
    parts.push(`<defs><marker id="ea" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><polygon points="0 0,7 3.5,0 7" fill="#666"/></marker></defs>`);
    for (const a of arrowData) {
      const shifted = a.path.replace(/([MHVh])(-?\d+\.?\d*),?(-?\d+\.?\d*)?/g, (_m, cmd, x, y) => {
        if (cmd === 'H') return `H${parseFloat(x) + tX}`;
        if (cmd === 'M' || cmd === 'V') return y !== undefined ? `${cmd}${parseFloat(x) + tX},${parseFloat(y) + bY}` : `${cmd}${parseFloat(x) + bY}`;
        return _m;
      });
      parts.push(`<path d="${shifted}" fill="none" stroke="#555" stroke-width="1.5" marker-end="url(#ea)"/>`);
    }

    // Grid separators
    parts.push(`<line x1="${panelWidth}" y1="0" x2="${panelWidth}" y2="${totalH}" stroke="#2a2a44" stroke-width="1"/>`);
    parts.push(`<line x1="0" y1="${HDR+ROW_H}" x2="${totalW}" y2="${HDR+ROW_H}" stroke="#2a2a44" stroke-width="1"/>`);

    parts.push('</svg>');
    return parts.join('\n');
  }

  async function exportSVG() {
    exportDropdownOpen = false;
    try {
      const filePath = await save({
        filters: [{ name: 'SVG Image', extensions: ['svg'] }],
        defaultPath: `${$currentProject?.name ?? 'gantt-chart'}.svg`,
      });
      if (!filePath) return;
      await invoke('write_file', { path: filePath, content: buildExportSVG() });
      showToast('Exported as SVG', 'success');
    } catch (err: any) { showToast('SVG export failed: ' + String(err), 'error'); }
  }

  async function exportJSON() {
    exportDropdownOpen = false;
    try {
      const data = {
        version: 1,
        exportedAt: new Date().toISOString(),
        project: $currentProject,
        tasks: $tasks,
        dependencies: $dependencies,
        milestones: $milestones,
      };
      const filePath = await save({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        defaultPath: `${$currentProject?.name ?? 'gantt-chart'}.json`,
      });
      if (!filePath) return;
      await invoke('write_file', { path: filePath, content: JSON.stringify(data, null, 2) });
      showToast('Exported as JSON', 'success');
    } catch (err: any) { showToast('JSON export failed: ' + String(err), 'error'); }
  }

  async function exportCSV() {
    exportDropdownOpen = false;
    try {
      const titleMap = new Map($tasks.map(t => [t.id, t.title]));
      const depsByTask = new Map<string, string[]>();
      for (const dep of $dependencies) {
        const list = depsByTask.get(dep.to_task_id) ?? [];
        list.push(titleMap.get(dep.from_task_id) ?? dep.from_task_id);
        depsByTask.set(dep.to_task_id, list);
      }
      const cell = (v: string) => (v.includes(',') || v.includes('"') || v.includes('\n'))
        ? `"${v.replace(/"/g, '""')}"` : v;
      const header = 'title,start_date,end_date,duration_days,progress,group,assigned_to,parent_task,dependencies';
      const dataRows = $tasks.map(t => [
        t.title, t.start_date, t.end_date,
        String(dateDuration(t.start_date, t.end_date)), String(t.progress),
        t.group_name ?? '', t.assigned_to ?? '',
        t.parent_task_id ? (titleMap.get(t.parent_task_id) ?? '') : '',
        (depsByTask.get(t.id) ?? []).join('; '),
      ].map(cell).join(','));
      const filePath = await save({
        filters: [{ name: 'CSV Spreadsheet', extensions: ['csv'] }],
        defaultPath: `${$currentProject?.name ?? 'gantt-chart'}.csv`,
      });
      if (!filePath) return;
      await invoke('write_file', { path: filePath, content: [header, ...dataRows].join('\n') });
      showToast('Exported as CSV', 'success');
    } catch (err: any) { showToast('CSV export failed: ' + String(err), 'error'); }
  }

  async function importJSON() {
    exportDropdownOpen = false;
    try {
      const selected = await openDialog({ filters: [{ name: 'Gantt JSON', extensions: ['json'] }], multiple: false });
      if (!selected || Array.isArray(selected)) return;
      const content = await invoke<string>('read_file', { path: selected as string });
      const data = JSON.parse(content) as { version?: number; project?: any; tasks?: any[]; dependencies?: any[]; milestones?: any[] };
      if (!data.project || !Array.isArray(data.tasks)) { showToast('Invalid Gantt JSON file', 'error'); return; }

      isLoading = true;
      const newProject = await createProject({
        name: data.project.name + ' (imported)',
        description: data.project.description,
        start_date: data.project.start_date,
        color_scheme: data.project.color_scheme,
        zoom_level: data.project.zoom_level,
      });
      const idMap = new Map<string, string>();
      for (const t of (data.tasks as GanttTask[]).filter(t => !t.parent_task_id)) {
        const created = await createTask({
          project_id: newProject.id, title: t.title,
          start_date: t.start_date, end_date: t.end_date,
          progress: t.progress, color: t.color, group_name: t.group_name,
          sort_order: t.sort_order, assigned_to: t.assigned_to, notes: t.notes,
        });
        idMap.set(t.id, created.id);
      }
      for (const t of (data.tasks as GanttTask[]).filter(t => !!t.parent_task_id)) {
        const created = await createTask({
          project_id: newProject.id, title: t.title,
          start_date: t.start_date, end_date: t.end_date,
          progress: t.progress, color: t.color, group_name: t.group_name,
          sort_order: t.sort_order, parent_task_id: idMap.get(t.parent_task_id!) ?? null,
          assigned_to: t.assigned_to, notes: t.notes,
        });
        idMap.set(t.id, created.id);
      }
      for (const dep of (data.dependencies ?? [])) {
        const fromId = idMap.get(dep.from_task_id), toId = idMap.get(dep.to_task_id);
        if (fromId && toId) {
          try { await createDependency({ project_id: newProject.id, from_task_id: fromId, to_task_id: toId, dependency_type: dep.dependency_type }); } catch { /* skip circular */ }
        }
      }
      for (const m of (data.milestones ?? [])) {
        await createMilestone({ project_id: newProject.id, title: m.title, date: m.date, color: m.color });
      }
      await loadProjectData(newProject.id);
      isLoading = false;
      showToast('Project imported successfully', 'success');
    } catch (err: any) {
      isLoading = false;
      showToast('Import failed: ' + String(err), 'error');
    }
  }
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} onclick={onWindowClick} onkeydown={onKeyDown} />

<div class="gantt" class:resizing class:link-mode={linkMode} class:show-cp={showCriticalPath}>

  <!-- ── Toolbar ── -->
  <div class="toolbar">

    <button class="tb-btn back-btn" onclick={goBack} title="Back to projects">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <polyline points="15 18 9 12 15 6"/>
      </svg>
    </button>

    {#if editingName}
      <input
        class="name-input"
        bind:value={nameInput}
        onblur={commitNameEdit}
        onkeydown={(e) => { if (e.key === 'Enter') commitNameEdit(); if (e.key === 'Escape') editingName = false; }}
        autofocus
      />
    {:else}
      <button class="project-name" onclick={startNameEdit} title="Click to rename">
        {$currentProject?.name ?? ''}
      </button>
    {/if}

    <div class="tb-sep"></div>

    <div class="zoom-group">
      {#each (['days','weeks','months'] as const) as z}
        <button
          class="tb-btn zoom-btn"
          class:active={$zoomLevel === z}
          onclick={() => zoomLevel.set(z)}
        >{z.charAt(0).toUpperCase() + z.slice(1)}</button>
      {/each}
    </div>

    <div class="tb-sep"></div>

    <button class="tb-btn action-btn" onclick={addTask}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      Task
    </button>

    <button class="tb-btn action-btn" onclick={addMilestone}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
      </svg>
      Milestone
    </button>

    <button class="tb-btn" onclick={scrollToToday}>Today</button>
    <button class="tb-btn" onclick={fitAll}>Fit All</button>

    <button
      class="tb-btn cp-btn"
      class:active={showCriticalPath}
      onclick={() => (showCriticalPath = !showCriticalPath)}
      title="Show critical path"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/>
      </svg>
      CP
    </button>

    <div class="tb-sep"></div>

    <!-- Project progress badge -->
    <div class="progress-badge" title="Project progress (weighted by duration)">
      <svg class="progress-ring" width="26" height="26" viewBox="0 0 26 26">
        <circle cx="13" cy="13" r="10" fill="none" stroke="var(--border)" stroke-width="3"/>
        <circle
          cx="13" cy="13" r="10"
          fill="none"
          stroke="var(--border-focus)"
          stroke-width="3"
          stroke-dasharray="{(projectProgress / 100) * 62.8} 62.8"
          stroke-linecap="round"
          transform="rotate(-90 13 13)"
        />
      </svg>
      <span class="progress-pct">{projectProgress}%</span>
    </div>

    <button
      class="tb-btn link-btn"
      class:active={linkMode}
      onclick={() => { linkMode = !linkMode; linkFromId = null; linkPreviewMouse = null; }}
      title={linkMode ? 'Cancel link mode' : 'Draw dependency link'}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
      </svg>
      Link
    </button>

    <!-- Export dropdown -->
    <div class="export-wrapper" onclick={(e) => e.stopPropagation()}>
      <button
        class="tb-btn export-btn"
        class:active={exportDropdownOpen}
        onclick={() => (exportDropdownOpen = !exportDropdownOpen)}
        title="Export / Import"
        disabled={isExporting}
      >
        {#if isExporting}
          <span class="export-spin">↻</span>
        {:else}
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
        {/if}
        Export
        <svg class="tb-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" style:transform={exportDropdownOpen ? 'rotate(180deg)' : ''}>
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </button>

      {#if exportDropdownOpen}
        <div class="export-dropdown">
          <div class="exp-section-label">Export</div>
          <button class="exp-item" onclick={exportPNG}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
            PNG Image
          </button>
          <button class="exp-item" onclick={exportSVG}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/></svg>
            SVG Vector
          </button>
          <button class="exp-item" onclick={exportJSON}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
            JSON Backup
          </button>
          <button class="exp-item" onclick={exportCSV}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>
            CSV Spreadsheet
          </button>
          <div class="exp-divider"></div>
          <div class="exp-section-label">Import</div>
          <button class="exp-item" onclick={importJSON}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
            Import from JSON
          </button>
        </div>
      {/if}
    </div>

    <button class="tb-btn settings-btn" onclick={() => (editProjectOpen = true)} title="Project settings">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>

  </div>

  <!-- ── Chart body ── -->
  <div class="chart-body" bind:this={chartBodyEl}>

    <!-- LEFT PANEL -->
    <div class="left-panel" style:width="{panelWidth}px" bind:this={leftEl} onscroll={onLeftScroll}>

      <div class="col-header" style:height="{HEADER_H}px">
        <span class="col-title">Task</span>
        <span class="col-start">Start</span>
        <span class="col-end">End</span>
        <span class="col-dur">Dur</span>
      </div>

      <div class="task-rows">
        {#each rows as row, i (rowKey(row, i))}

          {#if row.kind === 'group'}
            <div class="group-row" style:height="{ROW_H}px">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="6 9 12 15 18 9"/>
              </svg>
              {row.name}
            </div>

          {:else if row.kind === 'task'}
            <div
              class="task-row"
              class:selected={$selectedTaskId === row.task.id}
              class:is-parent={row.isParent}
              class:dnd-over={dndOverId === row.task.id}
              style:height="{ROW_H}px"
              style:padding-left="{8 + row.indent * 18}px"
              data-task-id={row.task.id}
              draggable={!row.isParent}
              onclick={() => selectedTaskId.set(row.task.id === $selectedTaskId ? null : row.task.id)}
              ondblclick={() => openTaskModal(row.task)}
              ondragstart={() => (dndRowId = row.task.id)}
              ondragover={(e) => { e.preventDefault(); dndOverId = row.task.id; }}
              ondrop={(e) => { e.preventDefault(); handleDrop(row.task.id); }}
              ondragend={() => { dndRowId = null; dndOverId = null; }}
              role="row"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && openTaskModal(row.task)}
            >
              {#if !row.isParent && row.indent === 0}
                <svg class="drag-grip" width="10" height="14" viewBox="0 0 10 14" fill="currentColor">
                  <circle cx="3" cy="2.5" r="1.3"/><circle cx="7" cy="2.5" r="1.3"/>
                  <circle cx="3" cy="7"   r="1.3"/><circle cx="7" cy="7"   r="1.3"/>
                  <circle cx="3" cy="11.5" r="1.3"/><circle cx="7" cy="11.5" r="1.3"/>
                </svg>
              {/if}
              <span class="color-dot" style:background={row.task.color}></span>
              <span class="col-title task-name" title={row.task.title}>
                <span class="task-name-text">{row.task.title}</span>
                {#if row.task.progress > 0}
                  <span class="row-progress-bar">
                    <span class="row-progress-fill" style:width="{row.task.progress}%" style:background={row.task.color}></span>
                  </span>
                {/if}
              </span>
              <span class="col-start">{formatDate(row.task.start_date)}</span>
              <span class="col-end">{formatDate(row.task.end_date)}</span>
              <span class="col-dur">{durationLabel(row.task.start_date, row.task.end_date)}</span>
            </div>

          {:else}
            <!-- Milestone row -->
            <div class="task-row milestone-row" style:height="{ROW_H}px">
              <svg width="11" height="11" viewBox="0 0 24 24" fill={row.milestone.color} stroke="none">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
              </svg>
              <span class="col-title task-name" title={row.milestone.title}>{row.milestone.title}</span>
              <span class="col-start">{formatDate(row.milestone.date)}</span>
              <span class="col-end">—</span>
              <span class="col-dur">—</span>
            </div>
          {/if}

        {/each}
      </div>
    </div>

    <!-- Resize handle -->
    <div class="resize-handle" onmousedown={onResizeMouseDown} role="separator" aria-orientation="vertical" tabindex="-1"></div>

    <!-- RIGHT PANEL -->
    <div class="right-panel">

      <!-- Time header -->
      <div class="time-header" style:height="{HEADER_H}px">
        <div class="header-inner" bind:this={headerInner}>
          <div class="major-row" style:width="{columns.length * colW}px">
            {#each majorGroups as g}
              <div class="major-cell" style:width="{g.span * colW}px">{g.label}</div>
            {/each}
          </div>
          <div class="minor-row" style:width="{columns.length * colW}px">
            {#each columns as col}
              <div class="minor-cell" class:major-marker={col.major} class:weekend-cell={col.isWeekend} style:width="{colW}px">{col.label}</div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Scrollable timeline -->
      <div class="timeline-body" bind:this={timelineEl} onscroll={onTimelineScroll} onmousemove={onTimelineBodyMouseMove} onwheel={onTimelineWheel}>
        <div
          class="grid-bg"
          style:width="{columns.length * colW}px"
          style:min-height="{rows.length * ROW_H}px"
        >

          <!-- Vertical grid columns -->
          {#each columns as col, ci}
            <div class="grid-col" class:grid-col-major={col.major} class:grid-col-weekend={col.isWeekend} style:left="{ci * colW}px" style:width="{colW}px"></div>
          {/each}

          <!-- Row content -->
          {#each rows as row, ri (rowKey(row, ri))}
            {@const rowY = ri * ROW_H}

            {#if row.kind === 'group'}
              <!-- Group band -->
              <div class="group-band" style:top="{rowY}px" style:height="{ROW_H}px" style:width="{columns.length * colW}px">
                <span class="group-band-label">{row.name}</span>
              </div>

            {:else if row.kind === 'task'}
              <!-- Row stripe -->
              <div class="row-stripe" class:row-stripe-alt={ri % 2 === 1} style:top="{rowY}px" style:height="{ROW_H}px" style:width="{columns.length * colW}px"></div>

              {#if row.isParent}
                <!-- Summary bar (spans children range) -->
                {@const sb = getSummaryBounds(row.task.id)}
                {#if sb}
                  <GanttTaskBar
                    task={row.task}
                    x={sb.x}
                    y={rowY + ROW_H / 2 - 4}
                    width={sb.w}
                    height={8}
                    isSummary={true}
                    isCritical={showCriticalPath && criticalPathIds.has(row.task.id)}
                    isSelected={$selectedTaskId === row.task.id}
                    isDragging={drag?.taskId === row.task.id}
                    onbardown={() => {}}
                    ondblclick={() => openTaskModal(row.task)}
                    oncontextmenu={(e) => onBarContextMenu(e, row.task)}
                  />
                {/if}
              {:else if row.task.start_date === row.task.end_date}
                <!-- Zero-duration: render as a task diamond -->
                {@const zdx = dateToX(parseDate(row.task.start_date))}
                <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                <div
                  class="task-diamond"
                  class:task-diamond-selected={$selectedTaskId === row.task.id}
                  style:left="{zdx - 10}px"
                  style:top="{rowY + ROW_H / 2 - 10}px"
                  style:background={row.task.color}
                  title={row.task.title}
                  onmousedown={() => selectedTaskId.set(row.task.id)}
                  ondblclick={() => openTaskModal(row.task)}
                  oncontextmenu={(e) => onBarContextMenu(e, row.task)}
                  role="button"
                  tabindex="0"
                  aria-label={row.task.title}
                ></div>
              {:else}
                <!-- Normal / sub-task bar -->
                {@const bh = row.indent > 0 ? 22 : 28}
                <GanttTaskBar
                  task={row.task}
                  x={dateToX(parseDate(row.task.start_date))}
                  y={rowY + (ROW_H - bh) / 2}
                  width={Math.max(barWidth(row.task), 6)}
                  height={bh}
                  isCritical={showCriticalPath && criticalPathIds.has(row.task.id)}
                  isSelected={$selectedTaskId === row.task.id}
                  isDragging={drag?.taskId === row.task.id}
                  showProgress={$settings.ganttShowProgressBars}
                  onbardown={(e, mode) => onTaskBarDown(e, row.task, mode)}
                  ondblclick={() => openTaskModal(row.task)}
                  oncontextmenu={(e) => onBarContextMenu(e, row.task)}
                />
              {/if}

            {:else}
              <!-- Milestone row stripe -->
              <div class="row-stripe" class:row-stripe-alt={ri % 2 === 1} style:top="{rowY}px" style:height="{ROW_H}px" style:width="{columns.length * colW}px"></div>
              <!-- Diamond -->
              {@const mx = dateToX(parseDate(row.milestone.date))}
              <div
                class="milestone-diamond"
                style:left="{mx - 9}px"
                style:top="{rowY + ROW_H / 2 - 9}px"
                style:background={row.milestone.color}
                onmouseenter={(e) => tooltip = { x: e.clientX + 10, y: e.clientY - 36, text: `${row.milestone.title} · ${formatDate(row.milestone.date)}` }}
                onmouseleave={() => (tooltip = null)}
                role="img"
                aria-label={row.milestone.title}
              ></div>
            {/if}
          {/each}

          <!-- Dependency arrows SVG overlay -->
          <svg
            class="dep-svg"
            style:width="{columns.length * colW}px"
            style:height="{rows.length * ROW_H}px"
            xmlns="http://www.w3.org/2000/svg"
          >
            <defs>
              <marker id="arr-norm" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto">
                <polygon points="0 0, 7 3.5, 0 7" fill="var(--text-muted)" opacity="0.7"/>
              </marker>
              <marker id="arr-hl" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto">
                <polygon points="0 0, 7 3.5, 0 7" fill="var(--border-focus)"/>
              </marker>
            </defs>

            {#each arrowData as a (a.dep.id)}
              <!-- Hit area (wide invisible stroke) -->
              <path
                d={a.path}
                fill="none"
                stroke="transparent"
                stroke-width="12"
                style="pointer-events: stroke; cursor: pointer"
                onmouseenter={() => (hoveredDepId = a.dep.id)}
                onmouseleave={() => (hoveredDepId = null)}
                oncontextmenu={(e) => onDepContextMenu(e, a.dep)}
              />
              <!-- Visible arrow -->
              <path
                class="dep-arrow"
                d={a.path}
                fill="none"
                stroke={a.isHl ? 'var(--border-focus)' : 'var(--text-muted)'}
                stroke-width={a.isHl ? 2 : 1.5}
                stroke-opacity={a.isHl ? 1 : 0.7}
                marker-end={a.isHl ? 'url(#arr-hl)' : 'url(#arr-norm)'}
                pointer-events="none"
              />
            {/each}

            <!-- Link-mode preview line -->
            {#if linkPreviewLine}
              <line
                x1={linkPreviewLine.sx} y1={linkPreviewLine.sy}
                x2={linkPreviewLine.tx} y2={linkPreviewLine.ty}
                stroke="var(--border-focus)"
                stroke-width="2"
                stroke-dasharray="6 4"
                pointer-events="none"
                opacity="0.85"
              />
            {/if}

            <!-- Link source indicator ring -->
            {#if linkMode && linkFromId}
              {@const edges = getTaskBarEdges(linkFromId)}
              {#if edges}
                <circle
                  cx={edges.rightX}
                  cy={edges.centerY}
                  r="6"
                  fill="var(--border-focus)"
                  fill-opacity="0.3"
                  stroke="var(--border-focus)"
                  stroke-width="2"
                  pointer-events="none"
                />
              {/if}
            {/if}
          </svg>

          <!-- Ghost bar during drag -->
          {#if drag && ghostRowIdx >= 0}
            {@const ghostTask = $tasks.find(t => t.id === drag!.taskId)}
            {#if ghostTask}
              <div
                class="bar-ghost"
                style:left="{drag.ghostX}px"
                style:top="{ghostRowIdx * ROW_H + 4}px"
                style:width="{Math.max(drag.ghostW, 6)}px"
                style:height="{ROW_H - 8}px"
                style:background={ghostTask.color}
              ></div>
            {/if}
          {/if}

          <!-- Today line -->
          {#if todayX >= 0}
            <div class="today-line" style:left="{todayX}px" style:height="{rows.length * ROW_H}px"></div>
          {/if}

        </div>

        <!-- Empty state -->
        {#if rows.length === 0 && !isLoading}
          <div class="empty-state">
            <svg width="72" height="72" viewBox="0 0 72 72" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect x="8" y="22" width="56" height="8" rx="4" fill="var(--border)" opacity="0.4"/>
              <rect x="8" y="36" width="38" height="8" rx="4" fill="var(--border)" opacity="0.3"/>
              <rect x="8" y="50" width="46" height="8" rx="4" fill="var(--border)" opacity="0.2"/>
              <rect x="16" y="22" width="20" height="8" rx="4" fill="var(--border-focus)" opacity="0.5"/>
              <rect x="22" y="36" width="28" height="8" rx="4" fill="var(--border-focus)" opacity="0.4"/>
              <rect x="12" y="50" width="18" height="8" rx="4" fill="var(--border-focus)" opacity="0.3"/>
            </svg>
            <h3 class="empty-title">No tasks yet</h3>
            <p class="empty-subtitle">Click <strong>Task</strong> to add your first task, or use <strong>Milestone</strong> to mark key dates.</p>
            <button class="empty-add-btn" onclick={addTask}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              Add First Task
            </button>
          </div>
        {/if}

        <!-- Loading skeleton -->
        {#if isLoading}
          <div class="skeleton-overlay">
            {#each SKELETON_ROWS as sk, i}
              <div class="skel-row" style:animation-delay="{i * 0.06}s">
                <div class="skel skel-label"></div>
                <div class="skel skel-bar" style:width="{sk.w}px" style:margin-left="{sk.ml}px"></div>
              </div>
            {/each}
          </div>
        {/if}

      </div>

      <!-- ── Minimap ── -->
      <div class="minimap" bind:clientWidth={minimapWidth}>
        <svg class="minimap-svg" width={minimapWidth} height="40" xmlns="http://www.w3.org/2000/svg" style="pointer-events:none">
          {#each rows as row, ri (rowKey(row, ri))}
            {#if row.kind === 'task'}
              {@const bx = (row.isParent
                ? (getSummaryBounds(row.task.id)?.x ?? dateToX(parseDate(row.task.start_date)))
                : dateToX(parseDate(row.task.start_date))) * minimapScale}
              {@const bw = Math.max((row.isParent
                ? (getSummaryBounds(row.task.id)?.w ?? barWidth(row.task))
                : barWidth(row.task)) * minimapScale, 2)}
              {@const by = (ri / Math.max(rows.length, 1)) * 38 + 1}
              {@const bh = Math.max(38 / Math.max(rows.length, 1) - 0.5, 1.5)}
              <rect x={bx} y={by} width={bw} height={bh} fill={row.task.color} opacity="0.75" rx="0.5"/>
            {/if}
          {/each}
          {#if todayX >= 0}
            <line x1={todayX * minimapScale} y1="0" x2={todayX * minimapScale} y2="40" stroke="#ef4444" stroke-width="1" opacity="0.6"/>
          {/if}
        </svg>
        <!-- Viewport indicator -->
        <div
          class="minimap-viewport"
          style:left="{Math.max(0, timelineScrollLeft * minimapScale)}px"
          style:width="{Math.max((timelineEl?.clientWidth ?? 200) * minimapScale, 20)}px"
          onmousedown={(e) => {
            minimapDragging = true;
            _mmDragStartX = e.clientX;
            _mmDragStartScroll = timelineScrollLeft;
            e.preventDefault();
          }}
          role="scrollbar"
          aria-orientation="horizontal"
          tabindex="-1"
          aria-valuenow={timelineScrollLeft}
          aria-valuemin={0}
          aria-valuemax={columns.length * colW}
        ></div>
      </div>
    </div>
  </div>

  <!-- ── Auto-schedule confirm ── -->
  {#if autoScheduleConfirm}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="autosched-overlay" onclick={() => (autoScheduleConfirm = null)}>
      <div class="autosched-popover" onclick={(e) => e.stopPropagation()}>
        <p class="autosched-msg">
          This will shift <strong>{autoScheduleConfirm.affectedIds.length}</strong> dependent
          task{autoScheduleConfirm.affectedIds.length > 1 ? 's' : ''} forward by
          <strong>{autoScheduleConfirm.deltaDays}</strong> day{autoScheduleConfirm.deltaDays > 1 ? 's' : ''}.
        </p>
        <div class="autosched-footer">
          <button class="btn-skip" onclick={() => (autoScheduleConfirm = null)}>Skip</button>
          <button class="btn-confirm" onclick={confirmAutoSchedule}>Move Tasks</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- ── Context menu (fixed) ── -->
  {#if contextMenu}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="ctx-menu" style:left="{contextMenu.x}px" style:top="{contextMenu.y}px" onclick={(e) => e.stopPropagation()}>

      {#if contextMenu.page === 'main'}
        <button class="ctx-item" onclick={ctxEdit}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          Edit…
        </button>
        <button class="ctx-item" onclick={ctxAddSubtask}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          Add Sub-task
        </button>
        <button class="ctx-item" onclick={() => { if (contextMenu) contextMenu.page = 'deps'; }}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="7 17 12 12 7 7"/><polyline points="17 17 12 12 17 7"/></svg>
          Add Dependency
          <svg class="ctx-arrow" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="9 18 15 12 9 6"/></svg>
        </button>
        <button class="ctx-item" onclick={() => { if (contextMenu) contextMenu.page = 'colors'; }}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><circle cx="8" cy="10" r="1.5" fill="currentColor"/><circle cx="14" cy="8" r="1.5" fill="currentColor"/><circle cx="16" cy="14" r="1.5" fill="currentColor"/></svg>
          Set Color
          <svg class="ctx-arrow" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="9 18 15 12 9 6"/></svg>
        </button>
        <div class="ctx-divider"></div>
        <button class="ctx-item ctx-danger" onclick={ctxDelete}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
          Delete
        </button>

      {:else if contextMenu.page === 'colors'}
        <button class="ctx-item ctx-back" onclick={() => { if (contextMenu) contextMenu.page = 'main'; }}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          Back
        </button>
        <div class="ctx-swatches">
          {#each TASK_COLORS as c}
            <button class="ctx-swatch" style:background={c} onclick={() => ctxSetColor(c)} aria-label={c}></button>
          {/each}
        </div>

      {:else if contextMenu.page === 'deps'}
        <button class="ctx-item ctx-back" onclick={() => { if (contextMenu) contextMenu.page = 'main'; }}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          Back
        </button>
        <div class="ctx-section-label">Select predecessor:</div>
        <div class="ctx-dep-list">
          {#each $tasks.filter(t => t.id !== contextMenu!.taskId) as t}
            <button class="ctx-item" onclick={() => ctxAddDep(t.id)}>{t.title}</button>
          {/each}
          {#if $tasks.filter(t => t.id !== contextMenu!.taskId).length === 0}
            <span class="ctx-empty">No other tasks</span>
          {/if}
        </div>
      {/if}

    </div>
  {/if}

  <!-- ── Dep context menu (fixed) ── -->
  {#if depCtxMenu}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="ctx-menu" style:left="{depCtxMenu.x}px" style:top="{depCtxMenu.y}px" onclick={(e) => e.stopPropagation()}>

      {#if depCtxMenu.page === 'main'}
        {@const fromTask = $tasks.find(t => t.id === depCtxMenu!.dep.from_task_id)}
        {@const toTask   = $tasks.find(t => t.id === depCtxMenu!.dep.to_task_id)}
        <div class="ctx-section-label">{fromTask?.title ?? '?'} → {toTask?.title ?? '?'}</div>
        <div class="ctx-section-label" style="padding-top:0;opacity:0.6">{depTypeLabel(depCtxMenu.dep.dependency_type)}</div>
        <div class="ctx-divider"></div>
        <button class="ctx-item" onclick={() => { if (depCtxMenu) depCtxMenu.page = 'types'; }}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="7 17 12 12 7 7"/><polyline points="17 17 12 12 17 7"/></svg>
          Change Type
          <svg class="ctx-arrow" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="9 18 15 12 9 6"/></svg>
        </button>
        <div class="ctx-divider"></div>
        <button class="ctx-item ctx-danger" onclick={depCtxDelete}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg>
          Delete Dependency
        </button>

      {:else}
        <button class="ctx-item ctx-back" onclick={() => { if (depCtxMenu) depCtxMenu.page = 'main'; }}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          Back
        </button>
        {#each ['finish_to_start','start_to_start','finish_to_finish','start_to_finish'] as type}
          <button
            class="ctx-item"
            class:ctx-active={depCtxMenu.dep.dependency_type === type}
            onclick={() => depCtxChangeType(type)}
          >{depTypeLabel(type)}</button>
        {/each}
      {/if}

    </div>
  {/if}

  <!-- ── Milestone tooltip (fixed) ── -->
  {#if tooltip}
    <div class="tooltip" style:left="{tooltip.x}px" style:top="{tooltip.y}px">{tooltip.text}</div>
  {/if}

  <!-- ── Toasts ── -->
  {#if toasts.length > 0}
    <div class="toast-container">
      {#each toasts as t (t.id)}
        <div class="toast toast-{t.type}">{t.msg}</div>
      {/each}
    </div>
  {/if}

</div>

<!-- Modals -->
<ProjectModal
  open={editProjectOpen}
  project={$currentProject}
  onclose={() => (editProjectOpen = false)}
  onsaved={() => {}}
  ondeleted={goBack}
/>

<TaskModal
  open={taskModalOpen}
  task={taskModalTask}
  projectId={$currentProject?.id ?? ''}
  defaultParentId={newTaskParentId}
  onclose={() => { taskModalOpen = false; taskModalTask = null; newTaskParentId = ''; }}
  onsaved={() => {}}
  ondeleted={() => {}}
/>

<style>
  .gantt {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-main);
    overflow: hidden;
    user-select: none;
  }
  .gantt.resizing { cursor: col-resize; }

  /* ── Toolbar ─────────────────────────────────────────────────────────────── */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    height: 44px;
    flex-shrink: 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
  }

  .tb-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.8rem;
    font-weight: 500;
    padding: 4px 10px;
    display: flex;
    align-items: center;
    gap: 5px;
    white-space: nowrap;
    transition: all 0.15s;
  }
  .tb-btn:hover { background: var(--hover-bg); color: var(--text-main); border-color: var(--border); }

  .back-btn { padding: 4px 8px; }

  .project-name {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    color: var(--text-main);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.92rem;
    font-weight: 700;
    padding: 3px 8px;
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: border-color 0.15s;
  }
  .project-name:hover { border-color: var(--border); }

  .name-input {
    background: var(--bg-main);
    border: 1px solid var(--border-focus);
    border-radius: 5px;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.92rem;
    font-weight: 700;
    outline: none;
    padding: 3px 8px;
    width: 200px;
  }

  .tb-sep { width: 1px; height: 20px; background: var(--border); margin: 0 4px; flex-shrink: 0; }

  .zoom-group {
    display: flex;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 5px;
    overflow: hidden;
  }
  .zoom-btn { border-radius: 0; border: none; padding: 4px 12px; }
  .zoom-btn:not(:last-child) { border-right: 1px solid var(--border); }
  .zoom-btn.active { background: var(--border-focus); color: #fff; }
  .zoom-btn.active:hover { background: var(--border-focus); }

  .action-btn { border: 1px solid var(--border); }
  .settings-btn { margin-left: auto; }

  /* CP button */
  .cp-btn.active {
    background: color-mix(in srgb, #ef4444 15%, transparent);
    border-color: #ef4444;
    color: #ef4444;
  }
  .cp-btn.active:hover { background: color-mix(in srgb, #ef4444 25%, transparent); }

  /* Project progress badge */
  .progress-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 4px;
    flex-shrink: 0;
  }
  .progress-ring { flex-shrink: 0; }
  .progress-pct {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-muted);
    min-width: 28px;
  }

  /* ── Chart body ──────────────────────────────────────────────────────────── */
  .chart-body {
    flex: 1;
    display: flex;
    flex-direction: row;
    overflow: hidden;
    min-height: 0;
  }

  /* ── Left panel ──────────────────────────────────────────────────────────── */
  .left-panel {
    flex-shrink: 0;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }
  .left-panel::-webkit-scrollbar { width: 6px; }
  .left-panel::-webkit-scrollbar-track { background: transparent; }
  .left-panel::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  .col-header {
    display: flex;
    align-items: center;
    padding: 0 8px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    position: sticky;
    top: 0;
    background: var(--bg-panel);
    z-index: 2;
  }

  .task-rows { flex: 1; }

  /* Group header row */
  .group-row {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 0 10px;
    font-size: 0.74rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    background: color-mix(in srgb, var(--border) 20%, transparent);
    border-bottom: 1px solid var(--border);
    box-sizing: border-box;
  }

  /* Task row */
  .task-row {
    display: flex;
    align-items: center;
    gap: 5px;
    padding-right: 8px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    transition: background 0.1s;
    box-sizing: border-box;
  }
  .task-row:hover { background: var(--hover-bg); }
  .task-row.selected { background: color-mix(in srgb, var(--border-focus) 12%, transparent); }
  .task-row.is-parent { font-weight: 600; }
  .task-row.dnd-over { border-top: 2px solid var(--border-focus); }
  .milestone-row { cursor: default; }

  .drag-grip { color: var(--text-muted); opacity: 0; flex-shrink: 0; cursor: grab; }
  .task-row:hover .drag-grip { opacity: 0.5; }

  .color-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }

  .col-title  { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.8rem; color: var(--text-main); display: flex; align-items: center; gap: 4px; }
  .col-start, .col-end { width: 44px; flex-shrink: 0; font-size: 0.7rem; color: var(--text-muted); text-align: center; }
  .col-dur  { width: 30px; flex-shrink: 0; font-size: 0.7rem; color: var(--text-muted); text-align: right; }

  .col-header .col-title,
  .col-header .col-start,
  .col-header .col-end,
  .col-header .col-dur {
    font-size: 0.68rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .task-name { gap: 2px; flex-direction: column; align-items: flex-start; overflow: hidden; white-space: normal; }
  .task-name-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; width: 100%; }

  /* ── Resize handle ───────────────────────────────────────────────────────── */
  .resize-handle {
    width: 5px;
    flex-shrink: 0;
    background: transparent;
    cursor: col-resize;
    transition: background 0.15s;
    position: relative;
    z-index: 3;
  }
  .resize-handle:hover, .resizing .resize-handle { background: var(--border-focus); }

  /* ── Right panel ─────────────────────────────────────────────────────────── */
  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  /* Time header */
  .time-header {
    flex-shrink: 0;
    overflow: hidden;
    border-bottom: 1px solid var(--border);
    background: var(--bg-panel);
    position: relative;
    z-index: 2;
  }
  .header-inner { overflow: hidden; display: flex; flex-direction: column; height: 100%; }
  .major-row, .minor-row { display: flex; flex: 1; overflow: hidden; }
  .major-row { border-bottom: 1px solid var(--border); }
  .major-cell {
    flex-shrink: 0; font-size: 0.72rem; font-weight: 600; color: var(--text-muted);
    padding: 0 8px; display: flex; align-items: center;
    border-right: 1px solid var(--border); white-space: nowrap; overflow: hidden;
  }
  .minor-cell {
    flex-shrink: 0; font-size: 0.68rem; color: var(--text-muted);
    display: flex; align-items: center; justify-content: center;
    border-right: 1px solid var(--border); white-space: nowrap; overflow: hidden;
  }
  .minor-cell.major-marker { border-right-color: color-mix(in srgb, var(--border) 50%, var(--text-muted)); font-weight: 600; }

  /* Timeline body */
  .timeline-body { flex: 1; overflow: auto; position: relative; }
  .timeline-body::-webkit-scrollbar { width: 6px; height: 6px; }
  .timeline-body::-webkit-scrollbar-track { background: transparent; }
  .timeline-body::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  /* Grid */
  .grid-bg { position: relative; }
  .grid-col { position: absolute; top: 0; bottom: 0; border-right: 1px solid var(--border); pointer-events: none; }
  .grid-col-major { border-right-color: color-mix(in srgb, var(--border) 50%, var(--text-muted)); background: color-mix(in srgb, var(--border) 18%, transparent); }

  /* Row stripes */
  .row-stripe { position: absolute; left: 0; pointer-events: none; }
  .row-stripe-alt { background: color-mix(in srgb, var(--border) 12%, transparent); }

  /* Group band in timeline */
  .group-band {
    position: absolute;
    left: 0;
    background: color-mix(in srgb, var(--border) 25%, transparent);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    pointer-events: none;
  }
  .group-band-label {
    padding-left: 10px;
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    white-space: nowrap;
  }

  /* Milestone diamond */
  .milestone-diamond {
    position: absolute;
    width: 18px;
    height: 18px;
    transform: rotate(45deg);
    border-radius: 2px;
    cursor: pointer;
    z-index: 3;
    transition: filter 0.1s;
  }
  .milestone-diamond:hover { filter: brightness(1.2); }

  /* Ghost bar */
  .bar-ghost {
    position: absolute;
    border-radius: 4px;
    opacity: 0.55;
    border: 2px dashed #fff;
    pointer-events: none;
    z-index: 5;
  }

  /* Today line */
  .today-line {
    position: absolute;
    top: 0;
    width: 2px;
    background: repeating-linear-gradient(to bottom, #ef4444 0px, #ef4444 4px, transparent 4px, transparent 8px);
    pointer-events: none;
    z-index: 4;
  }

  /* ── Context menu ────────────────────────────────────────────────────────── */
  .ctx-menu {
    position: fixed;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 7px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    min-width: 160px;
    z-index: 9000;
    overflow: hidden;
    padding: 4px 0;
  }

  .ctx-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.82rem;
    padding: 7px 14px;
    text-align: left;
    transition: background 0.1s;
  }
  .ctx-item:hover { background: var(--hover-bg); }
  .ctx-danger { color: #ef4444; }
  .ctx-danger:hover { background: color-mix(in srgb, #ef4444 12%, transparent); }
  .ctx-back { color: var(--text-muted); font-size: 0.78rem; }

  .ctx-arrow { margin-left: auto; color: var(--text-muted); }
  .ctx-divider { height: 1px; background: var(--border); margin: 3px 0; }
  .ctx-section-label { font-size: 0.7rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; padding: 4px 14px 2px; }

  .ctx-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 12px 10px;
  }
  .ctx-swatch {
    width: 20px; height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s;
  }
  .ctx-swatch:hover { transform: scale(1.2); border-color: #fff; }

  .ctx-dep-list { max-height: 160px; overflow-y: auto; }
  .ctx-empty { display: block; font-size: 0.78rem; color: var(--text-muted); padding: 6px 14px; }
  .ctx-active { background: color-mix(in srgb, var(--border-focus) 12%, transparent); color: var(--border-focus); }

  /* ── Tooltip ─────────────────────────────────────────────────────────────── */
  .tooltip {
    position: fixed;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 5px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    color: var(--text-main);
    font-size: 0.78rem;
    padding: 5px 10px;
    pointer-events: none;
    white-space: nowrap;
    z-index: 9001;
  }

  /* ── Critical path dim ──────────────────────────────────────────────────── */
  :global(.gantt.show-cp .bar:not(.bar-critical)) {
    opacity: 0.35;
    filter: grayscale(0.4);
  }

  /* ── Inline progress bar ─────────────────────────────────────────────────── */
  .row-progress-bar {
    display: block;
    width: 100%;
    height: 3px;
    background: var(--border);
    border-radius: 2px;
    overflow: hidden;
    margin-top: 3px;
    flex-shrink: 0;
  }
  .row-progress-fill {
    display: block;
    height: 100%;
    border-radius: 2px;
    opacity: 0.85;
    transition: width 0.3s;
  }

  /* ── Minimap ─────────────────────────────────────────────────────────────── */
  .minimap {
    height: 40px;
    flex-shrink: 0;
    background: var(--bg-main);
    border-top: 1px solid var(--border);
    position: relative;
    overflow: hidden;
    cursor: default;
  }

  .minimap-svg {
    position: absolute;
    top: 0; left: 0;
    display: block;
  }

  .minimap-viewport {
    position: absolute;
    top: 0;
    height: 100%;
    background: rgba(255,255,255,0.08);
    border: 1px solid rgba(255,255,255,0.22);
    border-radius: 2px;
    cursor: grab;
    transition: background 0.1s;
  }
  .minimap-viewport:hover { background: rgba(255,255,255,0.14); }
  .minimap-viewport:active { cursor: grabbing; }

  /* ── Auto-schedule confirm ───────────────────────────────────────────────── */
  .autosched-overlay {
    position: fixed;
    inset: 0;
    z-index: 8000;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
  }

  .autosched-popover {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 12px 36px rgba(0,0,0,0.5);
    padding: 16px 20px 14px;
    min-width: 300px;
    max-width: 400px;
  }

  .autosched-msg {
    margin: 0 0 14px;
    font-size: 0.86rem;
    color: var(--text-main);
    line-height: 1.5;
  }

  .autosched-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn-skip {
    background: var(--hover-bg);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.82rem;
    padding: 5px 14px;
  }
  .btn-skip:hover { color: var(--text-main); }

  .btn-confirm {
    background: var(--border-focus);
    border: none;
    border-radius: 5px;
    color: #fff;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.82rem;
    font-weight: 600;
    padding: 5px 14px;
  }
  .btn-confirm:hover { opacity: 0.85; }

  /* ── Dependency arrows ───────────────────────────────────────────────────── */
  .dep-svg {
    position: absolute;
    top: 0;
    left: 0;
    overflow: visible;
    pointer-events: none;
    z-index: 2;
  }

  /* Link mode: show crosshair on timeline task bars */
  .link-mode .timeline-body { cursor: crosshair; }

  /* Link mode toolbar button active state */
  .link-btn.active {
    background: color-mix(in srgb, var(--border-focus) 15%, transparent);
    border-color: var(--border-focus);
    color: var(--border-focus);
  }
  .link-btn.active:hover {
    background: color-mix(in srgb, var(--border-focus) 25%, transparent);
  }

  /* ── Toasts ──────────────────────────────────────────────────────────────── */
  .toast-container {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    pointer-events: none;
    z-index: 9500;
  }

  .toast {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
    color: var(--text-main);
    font-size: 0.82rem;
    padding: 8px 18px;
    white-space: nowrap;
    animation: toast-in 0.2s ease;
  }

  .toast-success { border-color: #22c55e; color: #22c55e; background: color-mix(in srgb, #22c55e 10%, var(--bg-panel)); }
  .toast-warning { border-color: #f97316; color: #f97316; background: color-mix(in srgb, #f97316 10%, var(--bg-panel)); }
  .toast-error   { border-color: #ef4444; color: #ef4444; background: color-mix(in srgb, #ef4444 10%, var(--bg-panel)); }

  @keyframes toast-in {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  /* ── Weekend columns ─────────────────────────────────────────────────────── */
  .grid-col-weekend { background: color-mix(in srgb, var(--border) 28%, transparent); }
  .weekend-cell { opacity: 0.55; }

  /* ── Dependency arrow draw animation ─────────────────────────────────────── */
  .dep-arrow {
    animation: arrow-draw 0.35s ease forwards;
    stroke-dasharray: 2000;
    stroke-dashoffset: 0;
  }
  @keyframes arrow-draw {
    from { stroke-dashoffset: 2000; opacity: 0; }
    to   { stroke-dashoffset: 0;    opacity: 1; }
  }

  /* ── Zoom transition on bars ─────────────────────────────────────────────── */
  :global(.gantt .bar) {
    transition: left 0.22s ease, width 0.22s ease, top 0.22s ease, filter 0.1s, opacity 0.1s;
  }

  /* ── Zero-duration diamond (task) ────────────────────────────────────────── */
  .task-diamond {
    position: absolute;
    width: 20px;
    height: 20px;
    transform: rotate(45deg);
    border-radius: 3px;
    cursor: pointer;
    z-index: 3;
    transition: filter 0.1s, box-shadow 0.1s;
  }
  .task-diamond:hover { filter: brightness(1.2); box-shadow: 0 3px 10px rgba(0,0,0,0.4); transform: rotate(45deg) scale(1.1); }
  .task-diamond-selected {
    outline: 2px solid #fff;
    outline-offset: 2px;
    box-shadow: 0 0 0 4px var(--border-focus);
  }

  /* ── Export dropdown ─────────────────────────────────────────────────────── */
  .export-wrapper {
    position: relative;
    flex-shrink: 0;
  }

  .export-btn { border: 1px solid var(--border); gap: 5px; }
  .export-btn.active { background: var(--hover-bg); border-color: var(--border-focus); }
  .export-btn:disabled { opacity: 0.6; cursor: not-allowed; }

  .export-spin {
    display: inline-block;
    animation: spin 0.8s linear infinite;
    font-size: 1rem;
    line-height: 1;
  }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

  .tb-chevron { transition: transform 0.15s; flex-shrink: 0; }

  .export-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 28px rgba(0,0,0,0.45);
    min-width: 170px;
    z-index: 9000;
    padding: 4px 0;
    animation: dropdown-in 0.12s ease;
  }
  @keyframes dropdown-in {
    from { opacity: 0; transform: translateY(-4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .exp-section-label {
    font-size: 0.68rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 5px 12px 3px;
  }

  .exp-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.82rem;
    padding: 7px 12px;
    text-align: left;
    transition: background 0.1s;
  }
  .exp-item:hover { background: var(--hover-bg); }

  .exp-divider { height: 1px; background: var(--border); margin: 3px 0; }

  /* ── Empty state ─────────────────────────────────────────────────────────── */
  .empty-state {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    pointer-events: none;
    z-index: 1;
  }
  .empty-state > * { pointer-events: auto; }

  .empty-title {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--text-main);
  }
  .empty-subtitle {
    margin: 0;
    font-size: 0.84rem;
    color: var(--text-muted);
    text-align: center;
    max-width: 300px;
    line-height: 1.5;
  }
  .empty-add-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--border-focus);
    border: none;
    border-radius: 6px;
    color: #fff;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.84rem;
    font-weight: 600;
    padding: 7px 16px;
    margin-top: 4px;
    transition: opacity 0.15s;
  }
  .empty-add-btn:hover { opacity: 0.85; }

  /* ── Loading skeleton ────────────────────────────────────────────────────── */
  .skeleton-overlay {
    position: absolute;
    inset: 0;
    background: var(--bg-main);
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-top: 4px;
  }
  .skel-row {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 36px;
    padding: 0 12px;
    animation: skel-pulse 1.4s ease-in-out infinite;
  }
  .skel {
    border-radius: 4px;
    background: linear-gradient(90deg, var(--border) 25%, color-mix(in srgb, var(--border) 60%, var(--bg-main)) 50%, var(--border) 75%);
    background-size: 200% 100%;
    animation: skel-shimmer 1.6s linear infinite;
  }
  .skel-label { width: 100px; height: 14px; flex-shrink: 0; }
  .skel-bar   { height: 20px; border-radius: 4px; }

  @keyframes skel-shimmer {
    from { background-position: 200% 0; }
    to   { background-position: -200% 0; }
  }
  @keyframes skel-pulse {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.7; }
  }
</style>
