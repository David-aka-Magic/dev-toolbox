<script lang="ts">
  import { currentView } from '$lib/stores/viewStore';
</script>

<div class="navbar">
  <button 
    class:active={$currentView === 'terminal'} 
    on:click={() => $currentView = 'terminal'}
    title="Terminal"
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  </button>

  <button 
    class:active={$currentView === 'files'} 
    on:click={() => $currentView = 'files'}
    title="File Manager"
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
    </svg>
  </button>

  <button 
    class:active={$currentView === 'editor'} 
    on:click={() => $currentView = 'editor'}
    title="Text Editor"
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
    </svg>
  </button>
</div>

<style>
  .navbar {
    width: 48px;
    height: 100%;
    background: var(--bg-header); 
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 10px;
    gap: 10px;
    transition: background 0.3s;
  }

  button {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 10px;
    border-radius: 4px;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: color 0.2s;
  }

  button:hover {
    color: var(--text-main);
    background: var(--hover-bg);
  }

  button.active {
    color: var(--border-focus);
    border-left: 3px solid var(--border-focus);
    background: var(--hover-bg);
  }
</style>