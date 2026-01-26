/**
 * Thumbnail loading queue system
 * Handles lazy loading of image/video thumbnails with concurrency limits
 * OPTIMIZED: Added global cache that persists across directory changes
 */

import { writable, get, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { isImageFile, isVideoFile, getImageMimeType } from './fileUtils';
import { settings } from '$lib/stores/settingsStore';

export interface ThumbnailState {
  thumbnails: Map<string, string>;
  loadingSet: Set<string>;
}

interface QueueItem {
  path: string;
  name: string;
}

// OPTIMIZATION: Global cache that persists across directory changes
const globalThumbnailCache = new Map<string, string>();
const MAX_GLOBAL_CACHE_SIZE = 500;

function createThumbnailLoader() {
  const { subscribe, set, update } = writable<ThumbnailState>({
    thumbnails: new Map<string, string>(),
    loadingSet: new Set<string>()
  });

  let thumbnailQueue: QueueItem[] = [];
  let activeLoads = 0;

  function getMaxConcurrent(): number {
    const currentSettings = get(settings);
    return currentSettings.fileMaxConcurrentThumbnails || 5;
  }

  function getThumbnailSize(): number {
    const currentSettings = get(settings);
    return currentSettings.fileThumbnailSize || 48;
  }

  async function processQueue() {
    const maxConcurrent = getMaxConcurrent();
    
    while (thumbnailQueue.length > 0 && activeLoads < maxConcurrent) {
      const item = thumbnailQueue.shift();
      if (item) {
        activeLoads++;
        loadThumbnail(item.path, item.name);
      }
    }
  }

  async function loadThumbnail(filePath: string, fileName: string) {
    const state = get({ subscribe });
    
    if (state.loadingSet.has(fileName)) {
      activeLoads--;
      processQueue();
      return;
    }

    // OPTIMIZATION: Check global cache first
    if (globalThumbnailCache.has(filePath)) {
      update(state => {
        const newThumbnails = new Map(state.thumbnails);
        newThumbnails.set(fileName, globalThumbnailCache.get(filePath)!);
        return { ...state, thumbnails: newThumbnails };
      });
      activeLoads--;
      processQueue();
      return;
    }

    // Mark as loading
    update(state => ({
      ...state,
      loadingSet: new Set([...state.loadingSet, fileName])
    }));

    try {
      let base64: string;
      let mimeType: string;

      if (isVideoFile(fileName)) {
        // Extract video thumbnail - Rust function only takes path
        base64 = await invoke<string>('extract_video_thumbnail', { path: filePath });
        mimeType = 'image/png';
      } else {
        // Load image directly
        base64 = await invoke<string>('read_file_base64', { path: filePath });
        mimeType = getImageMimeType(fileName);
      }

      const dataUrl = `data:${mimeType};base64,${base64}`;

      // OPTIMIZATION: Add to global cache
      globalThumbnailCache.set(filePath, dataUrl);
      
      // Limit global cache size
      if (globalThumbnailCache.size > MAX_GLOBAL_CACHE_SIZE) {
        const firstKey = globalThumbnailCache.keys().next().value;
        if (firstKey) globalThumbnailCache.delete(firstKey);
      }

      // Add to current view thumbnails
      update(state => {
        const newThumbnails = new Map(state.thumbnails);
        newThumbnails.set(fileName, dataUrl);
        
        const newLoadingSet = new Set(state.loadingSet);
        newLoadingSet.delete(fileName);

        return {
          thumbnails: newThumbnails,
          loadingSet: newLoadingSet
        };
      });
    } catch (err) {
      console.error(`Failed to load thumbnail for ${fileName}:`, err);
      
      update(state => {
        const newLoadingSet = new Set(state.loadingSet);
        newLoadingSet.delete(fileName);
        return { ...state, loadingSet: newLoadingSet };
      });
    } finally {
      activeLoads--;
      processQueue();
    }
  }

  return {
    subscribe,

    queueThumbnails: (files: any[]) => {
      thumbnailQueue = [];
      
      // OPTIMIZATION: Restore from global cache, queue the rest
      const restoredThumbnails = new Map<string, string>();
      const filesToLoad: QueueItem[] = [];
      
      for (const file of files) {
        if (!file.is_dir && (isImageFile(file.name) || isVideoFile(file.name))) {
          if (globalThumbnailCache.has(file.path)) {
            restoredThumbnails.set(file.name, globalThumbnailCache.get(file.path)!);
          } else {
            filesToLoad.push({ path: file.path, name: file.name });
          }
        }
      }

      // Set restored thumbnails immediately
      set({
        thumbnails: restoredThumbnails,
        loadingSet: new Set<string>()
      });

      // Queue remaining files
      thumbnailQueue = filesToLoad;
      processQueue();
    },

    clearThumbnails: () => {
      thumbnailQueue = [];
      activeLoads = 0;
      set({
        thumbnails: new Map<string, string>(),
        loadingSet: new Set<string>()
      });
    },

    clearGlobalCache: () => {
      globalThumbnailCache.clear();
    },

    getThumbnailSize,

    getGlobalCacheSize: (): number => {
      return globalThumbnailCache.size;
    }
  };
}

export const thumbnailLoader = createThumbnailLoader();

// Derived store for backward compatibility
export const thumbnails = derived(thumbnailLoader, $state => $state.thumbnails);