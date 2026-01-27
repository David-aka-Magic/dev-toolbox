<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let open = false;
  
  const dispatch = createEventDispatcher();
  
  const commands = [
    { name: 'Save File', shortcut: 'Ctrl+S', action: 'save' },
    { name: 'Open File', shortcut: 'Ctrl+O', action: 'open' },
    { name: 'Toggle Preview', shortcut: 'Ctrl+Shift+P', action: 'togglePreview' },
    { name: 'Toggle Sidebar', shortcut: 'Ctrl+B', action: 'toggleSidebar' },
    { name: 'Find', shortcut: 'Ctrl+F', action: 'find' },
    { name: 'Replace', shortcut: 'Ctrl+H', action: 'replace' },
    { name: 'Jump to Line', shortcut: 'Ctrl+G', action: 'jumpToLine' },
    { name: 'Command Palette', shortcut: 'Ctrl+K', action: 'commandPalette' },
    { name: 'Bold', shortcut: 'Ctrl+B', action: 'bold' },
    { name: 'Italic', shortcut: 'Ctrl+I', action: 'italic' },
    { name: 'Undo', shortcut: 'Ctrl+Z', action: 'undo' },
    { name: 'Redo', shortcut: 'Ctrl+Shift+Z', action: 'redo' },
  ];
  
  let searchQuery = '';
  $: filteredCommands = commands.filter(cmd => 
    cmd.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    cmd.shortcut.toLowerCase().includes(searchQuery.toLowerCase())
  );
  
  function executeCommand(action: string) {
    dispatch('command', { action });
    close();
  }
  
  function close() {
    open = false;
    searchQuery = '';
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

{#if open}
  <div class="backdrop" on:click={close}>
    <div class="palette" on:click|stopPropagation on:keydown={handleKeydown}>
      <div class="search-box">
        <input 
          type="text" 
          placeholder="Type a command or search..." 
          bind:value={searchQuery}
          autofocus
        />
      </div>
      
      <div class="commands-list">
        {#each filteredCommands as cmd}
          <button class="command-item" on:click={() => executeCommand(cmd.action)}>
            <span class="command-name">{cmd.name}</span>
            <span class="command-shortcut">{cmd.shortcut}</span>
          </button>
        {/each}
        
        {#if filteredCommands.length === 0}
          <div class="no-results">No commands found</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 10000;
  }
  
  .palette {
    width: 600px;
    max-height: 400px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .search-box {
    padding: 12px;
    border-bottom: 1px solid var(--border);
  }
  
  .search-box input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-main);
    font-size: 14px;
    outline: none;
  }
  
  .search-box input:focus {
    border-color: var(--border-focus);
  }
  
  .commands-list {
    overflow-y: auto;
    max-height: 320px;
  }
  
  .command-item {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-main);
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }
  
  .command-item:last-child {
    border-bottom: none;
  }
  
  .command-item:hover {
    background: var(--hover-bg);
  }
  
  .command-name {
    font-size: 13px;
  }
  
  .command-shortcut {
    font-size: 11px;
    color: var(--text-muted);
    font-family: 'Fira Code', monospace;
    background: var(--bg-main);
    padding: 2px 6px;
    border-radius: 3px;
  }
  
  .no-results {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>