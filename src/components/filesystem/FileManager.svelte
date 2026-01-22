<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { currentView } from '$lib/stores/viewStore';
  import { tick } from "svelte";

  import FileGrid from './FileGrid.svelte';
  import ContextMenu from './ContextMenu.svelte';
  
  import { fileSelection, selectedFiles } from './/fileoperations/useFileSelection';
  import { fileDragDrop } from './/fileoperations/useFileDragDrop';
  import { thumbnailLoader } from './/fileoperations/useThumbnailLoader';
  import { joinPath } from './/fileoperations/fileUtils';

  let files: any[] = [];
  let isLoading = false;
  let fileGridRef: FileGrid;

  // Context Menu
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let menuTargetFile: string | null = null;

  // Rename & Creation
  let renamingFile: string | null = null;
  let creationType: 'folder' | 'file' | null = null;

  // Load files when active tab path changes
  $: if ($activeTab && $activeTab.path) {
    loadFiles($activeTab.path);
  }

  async function loadFiles(path: string, targetSelect: string | null = null) {
    isLoading = true;
    
    if (!targetSelect) {
      fileSelection.reset();
    }
    
    renamingFile = null;
    creationType = null;
    thumbnailLoader.clearThumbnails();

    try {
      files = await invoke("read_directory", { path });
      thumbnailLoader.queueThumbnails(files);

      if (targetSelect) {
        const index = files.findIndex(f => f.name === targetSelect);
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

  // Menu Actions
  async function handleMenuAction(action: string) {
    showMenu = false;
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    if (action === 'refresh') {
      loadFiles(currentPath);
      return;
    }

    if (menuTargetFile) {
      const fullPath = joinPath(currentPath, menuTargetFile);
      const targetFile = files.find(f => f.name === menuTargetFile);

      if (action === 'open_in_editor' && targetFile) {
        openInEditor(targetFile);
      }
      if (action === 'rename') {
        renamingFile = menuTargetFile;
      }
      if (action === 'delete') {
        if (confirm(`Delete "${menuTargetFile}"?`)) {
          try {
            await invoke('delete_item', { path: fullPath });
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

  // Open file in editor
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

  // Grid event handlers
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
      openInEditor(file);
    }
  }

  function handleItemContextMenu(detail: any) {
    const { event: mouseEvent, fileName } = detail;
    
    if (creationType) return;

    // Don't let this close the menu if it's already being set
    const willShowMenu = true;

    menuX = mouseEvent.clientX;
    menuY = mouseEvent.clientY;
    menuTargetFile = fileName;

    if (fileName && !$selectedFiles.has(fileName)) {
      fileSelection.clearSelection();
      const index = files.findIndex(f => f.name === fileName);
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

  // Item drop
  async function handleItemDrop(detail: any) {
    const { event: dropEvent, file: targetFolder } = detail;
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    await fileDragDrop.handleItemDrop(dropEvent, targetFolder, currentPath, () => {
      loadFiles(currentPath);
    });
  }

  // Rename
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

  // Creation
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

  // Keyboard navigation
  function handleKeyDown(event: KeyboardEvent) {
    if (renamingFile || creationType) {
      if (event.key === 'Escape') {
        renamingFile = null;
        creationType = null;
      }
      return;
    }

    if (files.length === 0) return;

    if (event.key === 'Backspace') {
      fileTabs.goBack();
      return;
    }

    if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
      const containerWidth = fileGridRef?.getContainerWidth() || 800;
      fileSelection.handleKeyDown(event, files, containerWidth);
    }

    if (event.key === 'a' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      fileSelection.selectAll(files);
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
  <FileGrid
    bind:this={fileGridRef}
    {files}
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

  {#if showMenu}
    <ContextMenu
      x={menuX}
      y={menuY}
      options={menuTargetFile
        ? (files.find(f => f.name === menuTargetFile)?.is_dir
            ? [
                { label: 'Rename', action: 'rename' },
                { label: 'Delete', action: 'delete', danger: true }
              ]
            : [
                { label: 'Open in Editor', action: 'open_in_editor' },
                { label: 'Rename', action: 'rename' },
                { label: 'Delete', action: 'delete', danger: true }
              ])
        : [
            { label: 'New Folder', action: 'new_folder' },
            { label: 'New File', action: 'new_file' },
            { label: 'SEPARATOR', action: '' },
            { label: 'Refresh', action: 'refresh' }
          ]}
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
    z-index: 10000;
  }
</style>