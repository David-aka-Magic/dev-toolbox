/**
 * Thumbnail loading queue system
 * Handles lazy loading of image/video thumbnails with concurrency limits
 */

import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { isImageFile, isVideoFile, getImageMimeType } from './fileUtils';

export interface ThumbnailState {
  thumbnails: Map<string, string>;
  loadingSet: Set<string>;
}

interface QueueItem {
  path: string;
  name: string;
}

const MAX_CONCURRENT_LOADS = 5;

function createThumbnailLoader() {
  const { subscribe, set, update } = writable<ThumbnailState>({
    thumbnails: new Map<string, string>(),
    loadingSet: new Set<string>()
  });

  let thumbnailQueue: QueueItem[] = [];
  let activeLoads = 0;

  /**
   * Process the thumbnail queue
   */
  async function processQueue() {
    while (thumbnailQueue.length > 0 && activeLoads < MAX_CONCURRENT_LOADS) {
      const item = thumbnailQueue.shift();
      if (item) {
        activeLoads++;
        loadThumbnail(item.path, item.name);
      }
    }
  }

  /**
   * Load a single thumbnail
   */
  async function loadThumbnail(filePath: string, fileName: string) {
    const state = get({ subscribe });
    
    if (state.loadingSet.has(fileName)) {
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
        // Extract video thumbnail
        base64 = await invoke<string>('extract_video_thumbnail', { path: filePath });
        mimeType = 'image/png';
      } else {
        // Load image directly
        base64 = await invoke<string>('read_file_base64', { path: filePath });
        mimeType = getImageMimeType(fileName);
      }

      // Add to thumbnails map
      update(state => {
        const newThumbnails = new Map(state.thumbnails);
        newThumbnails.set(fileName, `data:${mimeType};base64,${base64}`);
        
        const newLoadingSet = new Set(state.loadingSet);
        newLoadingSet.delete(fileName);

        return {
          thumbnails: newThumbnails,
          loadingSet: newLoadingSet
        };
      });
    } catch (err) {
      console.error(`Failed to load thumbnail for ${fileName}:`, err);
      
      // Remove from loading set even on error
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
    // IMPORTANT: Expose subscribe so $thumbnailLoader works in Svelte
    subscribe,

    /**
     * Queue thumbnails for a list of files
     */
    queueThumbnails: (files: any[]) => {
      thumbnailQueue = [];
      
      for (const file of files) {
        if (!file.is_dir) {
          if (isImageFile(file.name) || isVideoFile(file.name)) {
            thumbnailQueue.push({ path: file.path, name: file.name });
          }
        }
      }

      processQueue();
    },

    /**
     * Clear all thumbnails (when changing directories)
     */
    clearThumbnails: () => {
      set({
        thumbnails: new Map<string, string>(),
        loadingSet: new Set<string>()
      });
      thumbnailQueue = [];
      activeLoads = 0;
    },

    /**
     * Get thumbnail for a specific file
     */
    getThumbnail: (fileName: string): string | undefined => {
      const state = get({ subscribe });
      return state.thumbnails.get(fileName);
    },

    /**
     * Check if a thumbnail is currently loading
     */
    isLoading: (fileName: string): boolean => {
      const state = get({ subscribe });
      return state.loadingSet.has(fileName);
    }
  };
}

export const thumbnailLoader = createThumbnailLoader();

// Also export a derived store for just the thumbnails map (for simpler usage)
export const thumbnails = {
  subscribe: (fn: (value: Map<string, string>) => void) => {
    return thumbnailLoader.subscribe(state => fn(state.thumbnails));
  }
};