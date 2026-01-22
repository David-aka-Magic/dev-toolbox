<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";

  export let x: number;
  export let y: number;
  export let options: { label: string; action: string; danger?: boolean }[] = [];

  // Callback props instead of createEventDispatcher
  export let onclose: (() => void) | undefined = undefined;
  export let onclick: ((action: string) => void) | undefined = undefined;

  let menuElement: HTMLDivElement;
  let isReady = false;

  // Close if clicking outside
  function handleClickOutside(event: MouseEvent) {
    if (!isReady) return; // Don't handle clicks until we're ready
    
    if (menuElement && !menuElement.contains(event.target as Node)) {
      onclose?.();
    }
  }

  function handleContextMenuOutside(event: MouseEvent) {
    if (!isReady) return; // Don't handle clicks until we're ready
    
    // Close on any new right-click anywhere
    onclose?.();
  }

  function handleMenuClick(action: string) {
    onclick?.(action);
  }

  // Adjust position if it goes off-screen
  onMount(() => {
    if (menuElement) {
      const rect = menuElement.getBoundingClientRect();
      if (x + rect.width > window.innerWidth) x -= rect.width;
      if (y + rect.height > window.innerHeight) y -= rect.height;
    }
    
    // Use setTimeout to add listeners AFTER the current event has finished
    const timeoutId = setTimeout(() => {
      isReady = true;
      window.addEventListener("click", handleClickOutside);
      window.addEventListener("contextmenu", handleContextMenuOutside);
    }, 0);
    
    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener("click", handleClickOutside);
      window.removeEventListener("contextmenu", handleContextMenuOutside);
    };
  });
</script>

<div 
  class="context-menu" 
  style="top: {y}px; left: {x}px;"
  bind:this={menuElement}
  transition:fade={{ duration: 100 }}
>
  {#each options as option}
    {#if option.label === 'SEPARATOR'}
      <div class="separator"></div>
    {:else}
      <button 
        class="menu-item" 
        class:danger={option.danger}
        on:click={() => handleMenuClick(option.action)}
      >
        {option.label}
      </button>
    {/if}
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 1000;
    width: 180px;
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
  }

  .menu-item:hover {
    background: var(--hover-bg);
  }

  .menu-item.danger { color: #ef4444; }
  .menu-item.danger:hover { background: rgba(239, 68, 68, 0.1); }

  .separator {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
</style>