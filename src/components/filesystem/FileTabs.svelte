<script lang="ts">
  import { fileTabs } from '$lib/stores/fileTabStore';
  import { tick } from 'svelte';

  let editingId: string | null = null;
  let editInput: HTMLInputElement;

  // NEW: Scroll tabs horizontally with mouse wheel
  function handleWheel(e: WheelEvent) {
    const container = e.currentTarget as HTMLElement;
    if (e.deltaY !== 0) {
      container.scrollLeft += e.deltaY;
      // Prevent the page/sidebar from scrolling while hovering tabs
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
</script>

<div class="tabs-container" on:wheel={handleWheel} data-tauri-drag-region>
  {#each $fileTabs.tabs as tab}
    <div 
      class="tab" 
      class:active={$fileTabs.activeId === tab.id}
      on:click={() => fileTabs.setActive(tab.id)}
      on:dblclick={() => startEditing(tab.id)}
      on:keydown={(e) => handleTabKeyDown(e, tab.id)}
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
    
    /* SCROLL LOGIC */
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none; /* Firefox */
  }
  
  /* Hide scrollbar for WebKit */
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
    
    /* CRITICAL FIX: Prevent tabs from shrinking, forcing the scroll */
    flex-shrink: 0; 
  }

  .tab:hover:not(.active) {
    background: var(--hover-bg);
    color:var(--text-main);
  }

  .tab.active {
    background: var(--bg-main);
    color: var(--tab-active-fg);
    border-top: 2px solid var(--tab-active-border);
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
    /* Ensure the + button also doesn't shrink */
    flex-shrink: 0; 
  }
  .new-tab-btn:hover { color: white; background: rgba(255,255,255,0.05); }
</style>