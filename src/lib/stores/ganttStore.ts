import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ─── TypeScript interfaces ────────────────────────────────────────────────────

export interface GanttProject {
  id: string;
  name: string;
  description: string | null;
  start_date: string;
  color_scheme: string;
  zoom_level: string;
  created_at: string;
  updated_at: string;
}

export interface GanttProjectInput {
  name: string;
  description?: string | null;
  start_date: string;
  color_scheme?: string | null;
  zoom_level?: string | null;
}

export interface GanttProjectDetail {
  project: GanttProject;
  tasks: GanttTask[];
  dependencies: GanttDependency[];
  milestones: GanttMilestone[];
}

export interface GanttTask {
  id: string;
  project_id: string;
  title: string;
  start_date: string;
  end_date: string;
  progress: number;
  color: string;
  group_name: string | null;
  sort_order: number;
  collapsed: boolean;
  parent_task_id: string | null;
  assigned_to: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface GanttTaskInput {
  project_id: string;
  title: string;
  start_date: string;
  end_date: string;
  progress?: number | null;
  color?: string | null;
  group_name?: string | null;
  sort_order?: number | null;
  collapsed?: boolean | null;
  parent_task_id?: string | null;
  assigned_to?: string | null;
  notes?: string | null;
}

export interface GanttTaskUpdate {
  id: string;
  start_date?: string | null;
  end_date?: string | null;
  progress?: number | null;
}

export interface GanttDependency {
  id: string;
  project_id: string;
  from_task_id: string;
  to_task_id: string;
  dependency_type: string;
}

export interface GanttDependencyInput {
  project_id: string;
  from_task_id: string;
  to_task_id: string;
  dependency_type?: string | null;
}

export interface GanttMilestone {
  id: string;
  project_id: string;
  title: string;
  date: string;
  color: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface GanttMilestoneInput {
  project_id: string;
  title: string;
  date: string;
  color?: string | null;
  sort_order?: number | null;
}

// ─── Stores ───────────────────────────────────────────────────────────────────

export const projects = writable<GanttProject[]>([]);
export const currentProject = writable<GanttProject | null>(null);
export const tasks = writable<GanttTask[]>([]);
export const dependencies = writable<GanttDependency[]>([]);
export const milestones = writable<GanttMilestone[]>([]);
export const zoomLevel = writable<'days' | 'weeks' | 'months'>('weeks');
export const selectedTaskId = writable<string | null>(null);
export const viewportStartDate = writable<Date>(new Date());

// ─── Derived helpers ──────────────────────────────────────────────────────────

/** Returns direct children of the given parent task id. */
export const getTaskChildren = derived(tasks, ($tasks) => (parentId: string): GanttTask[] =>
  $tasks.filter(t => t.parent_task_id === parentId)
);

/** Groups tasks by their group_name. Tasks with no group are keyed as ''. */
export const getTasksByGroup = derived(tasks, ($tasks) => (): Record<string, GanttTask[]> => {
  const groups: Record<string, GanttTask[]> = {};
  for (const task of $tasks) {
    const key = task.group_name ?? '';
    if (!groups[key]) groups[key] = [];
    groups[key].push(task);
  }
  return groups;
});

/**
 * Returns { start: Date, end: Date } spanning all tasks in the current project.
 * Falls back to today ± 30 days when no tasks exist.
 */
export const getProjectDateRange = derived(
  [tasks, currentProject],
  ([$tasks, $currentProject]) => (): { start: Date; end: Date } => {
    if ($tasks.length === 0) {
      const anchor = $currentProject
        ? new Date($currentProject.start_date)
        : new Date();
      const end = new Date(anchor);
      end.setDate(end.getDate() + 30);
      return { start: anchor, end };
    }
    const dates = $tasks.flatMap(t => [new Date(t.start_date), new Date(t.end_date)]);
    return {
      start: new Date(Math.min(...dates.map(d => d.getTime()))),
      end:   new Date(Math.max(...dates.map(d => d.getTime()))),
    };
  }
);

// ─── Project functions ────────────────────────────────────────────────────────

export async function loadProjects(): Promise<GanttProject[]> {
  const result = await invoke<GanttProject[]>('get_gantt_projects');
  projects.set(result);
  return result;
}

export async function createProject(input: GanttProjectInput): Promise<GanttProject> {
  const project = await invoke<GanttProject>('create_gantt_project', { project: input });
  projects.update(prev => [...prev, project]);
  return project;
}

export async function updateProject(id: string, input: GanttProjectInput): Promise<GanttProject> {
  const project = await invoke<GanttProject>('update_gantt_project', { id, project: input });
  projects.update(prev => prev.map(p => (p.id === id ? project : p)));
  currentProject.update(cp => (cp?.id === id ? project : cp));
  return project;
}

export async function deleteProject(id: string): Promise<void> {
  await invoke('delete_gantt_project', { id });
  projects.update(prev => prev.filter(p => p.id !== id));
  currentProject.update(cp => (cp?.id === id ? null : cp));
}

/** Loads all tasks, dependencies, and milestones for a project in one call. */
export async function loadProjectData(projectId: string): Promise<GanttProjectDetail> {
  const detail = await invoke<GanttProjectDetail>('get_gantt_project', { id: projectId });
  currentProject.set(detail.project);
  tasks.set(detail.tasks);
  dependencies.set(detail.dependencies);
  milestones.set(detail.milestones);
  // Seed the viewport to the project's start date
  viewportStartDate.set(new Date(detail.project.start_date));
  // Mirror the project's stored zoom level if valid
  const z = detail.project.zoom_level;
  if (z === 'days' || z === 'weeks' || z === 'months') {
    zoomLevel.set(z);
  }
  return detail;
}

// ─── Task functions ───────────────────────────────────────────────────────────

export async function createTask(input: GanttTaskInput): Promise<GanttTask> {
  const task = await invoke<GanttTask>('create_gantt_task', { task: input });
  tasks.update(prev => [...prev, task]);
  return task;
}

export async function updateTask(id: string, input: GanttTaskInput): Promise<GanttTask> {
  const task = await invoke<GanttTask>('update_gantt_task', { id, task: input });
  tasks.update(prev => prev.map(t => (t.id === id ? task : t)));
  return task;
}

export async function deleteTask(id: string): Promise<void> {
  await invoke('delete_gantt_task', { id });
  tasks.update(prev => prev.filter(t => t.id !== id));
  selectedTaskId.update(sel => (sel === id ? null : sel));
}

export async function batchUpdateTasks(updates: GanttTaskUpdate[]): Promise<GanttTask[]> {
  const updated = await invoke<GanttTask[]>('batch_update_gantt_tasks', { updates });
  tasks.update(prev => {
    const map = new Map(updated.map(t => [t.id, t]));
    return prev.map(t => map.get(t.id) ?? t);
  });
  return updated;
}

// ─── Dependency functions ─────────────────────────────────────────────────────

export async function createDependency(input: GanttDependencyInput): Promise<GanttDependency> {
  const dep = await invoke<GanttDependency>('create_gantt_dependency', { dependency: input });
  dependencies.update(prev => [...prev, dep]);
  return dep;
}

export async function deleteDependency(id: string): Promise<void> {
  await invoke('delete_gantt_dependency', { id });
  dependencies.update(prev => prev.filter(d => d.id !== id));
}

export async function validateDependencies(projectId: string): Promise<string[]> {
  return invoke<string[]>('validate_dependencies', { projectId });
}

// ─── Milestone functions ──────────────────────────────────────────────────────

export async function createMilestone(input: GanttMilestoneInput): Promise<GanttMilestone> {
  const milestone = await invoke<GanttMilestone>('create_gantt_milestone', { milestone: input });
  milestones.update(prev => [...prev, milestone].sort((a, b) => a.date.localeCompare(b.date)));
  return milestone;
}

export async function updateMilestone(id: string, input: GanttMilestoneInput): Promise<GanttMilestone> {
  const milestone = await invoke<GanttMilestone>('update_gantt_milestone', { id, milestone: input });
  milestones.update(prev => prev.map(m => (m.id === id ? milestone : m)));
  return milestone;
}

export async function deleteMilestone(id: string): Promise<void> {
  await invoke('delete_gantt_milestone', { id });
  milestones.update(prev => prev.filter(m => m.id !== id));
}

// ─── Reorder tasks ────────────────────────────────────────────────────────────

export async function reorderTasks(taskIds: string[]): Promise<void> {
  await invoke('reorder_gantt_tasks', { taskIds });
  tasks.update(prev => {
    const orderMap = new Map(taskIds.map((id, i) => [id, i]));
    return [...prev]
      .map(t => ({ ...t, sort_order: orderMap.get(t.id) ?? t.sort_order }))
      .sort((a, b) => a.sort_order - b.sort_order);
  });
}
