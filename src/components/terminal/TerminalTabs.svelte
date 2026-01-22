<script lang="ts">
  import { terminals, activeTabId, createTerminal, closeTerminal, setActiveTerminal, renameTerminal } from '$lib/stores/terminalStore';
  import { fade } from 'svelte/transition';
  import { tick } from 'svelte';

  // --- MENU STATE ---
  let showProfileMenu = false;
  let addBtn: HTMLButtonElement;

  // --- RENAME STATE ---
  let editingId: string | null = null;
  let editInput: HTMLInputElement;
  let editValue = "";

  const profiles = [
    { id: 'pwsh', label: 'PowerShell', icon: '⚡' },
    { id: 'cmd', label: 'Command Prompt', icon: '💻' },
    { id: 'git-bash', label: 'Git Bash', icon: '🌲' },
    { id: 'wsl', label: 'WSL', icon: '🐧' },
  ];

  // SCROLL HANDLER
  function handleWheel(e: WheelEvent) {
    const container = e.currentTarget as HTMLElement;
    if (e.deltaY !== 0) {
      container.scrollLeft += e.deltaY;
      e.preventDefault(); 
    }
  }

  function toggleMenu() { showProfileMenu = !showProfileMenu; }

  function handleCreate(profile: string) {
    createTerminal(profile);
    showProfileMenu = false;
  }

  function close(e: Event, id: string) {
    e.stopPropagation();
    closeTerminal(id);
  }

  function handleWindowClick(e: MouseEvent) {
    if (showProfileMenu && addBtn && !addBtn.contains(e.target as Node)) {
      showProfileMenu = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent, id: string) {
    if (e.key === 'Enter' || e.key === ' ') setActiveTerminal(id);
  }

  // --- RENAME HANDLERS ---
  async function startEditing(id: string, currentTitle: string) {
    editingId = id;
    editValue = currentTitle;
    await tick();
    editInput?.focus();
    editInput?.select();
  }

  function submitEdit() {
    if (editingId && editValue.trim()) {
      renameTerminal(editingId, editValue.trim());
    }
    editingId = null;
  }

  function handleEditKey(e: KeyboardEvent) {
    if (e.key === 'Enter') submitEdit();
    if (e.key === 'Escape') editingId = null;
  }
</script>

<svelte:window on:click={handleWindowClick} />

<div class="tabs-wrapper" data-tauri-drag-region>
  
  <div class="scroll-area" on:wheel={handleWheel} data-tauri-drag-region>
    {#each $terminals as term}
      <div 
        class="tab" 
        class:active={$activeTabId === term.id}
        on:click={() => setActiveTerminal(term.id)}
        on:dblclick={() => startEditing(term.id, term.title)}
        on:keydown={(e) => handleKeyDown(e, term.id)} 
        role="button"
        tabindex="0"
      >
        {#if editingId === term.id}
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
          <span class="tab-name">{term.title}</span>
          <span 
              class="close-btn" 
              on:click={(e) => close(e, term.id)} 
              role="button" 
              tabindex="0"
          >×</span>
        {/if}
      </div>
    {/each}
  </div>

  <div class="static-area" data-tauri-drag-region>
    <div class="add-wrapper" data-tauri-drag-region>
      <button 
        bind:this={addBtn}
        class="add-btn" 
        on:click|stopPropagation={toggleMenu} 
        class:active={showProfileMenu}
      >+</button>
  
      {#if showProfileMenu}
        <div class="profile-menu" transition:fade={{ duration: 100 }}>
          {#each profiles as p}
            <button class="menu-item" on:click={() => handleCreate(p.id)}>
              <span class="menu-icon">{p.icon}</span>
              {p.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

</div>

<style>
  .tabs-wrapper {
    display: flex;
    height: 100%;
    width: 100%;
    align-items: flex-end;
    padding-left: 0;
    
    /* Allows this container to shrink below its content size */
    min-width: 0; 
  }

  .scroll-area {
    display: flex;
    height: 100%;
    align-items: flex-end; 
    
    /* SCROLL LOGIC */
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none; 
    flex-wrap: nowrap; 

    /* SIZE LOGIC - CHANGED HERE */
    flex: 0 1 auto; /* Don't grow to fill space, just fit content */
    
    min-width: 0; 
    max-width: calc(100% - 32px); /* Reserve space for + button */
  }
  .scroll-area::-webkit-scrollbar { display: none; }

  .static-area {
    flex: 0 0 32px;
    height: 100%;
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
    overflow: visible; 
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

  .tab:hover:not(.active) { background: var(--hover-bg); color: var(--text-main); }

  .tab.active {
    background: var(--bg-main);
    color: var(--tab-active-fg);
    border-top: 2px solid var(--tab-active-border);
    border-bottom: 1px solid transparent;
  }

  .tab-name {
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    margin-right: 8px; flex: 1; pointer-events: none;
  }

  .rename-input {
    width: 100%; height: 20px; font-size: 12px;
    background: var(--bg-input); color: var(--text-main);
    border: 1px solid var(--border-focus); border-radius: 2px;
    padding: 0 4px; outline: none; -webkit-app-region: no-drag;
  }

  .close-btn {
    font-size: 16px; line-height: 1; color: var(--text-muted);
    padding: 2px 4px; border-radius: 4px; opacity: 0; transition: opacity 0.1s;
    -webkit-app-region: no-drag;
  }
  .tab:hover .close-btn, .tab.active .close-btn { opacity: 1; }
  .close-btn:hover { background: #cc3333; color: white; }

  .add-wrapper {
    position: relative; height: 100%; width: 100%;
    display: flex; align-items: center;
  }

  .add-btn {
    width: 100%; height: 100%; background: transparent; border: none;
    color: var(--text-muted); font-size: 18px;
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    -webkit-app-region: no-drag;
  }
  .add-btn:hover, .add-btn.active { color: var(--text-main); background: var(--hover-bg); }

  .profile-menu {
    position: absolute; top: 100%; left: 0; width: 160px;
    background: var(--bg-panel); border: 1px solid var(--border);
    box-shadow: 0 4px 6px rgba(0,0,0,0.3); border-radius: 0 0 4px 4px;
    z-index: 9999; display: flex; flex-direction: column; padding: 4px 0;
  }

  .menu-item {
    background: transparent; border: none; color: var(--text-main);
    padding: 8px 12px; text-align: left; cursor: pointer;
    display: flex; align-items: center; font-size: 13px; gap: 8px;
  }
  .menu-item:hover { background-color: var(--hover-bg); }
  .menu-icon { width: 16px; display: inline-block; text-align: center; }
</style>