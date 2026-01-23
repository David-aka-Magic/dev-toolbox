<script lang="ts">
  import { fileTabs } from '$lib/stores/fileTabStore';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { tick } from 'svelte';

  let editingId: string | null = null;
  let editInput: HTMLInputElement;
  let dragOverTabId: string | null = null;

  // Scroll tabs horizontally with mouse wheel
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

  // Drag & drop handlers for tabs
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

    // Get the active tab's path as the source
    const activeTab = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId);
    const currentPath = activeTab?.path;
    
    if (!currentPath) return;

    // Don't drop into the same folder
    if (tabPath === currentPath) {
      console.log("⚠️ Cannot drop into the same folder");
      return;
    }

    console.log(`📦 Moving ${state.draggedFiles.length} file(s) to tab:`, tabPath);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      for (const fileName of state.draggedFiles) {
        const sourcePath = state.draggedFilePaths.get(fileName) || `${currentPath}\\${fileName}`;
        
        // Safety check: don't move a folder into its own subdirectory
        if (tabPath.startsWith(sourcePath + '\\') || tabPath.startsWith(sourcePath + '/')) {
          alert(`Cannot move "${fileName}" into its own subdirectory`);
          continue;
        }
        
        console.log(`📦 Moving: ${sourcePath} -> ${tabPath}`);
        await invoke('move_item', { source: sourcePath, destination: tabPath });
      }
      
      // Refresh the current directory by triggering a re-load
      // We do this by "updating" to the same path which will re-trigger the file load
      fileTabs.updateActivePath(currentPath);
    } catch (err) {
      console.error("❌ Move error:", err);
      alert("Move failed: " + err);
    }
  }
</script>

<div class="tabs-container" on:wheel={handleWheel} data-tauri-drag-region>
  {#each $fileTabs.tabs as tab}
    <div 
      class="tab" 
      class:active={$fileTabs.activeId === tab.id}
      class:drag-over={dragOverTabId === tab.id}
      on:click={() => fileTabs.setActive(tab.id)}
      on:dblclick={() => startEditing(tab.id)}
      on:keydown={(e) => handleTabKeyDown(e, tab.id)}
      on:dragenter={(e) => handleTabDragEnter(e, tab.id)}
      on:dragover={(e) => handleTabDragOver(e, tab.id)}
      on:dragleave={handleTabDragLeave}
      on:drop={(e) => handleTabDrop(e, tab.id, tab.path)}
      role="button"
      tabindex="0"
    >
      {#if editingId === tab.id}
        <input 
          bind:this={editInput}
          type="text" 
          value={tab.name} 
          class="tab-edit-input"
          on:blur={(e) => finishEditing(tab.id, e.currentTarget.value)}
          on:keydown={(e) => handleKey(e, tab.id, e.currentTarget.value)}
          on:click|stopPropagation
        />
      {:else}
        <span class="tab-name">{tab.name}</span>
      {/if}
    
      {#if $fileTabs.tabs.length > 1}
        <span 
          class="close-btn" 
          on:click={(e) => close(e, tab.id)}
          on:keydown={(e) => handleCloseKeyDown(e, tab.id)}
          role="button" 
          tabindex="0"
          aria-label="Close Tab"
        >×</span>
      {/if}
    </div>
  {/each}
  
  <button class="new-tab-btn" on:click={() => fileTabs.addTab()} aria-label="New Tab">+</button>
</div>

<style>
  .tabs-container {
    display: flex;
    align-items: flex-end;
    height: 100%;
    padding-left: 0;
    -webkit-app-region: no-drag;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }
  
  .tabs-container::-webkit-scrollbar { 
    display: none;
  }

  .tab {
    position: relative;
    height: 100%;
    min-width: 100px;
    max-width: 160px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
    background: transparent;
    color: var(--tab-inactive-fg);
    cursor: pointer;
    font-size: 12px;
    user-select: none;
    border-top: 2px solid transparent;
    border-right: 1px solid var(--border);
    transition: background 0.1s;
    flex-shrink: 0; 
  }

  .tab:hover:not(.active) {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .tab.active {
    background: var(--bg-main);
    color: var(--tab-active-fg);
    border-top: 2px solid var(--tab-active-border);
  }

  .tab.drag-over {
    background: rgba(59, 130, 246, 0.3);
    border-top: 2px solid #3b82f6;
    outline: 2px dashed #3b82f6;
    outline-offset: -2px;
  }

  .tab-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-right: 8px;
    flex: 1;
  }

  .tab-edit-input {
    width: 100%;
    height: 22px;
    background: var(--bg-input);
    border: 1px solid var(--border-focus);
    color: var(--text-main);
    font-size: 12px;
    outline: none;
    padding: 0 4px;
    border-radius: 2px;
  }

  .close-btn {
    font-size: 16px;
    line-height: 1;
    color: #666;
    padding: 2px 4px;
    border-radius: 4px;
    opacity: 0; 
    transition: opacity 0.1s;
  }

  .tab:hover .close-btn, .tab.active .close-btn {
    opacity: 1;
  }

  .close-btn:hover {
    background: #cc3333;
    color: white;
  }

  .new-tab-btn {
    width: 32px;
    height: 100%;
    background: transparent;
    border: none;
    color: #888;
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0; 
  }
  .new-tab-btn:hover { color: white; background: rgba(255,255,255,0.05); }
</style>