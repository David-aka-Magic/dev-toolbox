import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { joinPath } from './fileUtils';

export interface DragState {
  draggedFile: string | null;
  draggedFilePath: string | null;
  draggedIsDir: boolean;
  draggedFiles: string[];
  draggedFilePaths: Map<string, string>;
  currentDropTarget: string | null;
}

const dragStore = writable<DragState>({
  draggedFile: null,
  draggedFilePath: null,
  draggedIsDir: false,
  draggedFiles: [],
  draggedFilePaths: new Map(),
  currentDropTarget: null
});

function createFileDragDrop() {
  const { subscribe, set, update } = dragStore;

  return {
    subscribe,

    handleDragStart: (event: DragEvent, file: any, selectedFiles?: Set<string>, allFiles?: any[]) => {
      if (!event.dataTransfer) return;

      let filesToDrag: any[] = [file];
      
      if (selectedFiles && allFiles && selectedFiles.has(file.name) && selectedFiles.size > 1) {
        filesToDrag = allFiles.filter(f => selectedFiles.has(f.name));
        console.log(`🚀 DRAG START: ${filesToDrag.length} selected files`);
      } else {
        console.log("🚀 DRAG START:", file.name, "IsDir:", file.is_dir);
      }

      const pathsMap = new Map<string, string>();
      filesToDrag.forEach(f => {
        pathsMap.set(f.name, f.path);
      });

      set({
        draggedFile: file.name,
        draggedFilePath: file.path,
        draggedIsDir: file.is_dir,
        draggedFiles: filesToDrag.map(f => f.name),
        draggedFilePaths: pathsMap,
        currentDropTarget: null
      });

      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/plain", filesToDrag.map(f => f.name).join(', '));

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

    handleItemDragEnter: (event: DragEvent, file: any) => {
      const state = get(dragStore);
      
      console.log("👋 ITEM ENTER:", file.name);
      
      if (!state.draggedFile || !file.is_dir || state.draggedFiles.includes(file.name)) {
        return;
      }
      
      event.preventDefault();
      event.stopPropagation();
    },

    handleItemDragOver: (event: DragEvent, file: any) => {
      const state = get(dragStore);
      
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

    handleItemDragLeave: (event: DragEvent) => {
      const element = event.currentTarget as HTMLElement;
      element.classList.remove('drag-over');
    },

    handleItemDrop: async (event: DragEvent, targetFolder: any, currentPath: string, onComplete: () => void) => {
      event.preventDefault();
      event.stopPropagation();

      const element = event.currentTarget as HTMLElement;
      element.classList.remove('drag-over');

      const state = get(dragStore);

      if (!targetFolder.is_dir || !state.draggedFile) return;

      const destPath = targetFolder.path;

      console.log(`💧 DROP ${state.draggedFiles.length} item(s) on:`, targetFolder.name);

      if (!currentPath) return;

      try {
        for (const fileName of state.draggedFiles) {
          const sourcePath = `${currentPath}\\${fileName}`;
          
          if (sourcePath === destPath) {
            console.log("⚠️ Cannot drop folder into itself");
            continue;
          }

          if (destPath.startsWith(sourcePath + '\\') || destPath.startsWith(sourcePath + '/')) {
            alert(`Cannot move "${fileName}" into its own subdirectory`);
            continue;
          }

          console.log(`📦 Moving: ${fileName} -> ${destPath}`);
          await invoke('move_item', { src: sourcePath, dest: destPath });
        }
        onComplete();
      } catch (err) {
        console.error("❌ Move error:", err);
        alert("Move failed: " + err);
      }
    },

    isDragging: (): boolean => {
      const state = get(dragStore);
      return state.draggedFile !== null;
    },

    getDraggedFile: (): string | null => {
      const state = get(dragStore);
      return state.draggedFile;
    }
  };
}

export const fileDragDrop = createFileDragDrop();