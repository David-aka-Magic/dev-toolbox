<script lang="ts">
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { tick } from 'svelte';

  let editingId: string | null = null;
  let editInput: HTMLInputElement;
  let editValue = "";

  // SCROLL HANDLER
  function handleWheel(e: WheelEvent) {
    const container = e.currentTarget as HTMLElement;
    if (e.deltaY !== 0) {
      container.scrollLeft += e.deltaY;
      e.preventDefault(); 
    }
  }

  function close(e: Event, id: string) {
    e.stopPropagation();
    const tab = $editorTabs.find(t => t.id === id);
    if (tab?.isDirty) {
      if (!confirm(`Close "${tab.name}" without saving?`)) return;
    }
    
    if ($editorTabs.length === 1) return;
    
    const remaining = $editorTabs.filter(t => t.id !== id);
    editorTabs.set(remaining);
    
    if ($activeEditorTabId === id) {
      activeEditorTabId.set(remaining[remaining.length - 1].id);
    }
  }

  async function startEditing(id: string, currentName: string) {
    editingId = id;
    editValue = currentName;
    await tick();
    editInput?.focus();
    editInput?.select();
  }

  function submitEdit() {
    if (editingId && editValue.trim()) {
      editorTabs.update(tabs => 
        tabs.map(t => t.id === editingId ? { ...t, name: editValue.trim() } : t)
      );
    }
    editingId = null;
  }

  function handleEditKey(e: KeyboardEvent) {
    if (e.key === 'Enter') submitEdit();
    if (e.key === 'Escape') editingId = null;
  }

  function handleTabKeyDown(e: KeyboardEvent, id: string) {
    if (e.key === 'Enter' || e.key === ' ') {
      activeEditorTabId.set(id);
    }
  }

  function addTab() {
    const newTab = {
      id: crypto.randomUUID(),
      name: 'Untitled',
      path: null,
      content: '',
      isDirty: false
    };
    editorTabs.update(tabs => [...tabs, newTab]);
    activeEditorTabId.set(newTab.id);
  }
</script>

<div class="tabs-wrapper" data-tauri-drag-region>
  
  <div class="scroll-area" on:wheel={handleWheel} data-tauri-drag-region>
    {#each $editorTabs as tab}
      <div 
        class="tab" 
        class:active={$activeEditorTabId === tab.id}
        on:click={() => activeEditorTabId.set(tab.id)}
        on:dblclick={() => startEditing(tab.id, tab.name)}
        on:keydown={(e) => handleTabKeyDown(e, tab.id)} 
        role="button"
        tabindex="0"
      >
        {#if editingId === tab.id}
          <input 
            bind:this={editInput}
            type="text" 
            class="rename-input"
            bind:value={editValue}
            on:blur={submitEdit}
            on:keydown={handleEditKey}
            on:click|stopPropagation
          />
        {:else}
          <span class="tab-name">{tab.name}{tab.isDirty ? '*' : ''}</span>
          {#if $editorTabs.length > 1}
            <span 
              class="close-btn" 
              on:click={(e) => close(e, tab.id)} 
              role="button" 
              tabindex="0"
            >×</span>
          {/if}
        {/if}
      </div>
    {/each}
  </div>

  <div class="static-area" data-tauri-drag-region>
    <button class="add-btn" on:click={addTab}>+</button>
  </div>

</div>

<style>
  .tabs-wrapper {
    display: flex;
    height: 100%;
    width: 100%;
    align-items: flex-end;
    padding-left: 0;
    min-width: 0; 
  }

  .scroll-area {
    display: flex;
    height: 100%;
    align-items: flex-end; 
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none; 
    flex-wrap: nowrap; 
    flex: 0 1 auto;
    min-width: 0; 
    max-width: calc(100% - 32px);
  }
  .scroll-area::-webkit-scrollbar { display: none; }

  .static-area {
    flex: 0 0 32px;
    height: 100%;
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
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
    cursor: pointer;
    font-size: 12px;
    user-select: none;
    border-top: 2px solid transparent; 
    border-right: 1px solid var(--border);
    background: transparent;
    color: var(--tab-inactive-fg);
    transition: background 0.1s;
    flex-shrink: 0; 
    -webkit-app-region: no-drag; 
  }

  .tab:hover:not(.active) { 
    background: var(--hover-bg); 
    color: var(--text-main); 
  }

  .tab.active {
    background: var(--bg-main);
    color: var(--tab-active-fg);
    border-top: 2px solid var(--tab-active-border);
    border-bottom: 1px solid transparent;
  }

  .tab-name {
    white-space: nowrap; 
    overflow: hidden; 
    text-overflow: ellipsis;
    margin-right: 8px; 
    flex: 1; 
    pointer-events: none;
  }

  .rename-input {
    width: 100%; 
    height: 20px; 
    font-size: 12px;
    background: var(--bg-input); 
    color: var(--text-main);
    border: 1px solid var(--border-focus); 
    border-radius: 2px;
    padding: 0 4px; 
    outline: none; 
    -webkit-app-region: no-drag;
  }

  .close-btn {
    font-size: 16px; 
    line-height: 1; 
    color: var(--text-muted);
    padding: 2px 4px; 
    border-radius: 4px; 
    opacity: 0; 
    transition: opacity 0.1s;
    -webkit-app-region: no-drag;
  }
  .tab:hover .close-btn, .tab.active .close-btn { opacity: 1; }
  .close-btn:hover { background: #cc3333; color: white; }

  .add-btn {
    width: 100%; 
    height: 100%; 
    background: transparent; 
    border: none;
    color: var(--text-muted); 
    font-size: 18px;
    cursor: pointer; 
    display: flex; 
    align-items: center; 
    justify-content: center;
    -webkit-app-region: no-drag;
  }
  .add-btn:hover { 
    color: var(--text-main); 
    background: var(--hover-bg); 
  }
</style>