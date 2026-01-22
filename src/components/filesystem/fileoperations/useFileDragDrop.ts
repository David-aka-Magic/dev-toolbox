/**
 * File drag and drop state and handlers
 */

import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { joinPath } from './fileUtils';

export interface DragState {
  draggedFile: string | null;
  draggedFilePath: string | null;
  draggedIsDir: boolean;
  currentDropTarget: string | null;
}

function createFileDragDrop() {
  const { subscribe, set, update } = writable<DragState>({
    draggedFile: null,
    draggedFilePath: null,
    draggedIsDir: false,
    currentDropTarget: null
  });

  return {
    subscribe,

    /**
     * Handle drag start
     */
    handleDragStart: (event: DragEvent, file: any) => {
      if (!event.dataTransfer) return;

      console.log("🚀 DRAG START:", file.name, "IsDir:", file.is_dir);

      set({
        draggedFile: file.name,
        draggedFilePath: file.path,
        draggedIsDir: file.is_dir,
        currentDropTarget: null
      });

      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/plain", file.name);

      // Create custom drag image
      const dragImg = document.createElement('div');
      dragImg.style.position = 'absolute';
      dragImg.style.top = '-1000px';
      dragImg.style.padding = '8px 12px';
      dragImg.style.background = 'rgba(59, 130, 246, 0.9)';
      dragImg.style.color = 'white';
      dragImg.style.borderRadius = '4px';
      dragImg.style.fontSize = '12px';
      dragImg.textContent = `${file.is_dir ? '📁' : '📄'} ${file.name}`;
      document.body.appendChild(dragImg);
      event.dataTransfer.setDragImage(dragImg, 0, 0);
      setTimeout(() => document.body.removeChild(dragImg), 0);
    },

    /**
     * Handle drag end
     */
    handleDragEnd: () => {
      console.log("🏁 DRAG END");
      set({
        draggedFile: null,
        draggedFilePath: null,
        draggedIsDir: false,
        currentDropTarget: null
      });
    },

    /**
     * Handle drag enter on a folder
     */
    handleItemDragEnter: (event: DragEvent, file: any) => {
      const state = get({ subscribe });
      
      console.log("👋 ITEM ENTER:", file.name);
      
      if (!state.draggedFile || !file.is_dir || file.name === state.draggedFile) {
        return;
      }
      
      event.preventDefault();
      event.stopPropagation();
    },

    /**
     * Handle drag over a folder
     */
    handleItemDragOver: (event: DragEvent, file: any) => {
      const state = get({ subscribe });
      
      if (!state.draggedFile || !file.is_dir || file.name === state.draggedFile) {
        return;
      }

      console.log("🔄 ITEM OVER:", file.name);
      event.preventDefault();
      event.stopPropagation();

      const element = event.currentTarget as HTMLElement;
      element.classList.add('drag-over');

      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = "move";
      }
    },

    /**
     * Handle drag leave
     */
    handleItemDragLeave: (event: DragEvent) => {
      const element = event.currentTarget as HTMLElement;
      element.classList.remove('drag-over');
    },

    /**
     * Handle drop on a folder
     */
    handleItemDrop: async (event: DragEvent, targetFolder: any, currentPath: string, onComplete: () => void) => {
      event.preventDefault();
      event.stopPropagation();

      const element = event.currentTarget as HTMLElement;
      element.classList.remove('drag-over');

      const state = get({ subscribe });

      if (!targetFolder.is_dir || !state.draggedFile) return;

      console.log("💧 DROP on:", targetFolder.name);

      if (!currentPath) return;

      const sourcePath = state.draggedFilePath || joinPath(currentPath, state.draggedFile);
      const destPath = targetFolder.path;

      if (sourcePath === destPath) {
        console.log("⚠️ Cannot drop folder into itself");
        return;
      }

      if (state.draggedIsDir && destPath.startsWith(sourcePath)) {
        alert("Cannot move a folder into its own subdirectory");
        return;
      }

      console.log("📦 Moving:", sourcePath, "->", destPath);

      try {
        await invoke('move_item', { source: sourcePath, destination: destPath });
        onComplete(); // Reload files
      } catch (err) {
        console.error("❌ Move error:", err);
        alert("Move failed: " + err);
      }
    },

    /**
     * Check if currently dragging
     */
    isDragging: (): boolean => {
      const state = get({ subscribe });
      return state.draggedFile !== null;
    },

    /**
     * Get dragged file name
     */
    getDraggedFile: (): string | null => {
      const state = get({ subscribe });
      return state.draggedFile;
    }
  };
}

export const fileDragDrop = createFileDragDrop();