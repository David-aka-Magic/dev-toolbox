<script lang="ts">
  import { viewMode, type ViewMode } from '$lib/stores/viewModeStore';
  
  const modes: { value: ViewMode; icon: string; label: string }[] = [
    { value: 'grid', icon: 'grid', label: 'Grid View' },
    { value: 'list', icon: 'list', label: 'List View' },
    { value: 'details', icon: 'details', label: 'Details View' }
  ];
</script>

<div class="view-mode-selector">
  {#each modes as mode}
    <button
      class="mode-btn"
      class:active={$viewMode === mode.value}
      on:click={() => $viewMode = mode.value}
      title={mode.label}
    >
      {#if mode.icon === 'grid'}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7"></rect>
          <rect x="14" y="3" width="7" height="7"></rect>
          <rect x="14" y="14" width="7" height="7"></rect>
          <rect x="3" y="14" width="7" height="7"></rect>
        </svg>
      {:else if mode.icon === 'list'}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"></line>
          <line x1="8" y1="12" x2="21" y2="12"></line>
          <line x1="8" y1="18" x2="21" y2="18"></line>
          <line x1="3" y1="6" x2="3.01" y2="6"></line>
          <line x1="3" y1="12" x2="3.01" y2="12"></line>
          <line x1="3" y1="18" x2="3.01" y2="18"></line>
        </svg>
      {:else if mode.icon === 'details'}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="4"></rect>
          <rect x="3" y="10" width="18" height="4"></rect>
          <rect x="3" y="17" width="18" height="4"></rect>
        </svg>
      {/if}
    </button>
  {/each}
</div>

<style>
  .view-mode-selector {
    display: flex;
    gap: 2px;
    background: var(--bg-input);
    border-radius: 4px;
    padding: 2px;
  }

  .mode-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    width: 26px;
    height: 22px;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .mode-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .mode-btn.active {
    background: var(--selection);
    color: var(--border-focus);
  }
</style>