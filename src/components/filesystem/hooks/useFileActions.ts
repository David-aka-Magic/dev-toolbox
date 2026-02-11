import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
import { currentView } from '$lib/stores/viewStore';
import { isImageFile, isVideoFile, joinPath } from './fileUtils';
import { openMediaInNewWindow } from '$lib/utils/openMediaWindow';
import { directoryCache } from '$lib/stores/directoryCacheStore';

export function openFile(file: any) {
  if (isImageFile(file.name)) {
    openMediaInNewWindow(file.path, file.name, 'image');
  } else if (isVideoFile(file.name)) {
    openMediaInNewWindow(file.path, file.name, 'video');
  } else {
    openInEditor(file);
  }
}

export function openInPreview(file: any) {
  if (isImageFile(file.name)) {
    openMediaInNewWindow(file.path, file.name, 'image');
  } else if (isVideoFile(file.name)) {
    openMediaInNewWindow(file.path, file.name, 'video');
  } else {
    openInEditor(file);
  }
}

export async function openInEditor(file: any) {
  try {
    const content = await invoke<string>('read_file', { path: file.path });

    const newTab = {
      id: crypto.randomUUID(),
      name: file.name,
      path: file.path,
      content: `<p>${content.replace(/\n/g, '</p><p>')}</p>`,
      isDirty: false
    };

    editorTabs.update(tabs => [...tabs, newTab]);
    activeEditorTabId.set(newTab.id);
    currentView.set('editor');
  } catch (err) {
    alert(`Failed to open file: ${err}`);
  }
}

export async function deleteFiles(
  filesToDelete: string[],
  currentPath: string,
  confirmDelete: boolean,
  reloadFn: (path: string, target: string | null, force: boolean) => Promise<void>
) {
  if (filesToDelete.length === 0) return;

  if (confirmDelete) {
    const msg = filesToDelete.length === 1
      ? `Delete "${filesToDelete[0]}"?`
      : `Delete ${filesToDelete.length} items?`;
    if (!confirm(msg)) return;
  }

  try {
    for (const fileName of filesToDelete) {
      const deletePath = joinPath(currentPath, fileName);
      await invoke('delete_item', { path: deletePath });
    }
    directoryCache.invalidate(currentPath);
    reloadFn(currentPath, null, true);
  } catch (err) {
    alert('Error deleting: ' + err);
  }
}

export async function createItem(
  type: 'folder' | 'file',
  name: string,
  currentPath: string,
  reloadFn: (path: string, target: string | null, force: boolean) => Promise<void>
) {
  if (!name || !currentPath) return;

  try {
    const existingFiles = await invoke<any[]>('read_directory', { path: currentPath });
    let finalName = name;

    if (existingFiles.some(f => f.name === name)) {
      let counter = 1;
      const ext = name.includes('.') ? '.' + name.split('.').pop() : '';
      const baseName = ext && type === 'file' ? name.slice(0, -ext.length) : name;

      while (existingFiles.some(f => f.name === finalName)) {
        finalName = type === 'file' && ext
          ? `${baseName} (${counter})${ext}`
          : `${baseName} (${counter})`;
        counter++;
      }
    }

    const finalPath = joinPath(currentPath, finalName);

    if (type === 'folder') {
      await invoke('create_directory', { path: finalPath });
    } else {
      await invoke('create_file', { path: finalPath });
    }

    directoryCache.invalidate(currentPath);
    await reloadFn(currentPath, finalName, true);
  } catch (err) {
    alert(`Error creating ${type}: ${err}`);
  }
}

export async function renameFile(
  oldName: string,
  newName: string,
  currentPath: string,
  reloadFn: (path: string, target: string | null, force: boolean) => Promise<void>
) {
  if (!newName || newName === oldName) return;

  const oldPath = joinPath(currentPath, oldName);

  try {
    await invoke('rename_item', { path: oldPath, newName });
    directoryCache.invalidate(currentPath);
    await reloadFn(currentPath, newName, true);
  } catch (err) {
    alert('Rename failed: ' + err);
  }
}