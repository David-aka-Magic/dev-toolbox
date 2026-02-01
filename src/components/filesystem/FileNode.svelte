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

  async function handleClick(event: MouseEvent) {
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      event.stopPropagation();
      
      if (isDir) {
        fileTabs.updateActivePath(path);
      } else {
        const parentPath = path.split(/[\\/]/).slice(0, -1).join('\\');
        if (parentPath) {
          fileTabs.updateActivePath(parentPath);
        }
      }
      return;
    }

    if (isDir) {
      isOpen = !isOpen;
      if (isOpen && children.length === 0) {
        await loadChildren();
      }
    }
  }

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
  }

  function handleDragLeave(event: DragEvent) {
    isDragOver = false;
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isDragOver = false;

    if (!isDir) return;

    const state = $fileDragDrop;
    if (!state.draggedFile) return;

    const sourcePath = state.draggedFilePath;
    const targetPath = `${path}\\${state.draggedFile}`;

    if (sourcePath === targetPath) return;

    try {
      await invoke('move_item', { src: sourcePath, dest: path });
      await loadChildren();
    } catch (err) {
      console.error('Drop failed:', err);
    }
  }
</script>

<div class="file-node">
  <button 
    class="node-row"
    class:active={isActive}
    class:drag-over={isDragOver}
    style="padding-left: {depth * 12 + 8}px"
    on:click={handleClick}
    on:dragenter={handleDragEnter}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
    title="Ctrl+Click to open in file manager"
  >
    {#if isDir}
      <span class="chevron" class:open={isOpen}>▶</span>
      <span class="icon">📁</span>
    {:else}
      <span class="chevron-placeholder"></span>
      <span class="icon">📄</span>
    {/if}
    <span class="name">{name}</span>
  </button>

  {#if isDir && isOpen}
    <div class="children">
      {#if loading}
        <div class="loading" style="padding-left: {(depth + 1) * 12 + 8}px">Loading...</div>
      {:else}
        {#each children as child}
          <svelte:self 
            name={child.name} 
            path={child.path} 
            isDir={child.is_dir}
            depth={depth + 1}
          />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .file-node {
    user-select: none;
  }

  .node-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    cursor: pointer;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 12px;
    width: 100%;
    text-align: left;
    border-radius: 3px;
    transition: background-color 0.1s;
  }

  .node-row:hover {
    background-color: var(--hover-bg);
    color: var(--text-main);
  }

  .node-row.active {
    background-color: var(--selection);
    color: var(--text-main);
  }

  .node-row.drag-over {
    background-color: rgba(59, 130, 246, 0.3);
    outline: 2px dashed #3b82f6;
  }

  .chevron {
    font-size: 8px;
    width: 12px;
    transition: transform 0.15s;
    color: var(--text-muted);
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .chevron-placeholder {
    width: 12px;
  }

  .icon {
    font-size: 14px;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .children {
    display: flex;
    flex-direction: column;
  }

  .loading {
    font-size: 11px;
    color: var(--text-muted);
    padding: 4px 8px;
  }
</style>