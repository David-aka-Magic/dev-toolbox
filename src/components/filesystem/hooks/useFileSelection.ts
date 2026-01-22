/**
 * File selection state and logic
 * Handles multi-select with Ctrl/Shift, keyboard navigation
 */

import { writable, derived, get } from 'svelte/store';

export interface FileSelectionState {
  selectedFiles: Set<string>;
  focusedIndex: number;
  lastSelectedIndex: number;
}

function createFileSelection() {
  const { subscribe, set, update } = writable<FileSelectionState>({
    selectedFiles: new Set<string>(),
    focusedIndex: -1,
    lastSelectedIndex: -1
  });

  return {
    subscribe,

    /**
     * Handle click on a file item
     */
    handleClick: (event: MouseEvent, index: number, fileName: string, files: any[]) => {
      const isCtrl = event.ctrlKey || event.metaKey;
      const isShift = event.shiftKey;

      update(state => {
        let newSelected: Set<string>;

        if (isShift && state.lastSelectedIndex !== -1) {
          // Range selection
          const start = Math.min(state.lastSelectedIndex, index);
          const end = Math.max(state.lastSelectedIndex, index);
          
          if (!isCtrl) {
            newSelected = new Set<string>();
          } else {
            newSelected = new Set(state.selectedFiles);
          }
          
          for (let i = start; i <= end; i++) {
            newSelected.add(files[i].name);
          }
          
          return {
            selectedFiles: newSelected,
            focusedIndex: index,
            lastSelectedIndex: state.lastSelectedIndex
          };
        } else if (isCtrl) {
          // Toggle selection - create new Set
          newSelected = new Set(state.selectedFiles);
          const wasSelected = newSelected.has(fileName);
          if (wasSelected) {
            newSelected.delete(fileName);
          } else {
            newSelected.add(fileName);
          }
          
          return {
            selectedFiles: newSelected,
            focusedIndex: index,
            lastSelectedIndex: wasSelected ? state.lastSelectedIndex : index
          };
        } else {
          // Single selection - create new Set
          return {
            selectedFiles: new Set([fileName]),
            focusedIndex: index,
            lastSelectedIndex: index
          };
        }
      });
    },

    /**
     * Handle keyboard navigation
     */
    handleKeyDown: (event: KeyboardEvent, files: any[], containerWidth: number = 800) => {
      const state = get({ subscribe });
      
      if (files.length === 0) return;
      
      const itemWidth = 100;
      const columns = Math.floor(containerWidth / itemWidth);
      let newFocusedIndex = state.focusedIndex;

      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
        event.preventDefault();
        
        if (state.focusedIndex === -1) {
          newFocusedIndex = 0;
        } else {
          if (event.key === 'ArrowRight') newFocusedIndex = Math.min(files.length - 1, state.focusedIndex + 1);
          if (event.key === 'ArrowLeft') newFocusedIndex = Math.max(0, state.focusedIndex - 1);
          if (event.key === 'ArrowDown') newFocusedIndex = Math.min(files.length - 1, state.focusedIndex + columns);
          if (event.key === 'ArrowUp') newFocusedIndex = Math.max(0, state.focusedIndex - columns);
        }

        update(state => {
          const newState = { ...state, focusedIndex: newFocusedIndex };
          
          if (!event.ctrlKey) {
            if (event.shiftKey) {
              newState.selectedFiles.add(files[newFocusedIndex].name);
            } else {
              newState.selectedFiles = new Set([files[newFocusedIndex].name]);
            }
          }
          
          return newState;
        });

        // Scroll into view
        const button = document.getElementById(`file-btn-${newFocusedIndex}`);
        button?.scrollIntoView({ block: 'nearest' });
      }
    },

    /**
     * Select all files
     */
    selectAll: (files: any[]) => {
      update(state => ({
        ...state,
        selectedFiles: new Set(files.map(f => f.name))
      }));
    },

    /**
     * Clear all selections
     */
    clearSelection: () => {
      console.log('clearSelection called! Stack:', new Error().stack);
      update(state => ({
        ...state,
        selectedFiles: new Set<string>(),
        focusedIndex: -1
      }));
    },

    /**
     * Select a specific file (used after creation/rename)
     */
    selectFile: (fileName: string, index: number) => {
      update(state => ({
        ...state,
        selectedFiles: new Set([fileName]),
        focusedIndex: index,
        lastSelectedIndex: index
      }));
    },

    /**
     * Add a file to selection without clearing others
     */
    addToSelection: (fileName: string, index: number) => {
      update(state => {
        const newSelected = new Set(state.selectedFiles);
        newSelected.add(fileName);
        return {
          ...state,
          selectedFiles: newSelected,
          focusedIndex: index,
          lastSelectedIndex: index
        };
      });
    },

    /**
     * Reset state when changing directories
     */
    reset: () => {
      set({
        selectedFiles: new Set<string>(),
        focusedIndex: -1,
        lastSelectedIndex: -1
      });
    }
  };
}

export const fileSelection = createFileSelection();

// Derived stores for convenience
export const selectedFiles = derived(fileSelection, $state => $state.selectedFiles);
export const focusedIndex = derived(fileSelection, $state => $state.focusedIndex);