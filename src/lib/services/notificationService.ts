import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { get } from 'svelte/store';
import { events, tasks } from '$lib/stores/plannerStore';
import { settings } from '$lib/stores/settingsStore';
import type { Event, Task } from '$lib/stores/plannerStore';

let intervalId: ReturnType<typeof setInterval> | null = null;
const notifiedIds = new Set<string>();
let lastResetDate = todayStr();

function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

function fmtTime12(isoOrTime: string): string {
  let h: number, m: number;
  if (isoOrTime.includes('T')) {
    const t = isoOrTime.split('T')[1].slice(0, 5);
    [h, m] = t.split(':').map(Number);
  } else {
    [h, m] = isoOrTime.split(':').map(Number);
  }
  const ampm = h < 12 ? 'AM' : 'PM';
  const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h;
  return `${h12}:${String(m).padStart(2, '0')} ${ampm}`;
}

async function ensurePermission(): Promise<boolean> {
  let granted = await isPermissionGranted();
  if (!granted) {
    const result = await requestPermission();
    granted = result === 'granted';
  }
  return granted;
}

function checkAndNotify() {
  const s = get(settings);
  if (!s.plannerNotificationsEnabled) return;

  const now = new Date();
  const today = todayStr();

  // Reset notified set at midnight
  if (today !== lastResetDate) {
    notifiedIds.clear();
    lastResetDate = today;
  }

  const windowMs = s.plannerNotifyMinutesBefore * 60 * 1000;
  const nowMs = now.getTime();

  // Check events
  const allEvents = get(events);
  for (const evt of allEvents) {
    if (notifiedIds.has(evt.id)) continue;
    if (evt.all_day) continue;

    const startMs = new Date(evt.start_time).getTime();
    const diff = startMs - nowMs;

    // Notify if within the window (including events already started up to 1 min ago)
    if (diff <= windowMs && diff > -60_000) {
      notifiedIds.add(evt.id);
      sendNotification({
        title: 'Upcoming Event',
        body: `${evt.title} starts at ${fmtTime12(evt.start_time)}`,
      });
    }
  }

  // Check tasks with due_time
  const allTasks = get(tasks);
  for (const task of allTasks) {
    if (notifiedIds.has(task.id)) continue;
    if (task.completed) continue;
    if (!task.due_date || !task.due_time) continue;
    if (task.due_date !== today) continue;

    const [h, m] = task.due_time.split(':').map(Number);
    const taskDate = new Date(now.getFullYear(), now.getMonth(), now.getDate(), h, m);
    const diff = taskDate.getTime() - nowMs;

    if (diff <= windowMs && diff > -60_000) {
      notifiedIds.add(task.id);
      sendNotification({
        title: 'Upcoming Task',
        body: `${task.title} is due at ${fmtTime12(task.due_time)}`,
      });
    }
  }
}

export async function startNotificationScheduler() {
  const granted = await ensurePermission();
  if (!granted) return;

  // Run immediately to catch events happening right now
  checkAndNotify();

  intervalId = setInterval(checkAndNotify, 30_000);
}

export function stopNotificationScheduler() {
  if (intervalId !== null) {
    clearInterval(intervalId);
    intervalId = null;
  }
}
