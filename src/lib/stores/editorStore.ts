import { writable } from 'svelte/store';

export type EditorTab = {
  id: string;
  name: string;
  path: string | null;
  content: string;
  isDirty: boolean;
};

const initialTab: EditorTab = {
  id: crypto.randomUUID(),
  name: 'Untitled',
  path: null,
  content: '',
  isDirty: false
};

export const editorTabs = writable<EditorTab[]>([initialTab]);
export const activeEditorTabId = writable<string>(initialTab.id);