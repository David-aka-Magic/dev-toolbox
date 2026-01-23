/**
 * Clipboard store for file copy/cut/paste operations
 */

import { writable, get } from 'svelte/store';

export type ClipboardOperation = 'copy' | 'cut' | null;

export interface ClipboardState {
  files: string[];        // Array of file paths
  fileNames: string[];    // Array of file names (for display)
  operation: ClipboardOperation;
  sourcePath: string;     // The directory where files were copied/cut from
}

function createClipboardStore() {
  const { subscribe, set, update } = writable<ClipboardState>({
    files: [],
    fileNames: [],
    operation: null,
    sourcePath: ''
  });

  return {
    subscribe,

    /**
     * Copy files to clipboard
     */
    copy: (filePaths: string[], fileNames: string[], sourcePath: string) => {
      console.log('📋 COPY:', fileNames);
      set({
        files: filePaths,
        fileNames: fileNames,
        operation: 'copy',
        sourcePath
      });
    },

    /**
     * Cut files to clipboard
     */
    cut: (filePaths: string[], fileNames: string[], sourcePath: string) => {
      console.log('✂️ CUT:', fileNames);
      set({
        files: filePaths,
        fileNames: fileNames,
        operation: 'cut',
        sourcePath
      });
    },

    /**
     * Clear clipboard after paste (especially for cut)
     */
    clear: () => {
      set({
        files: [],
        fileNames: [],
        operation: null,
        sourcePath: ''
      });
    },

    /**
     * Check if clipboard has files
     */
    hasFiles: (): boolean => {
      const state = get({ subscribe });
      return state.files.length > 0;
    },

    /**
     * Get current state
     */
    getState: (): ClipboardState => {
      return get({ subscribe });
    }
  };
}

export const clipboard = createClipboardStore();