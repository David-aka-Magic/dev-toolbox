import { writable } from 'svelte/store';
import type { ViewMode, SortField, SortDirection } from './viewModeStore';

export interface NavItem {
  id: string;
  label: string;
  icon: string;
  visible: boolean;
}

interface Settings {
  // General / Appearance
  globalFontFamily: string;

  // Terminal Settings
  termFontSize: number;
  termCursorStyle: 'block' | 'underline' | 'bar';
  termShellPath: string;
  
  // Editor Settings
  editorFontSize: number;
  editorFontFamily: string;
  editorTabSize: string;
  editorWordWrap: 'off' | 'on' | 'wordWrapColumn';
  editorShowLineNumbers: boolean;
  editorShowMinimap: boolean;
  editorAutoSave: boolean;
  editorAutoSaveInterval: number;
  
  // File Manager Settings
  fileShowHidden: boolean;
  fileConfirmDelete: boolean;
  fileDefaultView: ViewMode;
  fileDefaultSortField: SortField;
  fileDefaultSortDirection: SortDirection;
  fileShowExtensions: boolean;
  fileDoubleClickAction: 'open' | 'preview' | 'editor';
  fileIconTheme: 'material' | 'minimal' | 'none';
  fileGridIconSize: number;
  fileShowFolderSize: boolean;
  fileFolderSizeThreshold: number;
  fileDefaultStartPath: string;
  fileRememberLastPath: boolean;
  fileLastPath: string;
  
  // Thumbnails & Previews
  fileThumbnailSize: number;
  fileMaxConcurrentThumbnails: number;
  fileEnableVideoPreview: boolean;
  fileVideoPreviewDuration: number;
  fileVideoPreviewResolution: number;
  fileUseHardwareAccel: boolean;
  fileThumbnailCacheSize: number;

  // API Tester Settings
  apiDefaultTimeout: number;
  apiFollowRedirects: boolean;
  apiValidateSSL: boolean;
  apiDefaultContentType: 'application/json' | 'text/plain' | 'application/x-www-form-urlencoded';
  apiMaxHistoryItems: number;
  apiAutoFormatJson: boolean;
  apiSaveToHistory: boolean;

  // Media Player Settings
  mediaScreenshotPath: string;
  mediaDefaultVolume: number;

  // Gantt Chart Settings
  ganttDefaultZoom: 'days' | 'weeks' | 'months';
  ganttShowWeekends: boolean;
  ganttDefaultTaskColor: string;
  ganttDefaultTaskDuration: number;
  ganttShowProgressBars: boolean;
  ganttSnapToGrid: boolean;
  ganttRowHeight: 24 | 32 | 36 | 40;

  // Toolbar / Navigation
  navItems: NavItem[];
}

const defaultSettings: Settings = {
  // General / Appearance
  globalFontFamily: 'Segoe UI',

  // Terminal
  termFontSize: 14,
  termCursorStyle: 'block',
  termShellPath: 'C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe',
  
  // Editor
  editorFontSize: 14,
  editorFontFamily: "'Fira Code', Consolas, monospace",
  editorTabSize: '2',
  editorWordWrap: 'off',
  editorShowLineNumbers: true,
  editorShowMinimap: true,
  editorAutoSave: true,
  editorAutoSaveInterval: 3,
  
  // File Manager
  fileShowHidden: false,
  fileConfirmDelete: true,
  fileDefaultView: 'grid',
  fileDefaultSortField: 'name',
  fileDefaultSortDirection: 'asc',
  fileShowExtensions: true,
  fileDoubleClickAction: 'open',
  fileIconTheme: 'material',
  fileGridIconSize: 48,
  fileShowFolderSize: false,
  fileFolderSizeThreshold: 1000,
  fileDefaultStartPath: 'C:\\',
  fileRememberLastPath: true,
  fileLastPath: 'C:\\',
  
  // Thumbnails & Previews
  fileThumbnailSize: 48,
  fileMaxConcurrentThumbnails: 5,
  fileEnableVideoPreview: true,
  fileVideoPreviewDuration: 3,
  fileVideoPreviewResolution: 160,
  fileUseHardwareAccel: true,
  fileThumbnailCacheSize: 500,

  // API Tester
  apiDefaultTimeout: 30000,
  apiFollowRedirects: true,
  apiValidateSSL: true,
  apiDefaultContentType: 'application/json',
  apiMaxHistoryItems: 50,
  apiAutoFormatJson: true,
  apiSaveToHistory: true,

  // Media Player
  mediaScreenshotPath: '',
  mediaDefaultVolume: 1.0,

  // Gantt Chart
  ganttDefaultZoom: 'weeks',
  ganttShowWeekends: true,
  ganttDefaultTaskColor: '#3b82f6',
  ganttDefaultTaskDuration: 7,
  ganttShowProgressBars: true,
  ganttSnapToGrid: true,
  ganttRowHeight: 36,

  // Toolbar / Navigation
  navItems: [
    { id: 'terminal', label: 'Terminal', icon: 'terminal', visible: true },
    { id: 'files', label: 'File Manager', icon: 'folder', visible: true },
    { id: 'editor', label: 'Editor', icon: 'code', visible: true },
    { id: 'planner', label: 'Planner', icon: 'calendar', visible: true },
    { id: 'gantt', label: 'Gantt Chart', icon: 'gantt', visible: true },
    { id: 'api', label: 'API Tester', icon: 'api', visible: true },
  ],
};

const stored = localStorage.getItem('app-settings');
let initial = stored ? { ...defaultSettings, ...JSON.parse(stored) } : defaultSettings;

// Ensure any nav items added since the user's last session are present.
if (stored) {
  const knownIds = new Set(initial.navItems.map((it: NavItem) => it.id));
  const missing = defaultSettings.navItems.filter(it => !knownIds.has(it.id));
  if (missing.length > 0) {
    initial = { ...initial, navItems: [...initial.navItems, ...missing] };
  }
}

export const settings = writable<Settings>(initial);
export const isSettingsOpen = writable(false);

settings.subscribe((val) => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('app-settings', JSON.stringify(val));
  }
});

export function updateLastPath(path: string) {
  settings.update(s => ({ ...s, fileLastPath: path }));
}