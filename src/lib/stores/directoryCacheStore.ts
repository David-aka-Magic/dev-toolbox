import { writable, get } from 'svelte/store';

interface CacheEntry {
  files: any[];
  timestamp: number;
}

const CACHE_TTL = 30000; // 30 seconds
const MAX_CACHE_SIZE = 20;

function createDirectoryCache() {
  const { subscribe, set, update } = writable<Map<string, CacheEntry>>(new Map());

  return {
    subscribe,
    
    get: (path: string): any[] | null => {
      const cache = get({ subscribe });
      const entry = cache.get(path);
      if (entry && Date.now() - entry.timestamp < CACHE_TTL) {
        return entry.files;
      }
      return null;
    },

    set: (path: string, files: any[]) => {
      update(cache => {
        cache.set(path, { files, timestamp: Date.now() });
        
        if (cache.size > MAX_CACHE_SIZE) {
          const oldest = [...cache.entries()].sort((a, b) => a[1].timestamp - b[1].timestamp)[0];
          cache.delete(oldest[0]);
        }
        return cache;
      });
    },

    invalidate: (path: string) => {
      update(cache => {
        cache.delete(path);
        return cache;
      });
    },

    invalidateAll: () => {
      set(new Map());
    },

    clear: () => set(new Map())
  };
}

export const directoryCache = createDirectoryCache();