<script lang="ts">
  export let title: string = "Sidebar";
  export let width: number = 250;
  export let isVisible: boolean = true;
  
  // Optional: Allow parent to control visibility
  export let onToggle: ((visible: boolean) => void) | undefined = undefined;

  function toggle() {
    isVisible = !isVisible;
    onToggle?.(isVisible);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      toggle();
    }
  }
</script>

<div class="sidebar-container">
  
  <!-- Toggle Strip -->
  <div 
    class="toggle-strip" 
    on:click={toggle} 
    on:keydown={handleKeydown}
    role="button" 
    tabindex="0"
    aria-label={isVisible ? "Collapse sidebar" : "Expand sidebar"}
  >
    <div class="arrow-icon">
      {#if isVisible}
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      {/if}
    </div>
  </div>

  <!-- Content Wrapper (animates width) -->
  <div class="content-wrapper" class:collapsed={!isVisible}>
    
    <div class="fixed-inner" style="width: {width}px;">
      
      <!-- Header -->
      <div class="sidebar-header">
        <span class="header-label">{title}</span>
        <slot name="header-actions" />
      </div>

      <!-- Main Content -->
      <div class="sidebar-content">
        <slot />
      </div>

      <!-- Footer -->
      {#if $$slots.footer}
        <footer class="sidebar-footer">
          <slot name="footer" />
        </footer>
      {/if}
      
    </div>
    
  </div>

</div>

<style>
  .sidebar-container {
    height: 100%; 
    display: flex; 
    flex-direction: row; 
    background-color: var(--bg-panel); 
    color: var(--text-main);
  }

  /* --- Toggle Strip --- */
  .toggle-strip {
    width: 20px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-right: 1px solid var(--border);
    flex-shrink: 0;
    transition: background-color 0.2s;
    background-color: var(--bg-panel);
    z-index: 10;
  }
  
  .toggle-strip:hover {
    background-color: var(--hover-bg);
  }

  .toggle-strip:focus {
    outline: none;
    background-color: var(--hover-bg);
  }

  .arrow-icon {
    opacity: 0.5;
    transition: opacity 0.2s;
  }

  .toggle-strip:hover .arrow-icon {
    opacity: 1;
  }
  
  /* --- Content Wrapper --- */
  .content-wrapper {
    width: var(--sidebar-width, 250px);
    height: 100%;
    overflow: hidden;
    transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    background-color: var(--bg-panel); 
  }

  .content-wrapper.collapsed {
    width: 0px;
  }

  /* --- Fixed Inner Content --- */
  .fixed-inner {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  /* --- Header --- */
  .sidebar-header {
    height: 35px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background-color: var(--bg-panel); 
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-label {
    font-weight: bold;
    font-size: 10px;
    text-transform: uppercase;
    opacity: 0.6;
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  /* --- Content --- */
  .sidebar-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .sidebar-content::-webkit-scrollbar {
    width: 8px;
  }

  .sidebar-content::-webkit-scrollbar-thumb {
    background-color: var(--border);
    border-radius: 4px;
    border: 2px solid var(--bg-panel);
  }

  .sidebar-content::-webkit-scrollbar-thumb:hover {
    background-color: var(--text-muted);
  }

  /* --- Footer --- */
  .sidebar-footer {
    height: 32px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    padding: 0 8px;
    background-color: var(--bg-panel); 
    flex-shrink: 0;
  }
</style>