<script lang="ts">
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { tick } from "svelte";

  let isInputMode = false;
  let inputElement: HTMLInputElement;
  let rawPath = "";

  $: if ($activeTab) { rawPath = $activeTab.path; }
  $: parts = rawPath ? rawPath.split(/[\\/]/).filter(p => p !== "") : [];

  function navigateTo(index: number) {
    let newPath = parts.slice(0, index + 1).join("\\");
    if (index === 0 && newPath.endsWith(":")) newPath += "\\";
    fileTabs.updateActivePath(newPath);
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
        <div class="breadcrumbs" on:click={enableInput}>
          {#each parts as part, i}
            <button class="crumb" on:click|stopPropagation={() => navigateTo(i)}>
              {part}
            </button>
            <span class="divider">›</span>
          {/each}
          <div class="spacer"></div>
        </div>
      {/if}
    </div>

    <div class="nav-controls">
      <button class="nav-btn" disabled={!$activeTab || $activeTab.historyIndex <= 0} on:click={() => fileTabs.goBack()}>&lt;</button>
      <button class="nav-btn" disabled={!$activeTab || $activeTab.historyIndex >= $activeTab.history.length - 1} on:click={() => fileTabs.goForward()}>&gt;</button>
    </div>
</div>

<style>
  .address-container { display: flex; align-items: center; width: 100%; height: 100%; padding-right: 10px; }
  .nav-controls { display: flex; gap: 2px; margin-left: 6px; -webkit-app-region: no-drag; }
  
  .nav-btn { 
    background: transparent; border: none; 
    color: var(--text-muted); /* UPDATED */
    width: 24px; height: 24px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-weight: bold; 
  }
  .nav-btn:hover:not(:disabled) { 
    background: var(--hover-bg); /* UPDATED */
    color: var(--text-main); /* UPDATED */
  }
  .nav-btn:disabled { color: var(--text-muted); opacity: 0.3; cursor: default; }

  .address-bar { 
    flex: 1; height: 26px; 
    background: var(--bg-input); /* UPDATED */
    border: 1px solid var(--border); /* UPDATED */
    border-radius: 4px; display: flex; align-items: center; overflow: hidden; 
    color: var(--text-main); /* UPDATED */
    -webkit-app-region: no-drag; 
  }

  .path-input { width: 100%; height: 100%; background: transparent; border: none; color: inherit; padding: 0 8px; font-size: 13px; outline: none; }
  .breadcrumbs { display: flex; align-items: center; height: 100%; width: 100%; cursor: text; padding-left: 4px; }
  
  .crumb { 
    background: transparent; border: none; 
    color: var(--text-muted); /* UPDATED */
    cursor: pointer; padding: 0 4px; font-size: 13px; height: 20px; border-radius: 3px; 
  }
  .crumb:hover { 
    background: var(--hover-bg); /* UPDATED */
    color: var(--text-main); /* UPDATED */
  }
  
  .divider { color: var(--text-muted); font-size: 14px; margin: 0 2px; }
  .spacer { flex: 1; height: 100%; }
</style>