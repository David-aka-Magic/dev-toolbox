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
  draggedFiles: string[]; // Array of all files being dragged
  draggedFilePaths: Map<string, string>; // Map of fileName -> filePath
  currentDropTarget: string | null;
}

function createFileDragDrop() {
  const { subscribe, set, update } = writable<DragState>({
    draggedFile: null,
    draggedFilePath: null,
    draggedIsDir: false,
    draggedFiles: [],
    draggedFilePaths: new Map(),
    currentDropTarget: null
  });

  return {
    subscribe,

    /**
     * Handle drag start - supports multiple files
     */
    handleDragStart: (event: DragEvent, file: any, selectedFiles?: Set<string>, allFiles?: any[]) => {
      if (!event.dataTransfer) return;

      let filesToDrag: any[] = [file];
      
      // If selectedFiles is provided and the dragged file is in selection, drag all
      if (selectedFiles && allFiles && selectedFiles.has(file.name) && selectedFiles.size > 1) {
        filesToDrag = allFiles.filter(f => selectedFiles.has(f.name));
        console.log(`🚀 DRAG START: ${filesToDrag.length} selected files`);
      } else {
        console.log("🚀 DRAG START:", file.name, "IsDir:", file.is_dir);
      }

      // Build the file paths map
      const pathsMap = new Map<string, string>();
      filesToDrag.forEach(f => {
        pathsMap.set(f.name, f.path);
      });

      set({
        draggedFile: file.name, // Keep for backward compatibility
        draggedFilePath: file.path,
        draggedIsDir: file.is_dir,
        draggedFiles: filesToDrag.map(f => f.name),
        draggedFilePaths: pathsMap,
        currentDropTarget: null
      });

      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/plain", filesToDrag.map(f => f.name).join(', '));

      // Create custom drag image
      const dragImg = document.createElement('div');
      dragImg.style.position = 'absolute';
      dragImg.style.top = '-1000px';
      dragImg.style.padding = '8px 12px';
      dragImg.style.background = 'rgba(59, 130, 246, 0.9)';
      dragImg.style.color = 'white';
      dragImg.style.borderRadius = '4px';
      dragImg.style.fontSize = '12px';
      
      if (filesToDrag.length === 1) {
        dragImg.textContent = `${file.is_dir ? '📁' : '📄'} ${file.name}`;
      } else {
        dragImg.textContent = `📦 ${filesToDrag.length} items`;
      }
      
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
        draggedFiles: [],
        draggedFilePaths: new Map(),
        currentDropTarget: null
      });
    },

    /**
     * Handle drag enter on a folder
     */
    handleItemDragEnter: (event: DragEvent, file: any) => {
      const state = get({ subscribe });
      
      console.log("👋 ITEM ENTER:", file.name);
      
      if (!state.draggedFile || !file.is_dir || state.draggedFiles.includes(file.name)) {
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
      
      if (!state.draggedFile || !file.is_dir || state.draggedFiles.includes(file.name)) {
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
     * Handle drop on a folder - supports multiple files
     */
    handleItemDrop: async (event: DragEvent, targetFolder: any, currentPath: string, onComplete: () => void) => {
      event.preventDefault();
      event.stopPropagation();

      const element = event.currentTarget as HTMLElement;
      element.classList.remove('drag-over');

      const state = get({ subscribe });

      if (!targetFolder.is_dir || !state.draggedFile) return;

      const destPath = targetFolder.path;

      console.log(`💧 DROP ${state.draggedFiles.length} item(s) on:`, targetFolder.name);

      if (!currentPath) return;

      // Move all files (or just one if draggedFiles has only one item)
      try {
        for (const fileName of state.draggedFiles) {
          const sourcePath = state.draggedFilePaths.get(fileName) || joinPath(currentPath, fileName);
          
          // Safety checks
          if (sourcePath === destPath) {
            console.log("⚠️ Cannot drop folder into itself");
            continue;
          }

          // Check if dropping a folder into its own subdirectory
          if (destPath.startsWith(sourcePath + '\\') || destPath.startsWith(sourcePath + '/')) {
            alert(`Cannot move "${fileName}" into its own subdirectory`);
            continue;
          }

          console.log(`📦 Moving: ${fileName} -> ${destPath}`);
          await invoke('move_item', { source: sourcePath, destination: destPath });
        }
        onComplete(); // Reload files after all moves complete
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