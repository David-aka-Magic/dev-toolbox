import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { clipboard } from '$lib/stores/clipboardStore';
import { directoryCache } from '$lib/stores/directoryCacheStore';
import { selectedFiles } from './useFileSelection';
import { joinPath } from './fileUtils';

export function copySelectedFiles(currentPath: string | undefined) {
  const selected = get(selectedFiles);
  if (!currentPath || selected.size === 0) return;

  const fileNames = Array.from(selected);
  const filePaths = fileNames.map(name => joinPath(currentPath, name));
  clipboard.copy(filePaths, fileNames, currentPath);
}

export function cutSelectedFiles(currentPath: string | undefined) {
  const selected = get(selectedFiles);
  if (!currentPath || selected.size === 0) return;

  const fileNames = Array.from(selected);
  const filePaths = fileNames.map(name => joinPath(currentPath, name));
  clipboard.cut(filePaths, fileNames, currentPath);
}

let isPasting = false;

export async function pasteFiles(
  currentPath: string | undefined,
  targetPath?: string,
  reloadFn?: (path: string, target: string | null, force: boolean) => Promise<void>
) {
  const destinationPath = targetPath || currentPath;
  if (!destinationPath) return;

  const clipboardState = clipboard.getState();
  if (clipboardState.files.length === 0) return;
  if (isPasting) return;

  isPasting = true;

  try {
    const destFiles = await invoke<any[]>('read_directory', { path: destinationPath });

    for (let i = 0; i < clipboardState.files.length; i++) {
      const sourcePath = clipboardState.files[i];
      const fileName = clipboardState.fileNames[i];
      const existsInDest = destFiles.some(f => f.name === fileName);

      if (existsInDest && clipboardState.operation === 'copy') {
        let counter = 1;
        let newName = fileName;
        const ext = fileName.includes('.') ? '.' + fileName.split('.').pop() : '';
        const baseName = ext ? fileName.slice(0, -ext.length) : fileName;

        const existingNames = new Set(destFiles.map(f => f.name));
        while (existingNames.has(newName)) {
          newName = `${baseName} (${counter})${ext}`;
          counter++;
        }

        await invoke('copy_item', { src: sourcePath, dest: destinationPath, newName });
      } else if (clipboardState.operation === 'copy') {
        await invoke('copy_item', { src: sourcePath, dest: destinationPath });
      } else if (clipboardState.operation === 'cut') {
        await invoke('move_item', { src: sourcePath, dest: destinationPath });
      }
    }

    if (clipboardState.operation === 'cut') {
      clipboard.clear();
      if (clipboardState.sourcePath) {
        directoryCache.invalidate(clipboardState.sourcePath);
      }
    }

    directoryCache.invalidate(destinationPath);
    if (currentPath) {
      directoryCache.invalidate(currentPath);
      reloadFn?.(currentPath, null, true);
    }
  } catch (err) {
    console.error('Paste error:', err);
    alert('Paste failed: ' + err);
  } finally {
    isPasting = false;
  }
}