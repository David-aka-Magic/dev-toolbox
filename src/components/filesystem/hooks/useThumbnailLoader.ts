/**
 * Thumbnail loading queue system
 * Handles lazy loading of image/video thumbnails with concurrency limits
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

function createThumbnailLoader() {
  const { subscribe, set, update } = writable<ThumbnailState>({
    thumbnails: new Map<string, string>(),
    loadingSet: new Set<string>()
  });

  let thumbnailQueue: QueueItem[] = [];
  let activeLoads = 0;

  // Get max concurrent from settings
  function getMaxConcurrent(): number {
    const currentSettings = get(settings);
    return currentSettings.fileMaxConcurrentThumbnails || 5;
  }

  // Get thumbnail size from settings
  function getThumbnailSize(): number {
    const currentSettings = get(settings);
    return currentSettings.fileThumbnailSize || 48;
  }

  /**
   * Process the thumbnail queue
   */
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
      const thumbnailSize = getThumbnailSize();

      if (isVideoFile(fileName)) {
        // Extract video thumbnail with configured size
        base64 = await invoke<string>('extract_video_thumbnail', { 
          path: filePath,
          size: thumbnailSize
        });
        mimeType = 'image/png';
      } else {
        // Load image directly (could add resizing here if needed)
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
    subscribe,

    /**
     * Queue files for thumbnail loading
     */
    queueThumbnails: (files: any[]) => {
      const mediaFiles = files.filter(f => 
        !f.is_dir && (isImageFile(f.name) || isVideoFile(f.name))
      );

      thumbnailQueue = mediaFiles.map(f => ({
        path: f.path,
        name: f.name
      }));

      processQueue();
    },

    /**
     * Clear all thumbnails (when changing directory)
     */
    clearThumbnails: () => {
      thumbnailQueue = [];
      set({
        thumbnails: new Map<string, string>(),
        loadingSet: new Set<string>()
      });
    },

    /**
     * Get current thumbnail size setting
     */
    getThumbnailSize
  };
}

export const thumbnailLoader = createThumbnailLoader();

// Derived store that just exposes the thumbnails Map for backward compatibility
// Usage: $thumbnails.get(filename) or $thumbnails.has(filename)
export const thumbnails = derived(thumbnailLoader, $state => $state.thumbnails);