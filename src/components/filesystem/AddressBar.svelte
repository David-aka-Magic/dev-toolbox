<script lang="ts">
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { tick } from "svelte";
  import ViewModeSelector from './ViewModeSelector.svelte';

  let isInputMode = false;
  let inputElement: HTMLInputElement;
  let rawPath = "";
  let dragOverCrumb: number | null = null;

  $: if ($activeTab) { rawPath = $activeTab.path; }
  $: parts = rawPath ? rawPath.split(/[\\/]/).filter(p => p !== "") : [];

  function navigateTo(index: number) {
    let newPath = parts.slice(0, index + 1).join("\\");
    if (index === 0 && newPath.endsWith(":")) newPath += "\\";
    fileTabs.updateActivePath(newPath);
  }

  function getPathUpTo(index: number): string {
    let newPath = parts.slice(0, index + 1).join("\\");
    if (index === 0 && newPath.endsWith(":")) newPath += "\\";
    return newPath;
  }

  async function enableInput() {
    isInputMode = true;
    await tick();
    inputElement?.focus();
    inputElement?.select();
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Enter") { fileTabs.updateActivePath(rawPath); isInputMode = false; }
    if (e.key === "Escape") { rawPath = $activeTab?.path || ""; isInputMode = false; }
  }
  function handleBlur() { setTimeout(() => { isInputMode = false; }, 150); }

  function handleCrumbDragEnter(event: DragEvent, index: number) {
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    dragOverCrumb = index;
  }

  function handleCrumbDragOver(event: DragEvent, index: number) {
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  function handleCrumbDragLeave(event: DragEvent) {
    dragOverCrumb = null;
  }

  async function handleCrumbDrop(event: DragEvent, index: number) {
    event.preventDefault();
    event.stopPropagation();
    dragOverCrumb = null;

    const state = $fileDragDrop;
    if (!state.draggedFile) return;

    const destPath = getPathUpTo(index);
    const currentPath = $activeTab?.path;
    
    if (!currentPath) return;
    
    if (destPath === currentPath) {
      console.log("⚠️ Cannot drop into the same folder");
      return;
    }

    console.log(`📦 Moving ${state.draggedFiles.length} file(s) to breadcrumb:`, destPath);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { directoryCache } = await import('$lib/stores/directoryCacheStore');
      
      for (const fileName of state.draggedFiles) {
        const sourcePath = `${currentPath}\\${fileName}`;
        
        if (destPath.startsWith(sourcePath + '\\') || destPath.startsWith(sourcePath + '/')) {
          alert(`Cannot move "${fileName}" into its own subdirectory`);
          continue;
        }
        
        console.log(`📦 Moving: ${sourcePath} -> ${destPath}`);
        await invoke('move_item', { src: sourcePath, dest: destPath });
      }
      
      directoryCache.invalidate(currentPath);
      directoryCache.invalidate(destPath);
      window.dispatchEvent(new CustomEvent('force-file-refresh'));
      fileDragDrop.handleDragEnd();
    } catch (err) {
      console.error("❌ Move error:", err);
      alert("Move failed: " + err);
    }
  }
</script>

<div class="address-container">
    <div class="address-bar">
      {#if isInputMode}
        <input 
          bind:this={inputElement}
          type="text"
          bind:value={rawPath}
          on:keydown={handleKey}
          on:blur={handleBlur}
          class="path-input"
        />
      {:else}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <div class="breadcrumbs" on:click={enableInput} role="textbox" tabindex="0">
          {#each parts as part, i}
            <button 
              class="crumb" 
              class:drag-over={dragOverCrumb === i}
              on:click|stopPropagation={() => navigateTo(i)}
              on:dragenter={(e) => handleCrumbDragEnter(e, i)}
              on:dragover={(e) => handleCrumbDragOver(e, i)}
              on:dragleave={handleCrumbDragLeave}
              on:drop={(e) => handleCrumbDrop(e, i)}
            >
              {part}
            </button>
            <span class="divider">›</span>
          {/each}
          <div class="spacer"></div>
        </div>
      {/if}
    </div>

    <ViewModeSelector />

    <div class="nav-controls">
      <button class="nav-btn" disabled={!$activeTab || $activeTab.historyIndex <= 0} on:click={() => fileTabs.goBack()}>&lt;</button>
      <button class="nav-btn" disabled={!$activeTab || $activeTab.historyIndex >= $activeTab.history.length - 1} on:click={() => fileTabs.goForward()}>&gt;</button>
    </div>
</div>

<style>
  .address-container { 
    display: flex; 
    align-items: center; 
    width: 100%; 
    height: 100%; 
    padding-right: 10px; 
    gap: 8px;
  }
  .nav-controls { display: flex; gap: 2px; margin-left: 6px; -webkit-app-region: no-drag; }
  
  .nav-btn { 
    background: transparent; border: none; 
    color: var(--text-muted);
    width: 24px; height: 24px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-weight: bold; 
  }
  .nav-btn:hover:not(:disabled) { 
    background: var(--hover-bg);
    color: var(--text-main);
  }
  .nav-btn:disabled { color: var(--text-muted); opacity: 0.3; cursor: default; }

  .address-bar { 
    flex: 1; height: 26px; 
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 4px; display: flex; align-items: center; overflow: hidden; 
    color: var(--text-main);
    -webkit-app-region: no-drag; 
  }

  .path-input { width: 100%; height: 100%; background: transparent; border: none; color: inherit; padding: 0 8px; font-size: 13px; outline: none; }
  .breadcrumbs { display: flex; align-items: center; height: 100%; width: 100%; cursor: text; padding-left: 4px; }
  
  .crumb { 
    background: transparent; border: none; 
    color: var(--text-muted);
    cursor: pointer; padding: 0 4px; font-size: 13px; height: 20px; border-radius: 3px;
    transition: all 0.15s;
  }
  .crumb:hover { 
    background: var(--hover-bg);
    color: var(--text-main);
  }
  .crumb.drag-over {
    background: rgba(59, 130, 246, 0.3);
    color: var(--text-main);
    outline: 2px dashed #3b82f6;
    outline-offset: -2px;
  }
  
  .divider { color: var(--text-muted); font-size: 14px; margin: 0 2px; }
  .spacer { flex: 1; height: 100%; }
</style>