<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import FileNode from "./FileNode.svelte";
  import { toggleTheme } from "$lib/stores/theme";
  import { currentPath } from "$lib/stores/path";

  let rootFiles: any[] = [];
  let unsubscribePath: () => void;
  
  // Default to visible (Open)
  let isVisible = true;

  async function loadRoot(path: string) {
    try {
      rootFiles = await invoke("read_directory", { path });
    } catch (err) {
      console.error("Error loading root:", err);
    }
  }

  function toggleTree() {
    isVisible = !isVisible;
  }

  onMount(() => {
    unsubscribePath = currentPath.subscribe((newPath) => {
      loadRoot(newPath);
    });
  });

  onDestroy(() => {
    if (unsubscribePath) unsubscribePath();
  });
</script>

<div class="tree-container">
  
  <div class="toggle-strip" on:click={toggleTree} role="button" tabindex="0" on:keydown={(e) => e.key === 'Enter' && toggleTree()}>
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

  <div class="content-wrapper" class:collapsed={!isVisible}>
    
    <div class="fixed-inner">
      <div class="tree-header">
        <span class="header-label">Explorer</span>
      </div>

      <div class="tree-list">
        {#each rootFiles as file}
          <FileNode 
            name={file.name} 
            path={file.path} 
            isDir={file.is_dir} 
          />
        {/each}
      </div>

      <footer class="tree-footer">
        <button class="footer-btn" on:click={toggleTheme}>
          <span class="theme-icon">🌓</span>
          Theme
        </button>
      </footer>
    </div>
    
  </div>

</div>

<style>
  /* ROOT: Layout side-by-side */
  .tree-container {
    height: 100%; 
    display: flex; 
    flex-direction: row; 
    background-color: var(--bg-panel); 
    color: var(--text-main);
    /* No fixed width here; let children dictate width */
  }

  /* --- LEFT STRIP --- */
  .toggle-strip {
    width: 20px; /* Skinny clickable spine */
    height: 100%;
    display: flex;
    align-items: center; /* Center Vertically */
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
    color: var(--text-main);
  }
  
  /* --- RIGHT CONTENT WRAPPER --- */
  .content-wrapper {
    width: 250px; /* Default Open Width */
    height: 100%;
    overflow: hidden; /* Hide content when shrinking */
    transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1); /* Smooth easing */
    background-color: var(--bg-panel); 
  }

  .content-wrapper.collapsed {
    width: 0px; /* Shrink to nothing */
  }

  /* --- FIXED INNER CONTENT --- */
  /* Keeps layout stable while wrapper clips it */
  .fixed-inner {
    width: 250px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  /* --- EXISTING STYLES --- */
  .tree-header {
    height: 35px; display: flex; align-items: center; padding: 0 12px;
    background-color: var(--bg-panel); 
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-label { font-weight: bold; font-size: 10px; text-transform: uppercase; opacity: 0.6; letter-spacing: 0.5px; white-space: nowrap; }

  .tree-list { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; padding: 5px; }
  .tree-list::-webkit-scrollbar { width: 8px; }
  .tree-list::-webkit-scrollbar-thumb { background-color: var(--border); border-radius: 4px; border: 2px solid var(--bg-panel); }

  .tree-footer {
    height: 32px; border-top: 1px solid var(--border);
    display: flex; align-items: center; padding: 0 8px;
    background-color: var(--bg-panel); 
    flex-shrink: 0;
  }
  
  .footer-btn { background: none; border: none; color: var(--text-main); opacity: 0.7; font-size: 11px; cursor: pointer; display: flex;
    align-items: center; gap: 8px; width: 100%; height: 100%; white-space: nowrap; }
  .footer-btn:hover { opacity: 1; background-color: var(--hover-bg); }
  .theme-icon { font-size: 12px; }
</style>