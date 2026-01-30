<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";

  export let x: number;
  export let y: number;
  export let options: { label: string; action: string; danger?: boolean; disabled?: boolean; shortcut?: string }[] = [];
  export let targetFile: string | null = null;

  export let onclose: (() => void) | undefined = undefined;
  export let onclick: ((action: string, targetFile: string | null) => void) | undefined = undefined;

  let menuElement: HTMLDivElement;
  let isReady = false;
  
  // Capture targetFile at mount time so it can't be changed
  let capturedTargetFile: string | null = null;

  console.log('ContextMenu script init, targetFile prop:', targetFile);

  $: {
    console.log('ContextMenu reactive: targetFile prop changed to:', targetFile);
  }

  function handleClickOutside(event: MouseEvent) {
    if (!isReady) return;
    
    if (menuElement && !menuElement.contains(event.target as Node)) {
      onclose?.();
    }
  }

  function handleContextMenuOutside(event: MouseEvent) {
    if (!isReady) return;
    onclose?.();
  }

  function handleMenuClick(event: MouseEvent, action: string, disabled?: boolean) {
    event.stopPropagation();
    event.preventDefault();
    
    if (disabled) return;
    
    console.log('ContextMenu handleMenuClick - action:', action, 'capturedTargetFile:', capturedTargetFile);
    onclick?.(action, capturedTargetFile);
  }

  onMount(() => {
    capturedTargetFile = targetFile;
    console.log('ContextMenu onMount - captured targetFile:', capturedTargetFile);
    
    if (menuElement) {
      const rect = menuElement.getBoundingClientRect();
      if (x + rect.width > window.innerWidth) x -= rect.width;
      if (y + rect.height > window.innerHeight) y -= rect.height;
    }
    
    const timeoutId = setTimeout(() => {
      isReady = true;
      window.addEventListener("click", handleClickOutside, true);
      window.addEventListener("contextmenu", handleContextMenuOutside);
    }, 0);
    
    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener("click", handleClickOutside, true);
      window.removeEventListener("contextmenu", handleContextMenuOutside);
    };
  });
</script>

<div 
  class="context-menu" 
  style="top: {y}px; left: {x}px;"
  bind:this={menuElement}
  transition:fade={{ duration: 100 }}
  role="menu"
  on:click|stopPropagation
  on:keydown|stopPropagation
>
  {#each options as option}
    {#if option.label === 'SEPARATOR'}
      <div class="separator" role="separator"></div>
    {:else}
      <button 
        class="menu-item" 
        class:danger={option.danger}
        class:disabled={option.disabled}
        on:click={(e) => handleMenuClick(e, option.action, option.disabled)}
        disabled={option.disabled}
        role="menuitem"
      >
        <span class="label">{option.label}</span>
        {#if option.shortcut}
          <span class="shortcut">{option.shortcut}</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 1000;
    min-width: 180px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    padding: 4px 0;
    display: flex;
    flex-direction: column;
  }

  .menu-item {
    background: transparent;
    border: none;
    text-align: left;
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text-main);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
  }

  .menu-item:hover:not(.disabled) {
    background: var(--hover-bg);
  }

  .menu-item.danger { color: #ef4444; }
  .menu-item.danger:hover:not(.disabled) { background: rgba(239, 68, 68, 0.1); }

  .menu-item.disabled {
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.5;
  }

  .label {
    flex: 1;
  }

  .shortcut {
    font-size: 11px;
    color: var(--text-muted);
  }

  .separator {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
</style>