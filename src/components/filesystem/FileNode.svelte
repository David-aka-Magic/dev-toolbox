<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { fileTabs } from "$lib/stores/fileTabStore";
  import { fileDragDrop } from './hooks/useFileDragDrop';

  export let name: string;
  export let path: string;
  export let isDir: boolean;
  export let depth: number = 0;
  export let isRoot: boolean = false;

  let isOpen = isRoot;
  let children: any[] = [];
  let loading = false;
  let isDragOver = false;
  let isActive = false;

  $: currentPath = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId)?.path || '';
  
  $: {
    const normCurrent = currentPath.toLowerCase().replace(/\//g, '\\');
    const normThis = path.toLowerCase().replace(/\//g, '\\');
    isActive = normCurrent === normThis || (isDir && normCurrent.startsWith(normThis + '\\'));
  }

  $: if (isRoot && children.length === 0 && !loading) {
    loadChildren();
  }

  async function loadChildren() {
    if (!isDir) return;
    loading = true;
    try {
      children = await invoke("read_directory", { path });
    } catch (err) {
      console.error(err);
    }
    loading = false;
  }

  async function toggle() {
    if (!isDir) return;
    isOpen = !isOpen;

    if (isOpen && children.length === 0) {
      await loadChildren();
    }
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    // Navigate to this path in the file manager
    if (isDir) {
      fileTabs.updateActivePath(path);
    } else {
      // For files, navigate to the parent directory
      const parentPath = path.split(/[\\/]/).slice(0, -1).join('\\');
      if (parentPath) {
        fileTabs.updateActivePath(parentPath);
      }
    }
  }

  // Drag and drop handlers
  function handleDragEnter(event: DragEvent) {
    if (!isDir) return;
    
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    isDragOver = true;
  }

  function handleDragOver(event: DragEvent) {
    if (!isDir) return;
    
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    // Ensure isDragOver stays true during dragging
    isDragOver = true;
    
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  function handleDragLeave(event: DragEvent) {
    // Only clear if we're actually leaving this element
    const relatedTarget = event.relatedTarget as HTMLElement;
    const currentTarget = event.currentTarget as HTMLElement;
    
    // Check if we're leaving to a child element
    if (relatedTarget && currentTarget.contains(relatedTarget)) {
      return;
    }
    
    isDragOver = false;
  }

  async function handleDrop(event: DragEvent) {
    if (!isDir) return;
    
    event.preventDefault();
    event.stopPropagation();
    isDragOver = false;

    const state = $fileDragDrop;
    if (!state.draggedFile) return;

    const destPath = path;
    
    // Get the active tab's path as the source directory
    const activeTab = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId);
    const sourceDirPath = activeTab?.path;
    
    if (!sourceDirPath) return;
    
    // Don't drop into the same folder
    if (destPath === sourceDirPath) {
      console.log("⚠️ Cannot drop into the same folder");
      return;
    }

    console.log(`📦 Moving ${state.draggedFiles.length} file(s) to:`, destPath);
    console.log('Source directory:', sourceDirPath);
    console.log('Destination directory:', destPath);

    try {
      for (const fileName of state.draggedFiles) {
        // Always construct the source path from current directory + filename
        // Don't trust the cached path in draggedFilePaths since the file might have moved
        const sourcePath = `${sourceDirPath}\\${fileName}`;
        
        // Safety check: don't move a folder into its own subdirectory
        if (destPath.startsWith(sourcePath + '\\') || destPath.startsWith(sourcePath + '/')) {
          alert(`Cannot move "${fileName}" into its own subdirectory`);
          continue;
        }
        
        console.log(`📦 MOVE_ITEM: source="${sourcePath}" destination="${destPath}"`);
        const result = await invoke('move_item', { src: sourcePath, dest: destPath });
        console.log('Move result:', result);
      }
      
      // Invalidate cache for both source and destination
      const { directoryCache } = await import('$lib/stores/directoryCacheStore');
      directoryCache.invalidate(sourceDirPath);
      directoryCache.invalidate(destPath);
      
      // Trigger immediate reload by navigating to same path with force
      const currentPath = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId)?.path;
      if (currentPath === sourceDirPath || currentPath === destPath) {
        // We're viewing either source or destination, force an update
        window.dispatchEvent(new CustomEvent('force-file-refresh'));
      }
      
      // Clear the drag state since files have moved
      fileDragDrop.handleDragEnd();
      
      // Refresh this node if it's expanded
      if (isOpen) {
        children = [];
        await loadChildren();
      }
    } catch (err) {
      console.error("❌ Move error:", err);
      alert("Move failed: " + err);
    }
  }
</script>

{#if !isRoot}
  <div 
    class="file-item"
    class:drag-over={isDragOver}
    class:active={isActive}
    style="padding-left: {depth * 12 + 8}px"
    on:click={toggle}
    on:contextmenu={handleContextMenu}
    on:keydown={(e) => e.key === 'Enter' && toggle()}
    on:dragenter={handleDragEnter}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
    role="button"
    tabindex="0"
  >
    <span class="file-icon">
      {#if isDir}
        {isOpen ? '📂' : '📁'}
      {:else}
        📄
      {/if}
    </span>
    <span class="file-name">{name}</span>
  </div>
{/if}

{#if (isOpen || isRoot)}
  {#if loading}
    <div class="file-loading" style="padding-left: {(depth + 1) * 12 + 8}px">Loading...</div>
  {:else}
    {#each children as child}
      <svelte:self 
        name={child.name} 
        path={child.path} 
        isDir={child.is_dir} 
        depth={isRoot ? depth : depth + 1}
        isRoot={false}
      />
    {/each}
  {/if}
{/if}

<style>
  .file-item {
    display: flex;
    align-items: center;
    cursor: pointer;
    padding: 4px 0;
    white-space: nowrap;
    color: var(--text-main); 
    font-size: 13px;
    transition: background 0.15s;
  }

  .file-item:hover {
    background-color: var(--hover-bg);
  }

  .file-item.active {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
  }

  .file-item.drag-over {
    background: rgba(59, 130, 246, 0.2);
    outline: 2px dashed #3b82f6;
    outline-offset: -2px;
  }

  .file-icon {
    margin-right: 6px;
    font-size: 14px;
  }

  .file-name {
    user-select: none;
  }

  .file-loading {
    color: var(--text-muted);
    font-size: 11px;
    font-style: italic;
    padding: 4px 0;
  }
</style>