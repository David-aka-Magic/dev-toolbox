import { get } from 'svelte/store';
import { selectedFiles, fileSelection } from './useFileSelection';
import { joinPath } from './fileUtils';
import { directoryCache } from '$lib/stores/directoryCacheStore';
import { invoke } from '@tauri-apps/api/core';
import { copySelectedFiles, cutSelectedFiles, pasteFiles } from './useClipboardActions';
import { deleteFiles } from './useFileActions';

export interface KeyboardContext {
  renamingFile: string | null;
  creationType: 'folder' | 'file' | null;
  currentPath: string | undefined;
  sortedFiles: any[];
  fileGridRef: any;
  confirmDeleteSetting: boolean;
  setRenamingFile: (name: string | null) => void;
  setCreationType: (type: 'folder' | 'file' | null) => void;
  loadFiles: (path: string, target: string | null, force: boolean) => Promise<void>;
  goBack: () => void;
}

export function handleKeyDown(event: KeyboardEvent, ctx: KeyboardContext) {
  if (ctx.renamingFile || ctx.creationType) {
    if (event.key === 'Escape') {
      ctx.setRenamingFile(null);
      ctx.setCreationType(null);
    }
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key === 'c') {
    event.preventDefault();
    copySelectedFiles(ctx.currentPath);
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key === 'x') {
    event.preventDefault();
    cutSelectedFiles(ctx.currentPath);
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key === 'v') {
    event.preventDefault();
    pasteFiles(ctx.currentPath, undefined, ctx.loadFiles);
    return;
  }

  if (event.key === 'F2') {
    event.preventDefault();
    const selected = get(selectedFiles);
    if (selected.size === 1) {
      ctx.setRenamingFile(Array.from(selected)[0]);
    }
    return;
  }

  if (event.key === 'F5') {
    event.preventDefault();
    if (ctx.currentPath) {
      directoryCache.invalidate(ctx.currentPath);
      ctx.loadFiles(ctx.currentPath, null, true);
    }
    return;
  }

  if (ctx.sortedFiles.length === 0) return;

  if (event.key === 'Delete') {
    event.preventDefault();
    const selected = get(selectedFiles);
    if (!ctx.currentPath || selected.size === 0) return;

    const filesToDelete = Array.from(selected);
    deleteFiles(filesToDelete, ctx.currentPath, ctx.confirmDeleteSetting, ctx.loadFiles);
    return;
  }

  if (event.key === 'Backspace') {
    ctx.goBack();
    return;
  }

  if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
    const containerWidth = ctx.fileGridRef?.getContainerWidth() || 800;
    fileSelection.handleKeyDown(event, ctx.sortedFiles, containerWidth);
  }

  if (event.key === 'a' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    fileSelection.selectAll(ctx.sortedFiles);
  }
}