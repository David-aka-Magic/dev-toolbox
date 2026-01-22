<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { currentView } from '$lib/stores/viewStore';
  
  import TerminalTabs from '../terminal/TerminalTabs.svelte';
  import AddressBar from '../filesystem/AddressBar.svelte';
  import FileTabs from '../filesystem/FileTabs.svelte';
  import EditorTabs from '../editor/EditorTabs.svelte';

  const appWindow = getCurrentWindow();
  function minimize() { appWindow.minimize(); }
  function maximize() { appWindow.toggleMaximize(); }
  function closeWindow() { appWindow.close(); }
</script>

<div class="drag-handle-top" data-tauri-drag-region></div>

<div class="titlebar" data-tauri-drag-region>
  <div class="logo-area" data-tauri-drag-region>
    <span class="logo">DT</span>
  </div>
   
  <div class="center-content" data-tauri-drag-region>
    
    {#if $currentView === 'terminal'}
      <TerminalTabs />
      
    {:else if $currentView === 'files'}
      <div class="split-header" data-tauri-drag-region>
        <div class="tabs-section" data-tauri-drag-region>
            <FileTabs />
        </div>
        <div class="drag-spacer" data-tauri-drag-region></div>
        
        <div class="address-section">
           <AddressBar />
        </div>
      </div>

    {:else if $currentView === 'editor'}
      <EditorTabs />

    {:else}
      <div class="spacer" data-tauri-drag-region>
        <span class="view-title">{$currentView.toUpperCase()}</span>
      </div>
    {/if}

  </div>

  <div class="window-controls">
    <button class="control-btn" on:click={minimize} aria-label="Minimize">
      <svg width="10" height="1" viewBox="0 0 10 1"><path d="M0 0h10v1H0z" fill="currentColor"/></svg>
    </button>
    
    <button class="control-btn" on:click={maximize} aria-label="Maximize">
     <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1h8v8H1V1zm1 1v6h6V2H2z" fill="currentColor"/></svg>
    </button>
    
    <button class="control-btn close-btn" on:click={closeWindow} aria-label="Close">
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 0L0 1l4 4-4 4 1 1 4-4 4 4 1-1-4-4 4-4-1-1-4 4-4-4z" fill="currentColor"/></svg>
    </button>
  </div>
</div>

<style>
  .drag-handle-top {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 6px;
    z-index: 9999;
    -webkit-app-region: drag;
    background: transparent;
  }

  .titlebar {
    height: 38px;
    background: var(--bg-header);
    display: flex;
    justify-content: space-between;
    align-items: stretch;
    border-bottom: 1px solid var(--border);
    width: 100%;
    position: relative; 
    z-index: 50; 
    padding-top: 4px;
    box-sizing: border-box;
  }

  .logo-area { 
    display: flex; 
    align-items: center; 
    width: 48px; 
    justify-content: center; 
    flex-shrink: 0; 
    -webkit-app-region: drag;
  }
  .logo { font-weight: bold; color: #3b82f6; }

  .center-content {
    flex: 1;
    display: flex;
    align-items: center;
    min-width: 0;
    overflow: visible; 
    -webkit-app-region: drag;
    padding-right: 10px;
  }

  .split-header {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
  }
  
  .tabs-section {
    flex: 0 1 auto;
    min-width: 150px;
    max-width: 50%;
    height: 100%;
    display: flex;
    align-items: flex-end;
  }
  
  .drag-spacer {
    flex: 0 0 2px;
    height: 100%;
    -webkit-app-region: drag;
  }
  
  .address-section {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    min-width: 100px;
    padding: 4px 0; 
  }

  .spacer { width: 100%; display: flex; align-items: center; justify-content: center; }
  .view-title { color: #52525b; font-size: 12px; font-weight: 600; }
  
  .window-controls { 
    display: flex; 
    -webkit-app-region: no-drag; 
    flex-shrink: 0;
    position: relative;
    z-index: 10000; /* Higher than drag-handle-top */
  }
  
  .control-btn { 
    width: 46px; 
    height: 100%; 
    background: transparent; 
    border: none; 
    color: var(--text-muted); 
    cursor: pointer; 
    display: flex; 
    align-items: center; 
    justify-content: center; 
    -webkit-app-region: no-drag;
  }
  .control-btn:hover { background: var(--hover-bg); color:var(--text-main); }
  .close-btn:hover { background: #ef4444; color: white; }
</style>