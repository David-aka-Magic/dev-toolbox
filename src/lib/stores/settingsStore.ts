import { writable } from 'svelte/store';

interface Settings {
  termFontSize: number;
  termCursorStyle: 'block' | 'underline' | 'bar';
  termShellPath: string;
  editorFontSize: number;
  // REMOVED: windowZoom
}

const defaultSettings: Settings = {
  termFontSize: 14,
  termCursorStyle: 'block',
  termShellPath: 'C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe',
  editorFontSize: 14,
  // REMOVED: windowZoom
};

// Load from localStorage or use defaults
const stored = localStorage.getItem('app-settings');
const initial = stored ? JSON.parse(stored) : defaultSettings;

export const settings = writable<Settings>(initial);
export const isSettingsOpen = writable(false);

settings.subscribe((val) => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('app-settings', JSON.stringify(val));
  }
});