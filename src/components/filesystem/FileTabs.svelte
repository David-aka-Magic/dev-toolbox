<script lang="ts">
  import { fileTabs } from '$lib/stores/fileTabStore';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { tick } from 'svelte';

  let editingId: string | null = null;
  let editInput: HTMLInputElement;
  let dragOverTabId: string | null = null;

  function handleWheel(e: WheelEvent) {
    const container = e.currentTarget as HTMLElement;
    if (e.deltaY !== 0) {
      container.scrollLeft += e.deltaY;
      e.preventDefault(); 
    }
  }

  function close(e: Event, id: string) {
    e.stopPropagation();
    fileTabs.closeTab(id);
  }

  async function startEditing(id: string) {
    editingId = id;
    await tick();
    editInput?.focus();
    editInput?.select();
  }

  function finishEditing(id: string, newName: string) {
    if (newName.trim()) fileTabs.renameTab(id, newName);
    editingId = null;
  }

  function handleKey(e: KeyboardEvent, id: string, name: string) {
    if (e.key === 'Enter') finishEditing(id, name);
    if (e.key === 'Escape') editingId = null;
  }

  function handleTabKeyDown(e: KeyboardEvent, id: string) {
    if (e.key === 'Enter' || e.key === ' ') {
      fileTabs.setActive(id);
    }
  }

  function handleCloseKeyDown(e: KeyboardEvent, id: string) {
     if (e.key === 'Enter' || e.key === ' ') {
        e.stopPropagation();
        fileTabs.closeTab(id);
     }
  }

  function handleTabDragEnter(event: DragEvent, tabId: string) {
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    dragOverTabId = tabId;
  }

  function handleTabDragOver(event: DragEvent, tabId: string) {
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  function handleTabDragLeave(event: DragEvent) {
    dragOverTabId = null;
  }

  async function handleTabDrop(event: DragEvent, tabId: string, tabPath: string) {
    event.preventDefault();
    event.stopPropagation();
    dragOverTabId = null;

    const state = $fileDragDrop;
    if (!state.draggedFile) return;

    const activeTab = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId);
    const currentPath = activeTab?.path;
    
    if (!currentPath) return;

    if (tabPath === currentPath) {
      console.log("⚠️ Cannot drop into the same folder");
      return;
    }

    console.log(`📦 Moving ${state.draggedFiles.length} file(s) to tab:`, tabPath);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { directoryCache } = await import('$lib/stores/directoryCacheStore');
      
      for (const fileName of state.draggedFiles) {
        const sourcePath = `${currentPath}\\${fileName}`;
        
        if (tabPath.startsWith(sourcePath + '\\') || tabPath.startsWith(sourcePath + '/')) {
          alert(`Cannot move "${fileName}" into its own subdirectory`);
          continue;
        }
        
        console.log(`📦 Moving: ${sourcePath} -> ${tabPath}`);
        await invoke('move_item', { src: sourcePath, dest: tabPath });
      }
      
      directoryCache.invalidate(currentPath);
      directoryCache.invalidate(tabPath);
      window.dispatchEvent(new CustomEvent('force-file-refresh'));
      fileDragDrop.handleDragEnd();
    } catch (err) {
      console.error("❌ Move error:", err);
      alert("Move failed: " + err);
    }
  }
</script>

<div class="tabs-container" on:wheel={handleWheel}>
  {#each $fileTabs.tabs as tab (tab.id)}
    <div 
      class="tab" 
      class:active={tab.id === $fileTabs.activeId}
      class:drag-over={dragOverTabId === tab.id}
      on:click={() => fileTabs.setActive(tab.id)}
      on:keydown={(e) => handleTabKeyDown(e, tab.id)}
      on:dblclick={() => startEditing(tab.id)}
      on:dragenter={(e) => handleTabDragEnter(e, tab.id)}
      on:dragover={(e) => handleTabDragOver(e, tab.id)}
      on:dragleave={handleTabDragLeave}
      on:drop={(e) => handleTabDrop(e, tab.id, tab.path)}
      role="tab"
      tabindex="0"
      aria-selected={tab.id === $fileTabs.activeId}
    >
      {#if editingId === tab.id}
        <input 
          bind:this={editInput}
          type="text"
          value={tab.name}
          on:keydown={(e) => handleKey(e, tab.id, e.currentTarget.value)}
          on:blur={() => editingId = null}
          class="tab-input"
        />
      {:else}
        <span class="tab-name">{tab.name}</span>
      {/if}
      
      {#if $fileTabs.tabs.length > 1}
        <button 
          class="close-btn" 
          on:click={(e) => close(e, tab.id)}
          on:keydown={(e) => handleCloseKeyDown(e, tab.id)}
          aria-label="Close tab"
        >×</button>
      {/if}
    </div>
  {/each}
  
  <button class="add-tab-btn" on:click={() => fileTabs.addTab()} aria-label="New tab">+</button>
</div>

<style>
  .tabs-container {
    display: flex;
    align-items: center;
    height: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }
  .tabs-container::-webkit-scrollbar { display: none; }

  .tab {
    display: flex;
    align-items: center;
    height: 100%;
    padding: 0 12px;
    background: var(--bg-tab-inactive);
    border-right: 1px solid var(--border);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s;
    min-width: 100px;
    max-width: 200px;
  }
  
  .tab:hover {
    background: var(--bg-tab-hover);
  }
  
  .tab.active {
    background: var(--bg-tab-active);
  }

  .tab.drag-over {
    background: rgba(59, 130, 246, 0.3);
    outline: 2px dashed #3b82f6;
    outline-offset: -2px;
  }

  .tab-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
    color: var(--text-main);
  }

  .tab-input {
    flex: 1;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 2px;
    padding: 2px 4px;
    font-size: 13px;
    color: var(--text-main);
    outline: none;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    margin-left: 8px;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
  }
  
  .close-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .add-tab-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    padding: 0 12px;
    height: 100%;
    min-width: 32px;
  }
  
  .add-tab-btn:hover {
    background: var(--bg-tab-hover);
    color: var(--text-main);
  }
</style>