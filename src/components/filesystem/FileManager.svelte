<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { tick } from "svelte";
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';

  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { viewMode, sortConfig, sortFiles } from '$lib/stores/viewModeStore';
  import { settings } from '$lib/stores/settingsStore';
  import { directoryCache } from '$lib/stores/directoryCacheStore';

  import FileGrid from './FileGrid.svelte';
  import FileListView from './FileListView.svelte';
  import FileDetailsView from './FileDetailsView.svelte';
  import ContextMenu from '../ui/ContextMenu.svelte';
  import FileDetailsModal from './FileDetailsModal.svelte';

  import { fileSelection, selectedFiles } from './hooks/useFileSelection';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { thumbnailLoader } from './hooks/useThumbnailLoader';
  import { joinPath } from './hooks/fileUtils';

  import { getMenuOptions } from './hooks/useContextMenu';
  import { openFile, openInPreview, openInEditor, createItem, renameFile } from './hooks/useFileActions';
  import { copySelectedFiles, cutSelectedFiles, pasteFiles } from './hooks/useClipboardActions';
  import { handleKeyDown as onKeyDown } from './hooks/useKeyboardShortcuts';
  import type { KeyboardContext } from './hooks/useKeyboardShortcuts';

  let files: any[] = [];
  let isLoading = false;
  let fileGridRef: any;

  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let menuTargetFile: string | null = null;
  let menuSelection: Set<string> = new Set();

  let renamingFile: string | null = null;
  let creationType: 'folder' | 'file' | null = null;

  let showDetailsModal = false;
  let detailsModalFile: any = null;

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

  function handleBackgroundClick() {
    if (creationType || renamingFile) return;
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
    const currentPath = $activeTab?.path;
    if (currentPath && newName && newName !== renamingFile) {
      await renameFile(renamingFile, newName, currentPath, loadFiles);
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
      await createItem(type, name, currentPath, loadFiles);
    }
  }

  function handleCreationCancel() {
    creationType = null;
  }

  async function handleMenuAction(action: string, targetFileFromMenu: string | null) {
    const currentPath = $activeTab?.path;
    const targetFileName = targetFileFromMenu;
    const targetFile = targetFileName ? files.find(f => f.name === targetFileName) : null;
    const currentSelection = menuSelection;

    showMenu = false;

    if (!currentPath) return;

    if (action === 'refresh') {
      directoryCache.invalidate(currentPath);
      loadFiles(currentPath, null, true);
      return;
    }

    if (action === 'paste') {
      pasteFiles(currentPath, undefined, loadFiles);
      return;
    }

    if (action === 'copy') {
      copySelectedFiles(currentPath);
      return;
    }

    if (action === 'cut') {
      cutSelectedFiles(currentPath);
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

    if (!targetFileName) return;

    if (action === 'paste_into' && targetFile?.is_dir) {
      pasteFiles(currentPath, targetFile.path, loadFiles);
      return;
    }

    if (action === 'open') {
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
      openInEditor(targetFile);
      return;
    }

    if (action === 'rename') {
      renamingFile = targetFileName;
      return;
    }

    if (action === 'delete') {
      let filesToDelete: string[];
      if (currentSelection.size > 0 && currentSelection.has(targetFileName)) {
        filesToDelete = Array.from(currentSelection);
      } else {
        filesToDelete = [targetFileName];
      }

      if (filesToDelete.length === 0) return;

      const { deleteFiles } = await import('./hooks/useFileActions');
      deleteFiles(filesToDelete, currentPath, $settings.fileConfirmDelete, loadFiles);
      return;
    }

    if (action === 'details') {
      if (targetFile) {
        detailsModalFile = targetFile;
        showDetailsModal = true;
      }
      return;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    const ctx: KeyboardContext = {
      renamingFile,
      creationType,
      currentPath: $activeTab?.path,
      sortedFiles,
      fileGridRef,
      confirmDeleteSetting: $settings.fileConfirmDelete,
      setRenamingFile: (name) => { renamingFile = name; },
      setCreationType: (type) => { creationType = type; },
      loadFiles,
      goBack: () => fileTabs.goBack(),
    };
    onKeyDown(event, ctx);
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
      options={getMenuOptions(menuTargetFile, files)}
      targetFile={menuTargetFile}
      onclose={() => { showMenu = false; }}
      onclick={handleMenuAction}
    />
  {/if}

  <FileDetailsModal
    open={showDetailsModal}
    filePath={detailsModalFile?.path || ''}
    fileName={detailsModalFile?.name || ''}
    isDir={detailsModalFile?.is_dir || false}
    onclose={() => { showDetailsModal = false; detailsModalFile = null; }}
  />
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