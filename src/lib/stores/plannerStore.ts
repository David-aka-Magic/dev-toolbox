import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ─── TypeScript interfaces ────────────────────────────────────────────────────

export interface Event {
  id: string;
  title: string;
  description: string | null;
  start_time: string;
  end_time: string;
  all_day: boolean;
  color: string;
  recurrence_rule: string | null;
  created_at: string;
  updated_at: string;
}

export interface EventInput {
  title: string;
  description?: string | null;
  start_time: string;
  end_time: string;
  all_day: boolean;
  color?: string | null;
  recurrence_rule?: string | null;
}

export interface Task {
  id: string;
  title: string;
  description: string | null;
  due_date: string | null;
  due_time: string | null;
  /** 0 = none, 1 = low, 2 = medium, 3 = high */
  priority: number;
  completed: boolean;
  completed_at: string | null;
  list_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface TaskInput {
  title: string;
  description?: string | null;
  due_date?: string | null;
  due_time?: string | null;
  priority: number;
  completed: boolean;
  list_id?: string | null;
  sort_order?: number | null;
}

export interface TaskList {
  id: string;
  name: string;
  color: string;
  sort_order: number;
}

export interface TimeBlock {
  id: string;
  title: string;
  start_time: string;
  end_time: string;
  color: string;
  event_id: string | null;
  task_id: string | null;
  created_at: string;
}

export interface TimeBlockInput {
  title: string;
  start_time: string;
  end_time: string;
  color?: string | null;
  event_id?: string | null;
  task_id?: string | null;
}

// ─── Stores ───────────────────────────────────────────────────────────────────

export const events = writable<Event[]>([]);
export const tasks = writable<Task[]>([]);
export const taskLists = writable<TaskList[]>([]);
export const timeBlocks = writable<TimeBlock[]>([]);
export const currentDate = writable<string>(new Date().toISOString().split('T')[0]);
export const calendarView = writable<'month' | 'week' | 'day'>('month');
export const selectedDate = writable<string | null>(null);

// ─── Event functions ──────────────────────────────────────────────────────────

export async function loadEvents(startDate: string, endDate: string): Promise<Event[]> {
  const [regular, recurring] = await Promise.all([
    invoke<Event[]>('get_events', { startDate, endDate }),
    invoke<Event[]>('get_recurring_event_instances', { startDate, endDate }),
  ]);
  const all = [...regular, ...recurring].sort((a, b) =>
    a.start_time.localeCompare(b.start_time)
  );
  events.set(all);
  return all;
}

export async function createEvent(input: EventInput): Promise<Event> {
  const event = await invoke<Event>('create_event', { event: input });
  events.update(prev => [...prev, event].sort((a, b) => a.start_time.localeCompare(b.start_time)));
  return event;
}

export async function updateEvent(id: string, input: EventInput): Promise<Event> {
  const event = await invoke<Event>('update_event', { id, event: input });
  events.update(prev => prev.map(e => (e.id === id ? event : e)));
  return event;
}

export async function deleteEvent(id: string): Promise<void> {
  await invoke('delete_event', { id });
  events.update(prev => prev.filter(e => e.id !== id));
}

// ─── Task functions ───────────────────────────────────────────────────────────

export async function loadTasks(
  listId?: string | null,
  includeCompleted = false
): Promise<Task[]> {
  const result = await invoke<Task[]>('get_tasks', {
    listId: listId ?? null,
    includeCompleted,
  });
  tasks.set(result);
  return result;
}

export async function createTask(input: TaskInput): Promise<Task> {
  const task = await invoke<Task>('create_task', { task: input });
  tasks.update(prev => [...prev, task]);
  return task;
}

export async function updateTask(id: string, input: TaskInput): Promise<Task> {
  const task = await invoke<Task>('update_task', { id, task: input });
  tasks.update(prev => prev.map(t => (t.id === id ? task : t)));
  return task;
}

export async function deleteTask(id: string): Promise<void> {
  await invoke('delete_task', { id });
  tasks.update(prev => prev.filter(t => t.id !== id));
}

export async function toggleTask(id: string): Promise<Task> {
  const task = await invoke<Task>('toggle_task', { id });
  tasks.update(prev => prev.map(t => (t.id === id ? task : t)));
  return task;
}

export async function reorderTasks(taskIds: string[]): Promise<void> {
  await invoke('reorder_tasks', { taskIds });
  tasks.update(prev => {
    const map = new Map(prev.map(t => [t.id, t]));
    return taskIds
      .map((id, i) => {
        const t = map.get(id);
        return t ? { ...t, sort_order: i } : null;
      })
      .filter((t): t is Task => t !== null);
  });
}

// ─── Task list functions ──────────────────────────────────────────────────────

export async function loadTaskLists(): Promise<TaskList[]> {
  const result = await invoke<TaskList[]>('get_task_lists');
  taskLists.set(result);
  return result;
}

export async function createTaskList(name: string, color: string): Promise<TaskList> {
  const list = await invoke<TaskList>('create_task_list', { name, color });
  taskLists.update(prev => [...prev, list]);
  return list;
}

export async function updateTaskList(
  id: string,
  name: string,
  color: string
): Promise<TaskList> {
  const list = await invoke<TaskList>('update_task_list', { id, name, color });
  taskLists.update(prev => prev.map(l => (l.id === id ? list : l)));
  return list;
}

export async function deleteTaskList(id: string): Promise<void> {
  await invoke('delete_task_list', { id });
  taskLists.update(prev => prev.filter(l => l.id !== id));
}

// ─── Time block functions ─────────────────────────────────────────────────────

export async function loadTimeBlocks(date: string): Promise<TimeBlock[]> {
  const result = await invoke<TimeBlock[]>('get_time_blocks', { date });
  timeBlocks.set(result);
  return result;
}

export async function createTimeBlock(input: TimeBlockInput): Promise<TimeBlock> {
  const block = await invoke<TimeBlock>('create_time_block', { block: input });
  timeBlocks.update(prev =>
    [...prev, block].sort((a, b) => a.start_time.localeCompare(b.start_time))
  );
  return block;
}

export async function updateTimeBlock(
  id: string,
  input: TimeBlockInput
): Promise<TimeBlock> {
  const block = await invoke<TimeBlock>('update_time_block', { id, block: input });
  timeBlocks.update(prev => prev.map(b => (b.id === id ? block : b)));
  return block;
}

export async function deleteTimeBlock(id: string): Promise<void> {
  await invoke('delete_time_block', { id });
  timeBlocks.update(prev => prev.filter(b => b.id !== id));
}
