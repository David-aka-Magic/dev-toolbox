<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { currentView } from '$lib/stores/viewStore';
  import { viewMode, sortConfig, sortFiles } from '$lib/stores/viewModeStore';
  import { settings } from '$lib/stores/settingsStore';
  import { tick } from "svelte";
  import { onMount, onDestroy } from 'svelte';

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
  import { directoryCache } from '$lib/stores/directoryCacheStore';
  import { get } from 'svelte/store';

  let files: any[] = [];
  let isLoading = false;
  let fileGridRef: any;

  let isPasting = false;

  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let menuTargetFile: string | null = null;
  let menuSelection: Set<string> = new Set();

  let renamingFile: string | null = null;
  let creationType: 'folder' | 'file' | null = null;

  let folderSizes: Map<string, number | null> = new Map();

  let lastLoadedPath: string | null = null;
  let lastRefreshCounter: number | undefined = undefined;
  let forceRefreshListener: (() => void) | null = null;


  $: visibleFiles = $settings.fileShowHidden 
    ? files 
    : files.filter(f => !f.name.startsWith('.'));

  $: sortedFiles = sortFiles(visibleFiles, $sortConfig);

  $: if ($activeTab?.path && ($activeTab.path !== lastLoadedPath || $activeTab.refreshCounter !== lastRefreshCounter)) {
      loadFiles($activeTab.path);
      lastRefreshCounter = $activeTab.refreshCounter;
  }

  $: if ($settings.fileShowFolderSize && files.length > 0) {
    calculateFolderSizes();
  }

  $: console.log('FileManager: menuTargetFile is now:', menuTargetFile);

  onMount(() => {
    forceRefreshListener = () => {
      const currentPath = $activeTab?.path;
      if (currentPath) {
        directoryCache.invalidate(currentPath);
        loadFiles(currentPath, null, true);
      }
    };
    window.addEventListener('force-file-refresh', forceRefreshListener);
  });
  
  onDestroy(() => {
    if (forceRefreshListener) {
      window.removeEventListener('force-file-refresh', forceRefreshListener);
    }
  });
  
  async function calculateFolderSizes() {
    const folders = files.filter(f => f.is_dir);
    
    for (const folder of folders) {
      if (folderSizes.has(folder.path)) continue;
      
      try {
        const size = await invoke<number | null>("get_directory_size", { path: folder.path });
        folderSizes.set(folder.path, size);
        folderSizes = folderSizes;
      } catch (err) {
        folderSizes.set(folder.path, null);
        folderSizes = folderSizes;
      }
    }
  }

  async function loadFiles(path: string, targetSelect: string | null = null, forceRefresh: boolean = false) {
    if (path === lastLoadedPath && !targetSelect && !forceRefresh) return;
    
    if (!forceRefresh && !targetSelect) {
      const cached = directoryCache.get(path);
      if (cached) {
        files = cached;
        lastLoadedPath = path;
        fileSelection.reset();
        thumbnailLoader.queueThumbnails(files);
        folderSizes = new Map();
        return;
      }
    }

    isLoading = true;
    lastLoadedPath = path;
    
    if (!targetSelect) {
      fileSelection.reset();
    }
    
    renamingFile = null;
    creationType = null;
    folderSizes = new Map();

    try {
      files = await invoke("read_directory", { path });
      
      directoryCache.set(path, files);
      
      thumbnailLoader.queueThumbnails(files);

      if (targetSelect) {
        await tick();
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
    const selected = get(selectedFiles);
    if (!currentPath || selected.size === 0) return;
    
    const fileNames = Array.from(selected);
    const filePaths = fileNames.map(name => joinPath(currentPath, name));
    clipboard.copy(filePaths, fileNames, currentPath);
  }

  function cutSelectedFiles() {
    const currentPath = $activeTab?.path;
    const selected = get(selectedFiles);
    if (!currentPath || selected.size === 0) return;
    
    const fileNames = Array.from(selected);
    const filePaths = fileNames.map(name => joinPath(currentPath, name));
    clipboard.cut(filePaths, fileNames, currentPath);
  }

  async function pasteFiles(targetPath?: string) {
    const currentPath = $activeTab?.path;
    const destinationPath = targetPath || currentPath;
    
    if (!destinationPath) return;
    
    const clipboardState = clipboard.getState();
    if (clipboardState.files.length === 0) return;
    if (isPasting) return;
    
    isPasting = true;

    try {
      const destFiles = await invoke<any[]>("read_directory", { path: destinationPath });
      
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
        loadFiles(currentPath, null, true);
      }
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

  async function handleMenuAction(action: string, targetFileFromMenu: string | null) {
    console.log('=== handleMenuAction ===');
    console.log('action:', action);
    console.log('targetFileFromMenu (from ContextMenu):', targetFileFromMenu);
    console.log('menuTargetFile (local state):', menuTargetFile);
    console.log('menuSelection:', Array.from(menuSelection));
    
    const currentPath = $activeTab?.path;
    const targetFileName = targetFileFromMenu;
    const targetFile = targetFileName ? files.find(f => f.name === targetFileName) : null;
    const currentSelection = menuSelection;
    
    console.log('Resolved targetFile object:', targetFile);
    
    showMenu = false;
    
    if (!currentPath) {
      console.log('No currentPath, returning');
      return;
    }

    if (action === 'refresh') {
      directoryCache.invalidate(currentPath);
      loadFiles(currentPath, null, true);
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

    if (action === 'new_folder') {
      creationType = 'folder';
      return;
    }
    
    if (action === 'new_file') {
      creationType = 'file';
      return;
    }

    if (!targetFileName) {
      console.log('No targetFileName, returning');
      return;
    }

    if (action === 'paste_into' && targetFile?.is_dir) {
      pasteFiles(targetFile.path);
      return;
    }

    if (action === 'open') {
      console.log('Executing open');
      if (targetFile?.is_dir) {
        fileTabs.updateActivePath(targetFile.path);
      } else if (targetFile) {
        openFile(targetFile);
      }
      return;
    }
    
    if (action === 'open_new_tab' && targetFile?.is_dir) {
      fileTabs.addTab(targetFile.path);
      return;
    }
    
    if (action === 'open_in_editor' && targetFile) {
      console.log('Executing open_in_editor with:', targetFile);
      openInEditor(targetFile);
      return;
    }
    
    if (action === 'rename') {
      console.log('Executing rename, setting renamingFile to:', targetFileName);
      renamingFile = targetFileName;
      return;
    }
    
    if (action === 'delete') {
      console.log('Executing delete');
      let filesToDelete: string[];
      
      if (currentSelection.size > 0 && currentSelection.has(targetFileName)) {
        filesToDelete = Array.from(currentSelection);
      } else {
        filesToDelete = [targetFileName];
      }
      
      console.log('Files to delete:', filesToDelete);
      
      if (filesToDelete.length === 0) return;
      
      if (confirmDelete(filesToDelete)) {
        try {
          for (const fileName of filesToDelete) {
            const deletePath = joinPath(currentPath, fileName);
            console.log('Deleting:', deletePath);
            await invoke('delete_item', { path: deletePath });
          }
          directoryCache.invalidate(currentPath);
          loadFiles(currentPath, null, true);
        } catch (err) {
          alert('Error deleting: ' + err);
        }
      }
      return;
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
    console.log('openInEditor called with:', file);
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
    if (creationType || renamingFile) {
      return;
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
    
    console.log('handleItemContextMenu called with fileName:', fileName);
    
    if (creationType) return;

    menuX = mouseEvent.clientX;
    menuY = mouseEvent.clientY;
    menuTargetFile = fileName;
    
    console.log('Set menuTargetFile to:', menuTargetFile);

    const currentSelection = get(selectedFiles);
    if (fileName && !currentSelection.has(fileName)) {
      fileSelection.clearSelection();
      const index = sortedFiles.findIndex(f => f.name === fileName);
      if (index !== -1) {
        fileSelection.selectFile(fileName, index);
      }
      menuSelection = new Set([fileName]);
    } else {
      menuSelection = new Set(currentSelection);
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
    menuSelection = get(selectedFiles);
    showMenu = true;
  }

  async function handleItemDrop(detail: any) {
    const { event: dropEvent, file: targetFolder } = detail;
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    await fileDragDrop.handleItemDrop(dropEvent, targetFolder, currentPath, () => {
      directoryCache.invalidate(currentPath);
      loadFiles(currentPath, null, true);
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
        directoryCache.invalidate(currentPath!);
        await loadFiles(currentPath!, newName, true);
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
        const existingFiles = await invoke<any[]>("read_directory", { path: currentPath });
        let finalName = name;
        
        if (existingFiles.some(f => f.name === name)) {
          let counter = 1;
          const ext = name.includes('.') ? '.' + name.split('.').pop() : '';
          const baseName = ext && type === 'file' ? name.slice(0, -ext.length) : name;
          
          while (existingFiles.some(f => f.name === finalName)) {
            finalName = type === 'file' && ext ? `${baseName} (${counter})${ext}` : `${baseName} (${counter})`;
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
        await loadFiles(currentPath, finalName, true);
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

    if (event.key === 'F2') {
      event.preventDefault();
      const selected = get(selectedFiles);
      if (selected.size === 1) {
        renamingFile = Array.from(selected)[0];
      }
      return;
    }

    if (event.key === 'F5') {
      event.preventDefault();
      const currentPath = $activeTab?.path;
      if (currentPath) {
        directoryCache.invalidate(currentPath);
        loadFiles(currentPath, null, true);
      }
      return;
    }

    if (sortedFiles.length === 0) return;

    if (event.key === 'Delete') {
      event.preventDefault();
      const currentPath = $activeTab?.path;
      const selected = get(selectedFiles);
      if (!currentPath || selected.size === 0) return;
      
      const filesToDelete = Array.from(selected);
      
      if (confirmDelete(filesToDelete)) {
        (async () => {
          try {
            for (const fileName of filesToDelete) {
              const fullPath = joinPath(currentPath, fileName);
              await invoke('delete_item', { path: fullPath });
            }
            directoryCache.invalidate(currentPath);
            loadFiles(currentPath, null, true);
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
      targetFile={menuTargetFile}
      onclose={() => { showMenu = false; }}
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