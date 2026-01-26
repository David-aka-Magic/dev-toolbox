<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { currentView } from '$lib/stores/viewStore';
  import { viewMode, sortConfig, sortFiles } from '$lib/stores/viewModeStore';
  import { settings } from '$lib/stores/settingsStore';
  import { tick } from "svelte";

  import FileGrid from './FileGrid.svelte';
  import FileListView from './FileListView.svelte';
  import FileDetailsView from './FileDetailsView.svelte';
  import ContextMenu from '../ui/ContextMenu.svelte';
  
  import { fileSelection, selectedFiles } from './hooks/useFileSelection';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { thumbnailLoader } from './hooks/useThumbnailLoader';
  import { joinPath, isImageFile, isVideoFile } from './hooks/fileUtils';
  import { clipboard } from '$lib/stores/clipboardStore';
  import { openMediaInNewWindow } from '$lib/utils/openMediaWindow';

  let files: any[] = [];
  let isLoading = false;
  let fileGridRef: any;

  let isPasting = false;

  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let menuTargetFile: string | null = null;

  let renamingFile: string | null = null;
  let creationType: 'folder' | 'file' | null = null;

  // Folder size cache
  let folderSizes: Map<string, number | null> = new Map();

  // Filter hidden files based on settings
  $: visibleFiles = $settings.fileShowHidden 
    ? files 
    : files.filter(f => !f.name.startsWith('.'));

  $: sortedFiles = sortFiles(visibleFiles, $sortConfig);

  $: if ($activeTab && $activeTab.path) {
    loadFiles($activeTab.path);
  }

  // Calculate folder sizes when setting is enabled
  $: if ($settings.fileShowFolderSize && files.length > 0) {
    calculateFolderSizes();
  }

  async function calculateFolderSizes() {
    const folders = files.filter(f => f.is_dir);
    
    for (const folder of folders) {
      if (folderSizes.has(folder.path)) continue;
      
      try {
        const folderFiles = await invoke<any[]>("read_directory", { path: folder.path });
        
        if (folderFiles.length > $settings.fileFolderSizeThreshold) {
          folderSizes.set(folder.path, null);
        } else {
          const totalSize = await invoke<number>("get_folder_size", { path: folder.path });
          folderSizes.set(folder.path, totalSize);
        }
        
        folderSizes = folderSizes;
      } catch (err) {
        folderSizes.set(folder.path, null);
        folderSizes = folderSizes;
      }
    }
  }

  async function loadFiles(path: string, targetSelect: string | null = null) {
    isLoading = true;
    
    if (!targetSelect) {
      fileSelection.reset();
    }
    
    renamingFile = null;
    creationType = null;
    thumbnailLoader.clearThumbnails();
    folderSizes = new Map();

    try {
      files = await invoke("read_directory", { path });
      thumbnailLoader.queueThumbnails(files);

      if (targetSelect) {
        const index = sortedFiles.findIndex(f => f.name === targetSelect);
        if (index !== -1) {
          fileSelection.selectFile(targetSelect, index);
          await tick();
          const el = document.getElementById(`file-btn-${index}`);
          el?.scrollIntoView({ block: 'center' });
        }
      }
    } catch (err) {
      console.error("Failed to load directory:", err);
    } finally {
      isLoading = false;
    }
  }

  function copySelectedFiles() {
    const currentPath = $activeTab?.path;
    if (!currentPath || $selectedFiles.size === 0) return;
    
    const fileNames = Array.from($selectedFiles);
    const filePaths = fileNames.map(name => joinPath(currentPath, name));
    clipboard.copy(filePaths, fileNames, currentPath);
  }

  function cutSelectedFiles() {
    const currentPath = $activeTab?.path;
    if (!currentPath || $selectedFiles.size === 0) return;
    
    const fileNames = Array.from($selectedFiles);
    const filePaths = fileNames.map(name => joinPath(currentPath, name));
    clipboard.cut(filePaths, fileNames, currentPath);
  }

  async function pasteFiles(targetPath?: string) {
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    const clipboardState = clipboard.getState();
    if (clipboardState.files.length === 0) return;

    if (isPasting) return;
    isPasting = true;

    const destinationPath = targetPath || currentPath;

    try {
      const destFiles = await invoke<any[]>("read_directory", { path: destinationPath });

      for (const sourcePath of clipboardState.files) {
        const fileName = sourcePath.split(/[\\/]/).pop()!;
        
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
          
          await invoke('copy_item', { source: sourcePath, destination: destinationPath, newName });
        } else if (clipboardState.operation === 'copy') {
          await invoke('copy_item', { source: sourcePath, destination: destinationPath });
        } else if (clipboardState.operation === 'cut') {
          await invoke('move_item', { source: sourcePath, destination: destinationPath });
        }
      }
      
      if (clipboardState.operation === 'cut') {
        clipboard.clear();
      }
      
      loadFiles(currentPath);
    } catch (err) {
      console.error("Paste error:", err);
      alert("Paste failed: " + err);
    } finally {
      isPasting = false;
    }
  }

  function getMenuOptions() {
    const clipboardState = clipboard.getState();
    const hasClipboard = clipboardState.files.length > 0;
    
    if (menuTargetFile) {
      const targetFile = files.find(f => f.name === menuTargetFile);
      if (targetFile?.is_dir) {
        return [
          { label: 'Open', action: 'open' },
          { label: 'Open in New Tab', action: 'open_new_tab' },
          { label: 'SEPARATOR', action: '' },
          { label: 'Cut', action: 'cut', shortcut: 'Ctrl+X' },
          { label: 'Copy', action: 'copy', shortcut: 'Ctrl+C' },
          { label: 'Paste Into', action: 'paste_into', disabled: !hasClipboard, shortcut: 'Ctrl+V' },
          { label: 'SEPARATOR', action: '' },
          { label: 'Rename', action: 'rename', shortcut: 'F2' },
          { label: 'Delete', action: 'delete', danger: true, shortcut: 'Del' }
        ];
      } else {
        return [
          { label: 'Open', action: 'open' },
          { label: 'Open in Editor', action: 'open_in_editor' },
          { label: 'SEPARATOR', action: '' },
          { label: 'Cut', action: 'cut', shortcut: 'Ctrl+X' },
          { label: 'Copy', action: 'copy', shortcut: 'Ctrl+C' },
          { label: 'SEPARATOR', action: '' },
          { label: 'Rename', action: 'rename', shortcut: 'F2' },
          { label: 'Delete', action: 'delete', danger: true, shortcut: 'Del' }
        ];
      }
    } else {
      return [
        { label: 'New Folder', action: 'new_folder' },
        { label: 'New File', action: 'new_file' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Paste', action: 'paste', disabled: !hasClipboard, shortcut: 'Ctrl+V' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Refresh', action: 'refresh', shortcut: 'F5' }
      ];
    }
  }

  function confirmDelete(filesToDelete: string[]): boolean {
    if (!$settings.fileConfirmDelete) return true;
    
    const confirmMessage = filesToDelete.length === 1
      ? `Delete "${filesToDelete[0]}"?`
      : `Delete ${filesToDelete.length} items?`;
    
    return confirm(confirmMessage);
  }

  async function handleMenuAction(action: string) {
    showMenu = false;
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    if (action === 'refresh') {
      loadFiles(currentPath);
      return;
    }

    if (action === 'paste') {
      pasteFiles();
      return;
    }

    if (action === 'copy') {
      copySelectedFiles();
      return;
    }

    if (action === 'cut') {
      cutSelectedFiles();
      return;
    }

    if (menuTargetFile) {
      const fullPath = joinPath(currentPath, menuTargetFile);
      const targetFile = files.find(f => f.name === menuTargetFile);

      if (action === 'paste_into' && targetFile?.is_dir) {
        pasteFiles(targetFile.path);
        return;
      }

      if (action === 'open' && targetFile?.is_dir) {
        fileTabs.updateActivePath(targetFile.path);
      }
      if (action === 'open_new_tab' && targetFile?.is_dir) {
        fileTabs.addTab(targetFile.path);
      }
      if (action === 'open' && !targetFile?.is_dir) {
        openFile(targetFile);
      }
      if (action === 'open_in_editor' && targetFile) {
        openInEditor(targetFile);
      }
      if (action === 'rename') {
        renamingFile = menuTargetFile;
      }
      if (action === 'delete') {
        const filesToDelete = $selectedFiles.size > 0 
          ? Array.from($selectedFiles) 
          : (menuTargetFile ? [menuTargetFile] : []);
        
        if (filesToDelete.length === 0) return;
        
        if (confirmDelete(filesToDelete)) {
          try {
            for (const fileName of filesToDelete) {
              const deletePath = joinPath(currentPath, fileName);
              await invoke('delete_item', { path: deletePath });
            }
            loadFiles(currentPath);
          } catch (err) {
            alert('Error deleting: ' + err);
          }
        }
      }
    } else {
      if (action === 'new_folder') creationType = 'folder';
      if (action === 'new_file') creationType = 'file';
    }
  }

  function openFile(file: any) {
    if (isImageFile(file.name)) {
      openMediaInNewWindow(file.path, file.name, 'image');
    } else if (isVideoFile(file.name)) {
      openMediaInNewWindow(file.path, file.name, 'video');
    } else {
      openInEditor(file);
    }
  }

  function openInPreview(file: any) {
    if (isImageFile(file.name)) {
      openMediaInNewWindow(file.path, file.name, 'image');
    } else if (isVideoFile(file.name)) {
      openMediaInNewWindow(file.path, file.name, 'video');
    } else {
      openInEditor(file);
    }
  }

  async function openInEditor(file: any) {
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

  function handleBackgroundClick() {
    if (creationType) {
      creationType = null;
    }
    fileSelection.clearSelection();
    showMenu = false;
  }

  function handleItemDblClick(detail: any) {
    const { file } = detail;
    if (renamingFile || creationType) return;

    if (file.is_dir) {
      fileTabs.updateActivePath(file.path);
    } else {
      switch ($settings.fileDoubleClickAction) {
        case 'preview':
          openInPreview(file);
          break;
        case 'editor':
          openInEditor(file);
          break;
        case 'open':
        default:
          openFile(file);
          break;
      }
    }
  }

  function handleItemContextMenu(detail: any) {
    const { event: mouseEvent, fileName } = detail;
    
    if (creationType) return;

    menuX = mouseEvent.clientX;
    menuY = mouseEvent.clientY;
    menuTargetFile = fileName;

    if (fileName && !$selectedFiles.has(fileName)) {
      fileSelection.clearSelection();
      const index = sortedFiles.findIndex(f => f.name === fileName);
      if (index !== -1) {
        fileSelection.selectFile(fileName, index);
      }
    }
    
    showMenu = true;
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    if (creationType) return;

    menuX = event.clientX;
    menuY = event.clientY;
    menuTargetFile = null;
    showMenu = true;
  }

  async function handleItemDrop(detail: any) {
    const { event: dropEvent, file: targetFolder } = detail;
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    await fileDragDrop.handleItemDrop(dropEvent, targetFolder, currentPath, () => {
      loadFiles(currentPath);
    });
  }

  async function handleRenameSubmit(detail: any) {
    if (!renamingFile) return;
    
    const { newName } = detail;
    if (newName && newName !== renamingFile) {
      const currentPath = $activeTab?.path;
      const oldPath = joinPath(currentPath!, renamingFile);
      
      try {
        await invoke('rename_item', { path: oldPath, newName });
        await loadFiles(currentPath!, newName);
      } catch (err) {
        alert("Rename failed: " + err);
      }
    }
    
    renamingFile = null;
  }

  async function handleCreationConfirm(detail: any) {
    if (!creationType) return;
    
    const { name } = detail;
    if (!name) {
      creationType = null;
      return;
    }

    const currentPath = $activeTab?.path;
    const type = creationType;
    creationType = null;

    if (currentPath) {
      try {
        if (type === 'folder') {
          await invoke('create_directory', { path: currentPath, name });
        } else {
          await invoke('create_file', { path: currentPath, name });
        }
        await loadFiles(currentPath, name);
      } catch (err) {
        alert(`Error creating ${type}: ${err}`);
      }
    }
  }

  function handleCreationCancel() {
    creationType = null;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (renamingFile || creationType) {
      if (event.key === 'Escape') {
        renamingFile = null;
        creationType = null;
      }
      return;
    }

    if ((event.ctrlKey || event.metaKey) && event.key === 'c') {
      event.preventDefault();
      copySelectedFiles();
      return;
    }

    if ((event.ctrlKey || event.metaKey) && event.key === 'x') {
      event.preventDefault();
      cutSelectedFiles();
      return;
    }

    if ((event.ctrlKey || event.metaKey) && event.key === 'v') {
      event.preventDefault();
      pasteFiles();
      return;
    }

    if (event.key === 'F2' && $selectedFiles.size === 1) {
      event.preventDefault();
      renamingFile = Array.from($selectedFiles)[0];
      return;
    }

    if (event.key === 'F5') {
      event.preventDefault();
      const currentPath = $activeTab?.path;
      if (currentPath) loadFiles(currentPath);
      return;
    }

    if (sortedFiles.length === 0) return;

    if (event.key === 'Delete') {
      event.preventDefault();
      const currentPath = $activeTab?.path;
      if (!currentPath || $selectedFiles.size === 0) return;
      
      const filesToDelete = Array.from($selectedFiles);
      
      if (confirmDelete(filesToDelete)) {
        (async () => {
          try {
            for (const fileName of filesToDelete) {
              const fullPath = joinPath(currentPath, fileName);
              await invoke('delete_item', { path: fullPath });
            }
            loadFiles(currentPath);
          } catch (err) {
            alert('Error deleting: ' + err);
          }
        })();
      }
      return;
    }

    if (event.key === 'Backspace') {
      fileTabs.goBack();
      return;
    }

    if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
      const containerWidth = fileGridRef?.getContainerWidth() || 800;
      fileSelection.handleKeyDown(event, sortedFiles, containerWidth);
    }

    if (event.key === 'a' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      fileSelection.selectAll(sortedFiles);
    }
  }
</script>

<svelte:window 
  on:keydown={handleKeyDown} 
  on:dragover={(e) => { e.preventDefault(); }} 
/>

<div
  class="file-manager"
  on:contextmenu={handleContextMenu}
  role="presentation"
>
  {#if $viewMode === 'grid'}
    <FileGrid
      bind:this={fileGridRef}
      files={sortedFiles}
      {isLoading}
      {renamingFile}
      {creationType}
      {folderSizes}
      onbackgroundclick={handleBackgroundClick}
      onitemdblclick={handleItemDblClick}
      onitemcontextmenu={handleItemContextMenu}
      onitemdrop={handleItemDrop}
      onrenamesubmit={handleRenameSubmit}
      oncreationconfirm={handleCreationConfirm}
      oncreationcancel={handleCreationCancel}
    />
  {:else if $viewMode === 'list'}
    <FileListView
      bind:this={fileGridRef}
      files={sortedFiles}
      {isLoading}
      {renamingFile}
      {creationType}
      onbackgroundclick={handleBackgroundClick}
      onitemdblclick={handleItemDblClick}
      onitemcontextmenu={handleItemContextMenu}
      onitemdrop={handleItemDrop}
      onrenamesubmit={handleRenameSubmit}
      oncreationconfirm={handleCreationConfirm}
      oncreationcancel={handleCreationCancel}
    />
  {:else if $viewMode === 'details'}
    <FileDetailsView
      bind:this={fileGridRef}
      files={sortedFiles}
      {isLoading}
      {renamingFile}
      {creationType}
      onbackgroundclick={handleBackgroundClick}
      onitemdblclick={handleItemDblClick}
      onitemcontextmenu={handleItemContextMenu}
      onitemdrop={handleItemDrop}
      onrenamesubmit={handleRenameSubmit}
      oncreationconfirm={handleCreationConfirm}
      oncreationcancel={handleCreationCancel}
    />
  {/if}

  {#if showMenu}
    <ContextMenu
      x={menuX}
      y={menuY}
      options={getMenuOptions()}
      onclose={() => (showMenu = false)}
      onclick={handleMenuAction}
    />
  {/if}
</div>

<style>
  .file-manager {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-main);
    color: var(--text-main);
    position: relative;
  }
</style>