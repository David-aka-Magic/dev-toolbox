import { writable } from 'svelte/store';

export interface RecentFile {
  path: string;
  name: string;
  lastOpened: number;
}

const MAX_RECENT_FILES = 20;

function createRecentFilesStore() {
  const stored = typeof localStorage !== 'undefined' 
    ? localStorage.getItem('recent-files') 
    : null;
  
  const initial: RecentFile[] = stored ? JSON.parse(stored) : [];
  const { subscribe, update, set } = writable<RecentFile[]>(initial);

  subscribe(val => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('recent-files', JSON.stringify(val));
    }
  });

  return {
    subscribe,
    
    addFile: (path: string, name: string) => {
      update(files => {
        const existing = files.filter(f => f.path !== path);
        const newFile: RecentFile = {
          path,
          name,
          lastOpened: Date.now()
        };
        
        const updated = [newFile, ...existing].slice(0, MAX_RECENT_FILES);
        return updated;
      });
    },
    
    removeFile: (path: string) => {
      update(files => files.filter(f => f.path !== path));
    },
    
    clear: () => {
      set([]);
    }
  };
}

export const recentFiles = createRecentFilesStore();