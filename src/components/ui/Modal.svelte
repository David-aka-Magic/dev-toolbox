<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  // Props
  export let open: boolean = false;
  export let title: string = 'Modal';
  export let width: string = '500px';
  export let height: string = 'auto'; // NEW PROP

  const dispatch = createEventDispatcher();

  function close() {
    dispatch('close');
  }

  function handleKeydown(event: KeyboardEvent) {
    if (open && event.key === 'Escape') close();
  }
</script>

<svelte:window on:keydown={handleKeydown}/>

{#if open}
  <div 
    class="backdrop" 
    on:click={close} 
    transition:fade={{ duration: 200 }}
  >
    <div 
      class="modal-window" 
      style:width={width}
      style:height={height}
      on:click|stopPropagation 
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <header>
        <h2>{title}</h2>
        <button class="close-btn" on:click={close}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </header>
      
      <div class="content">
        <slot />
      </div>

      {#if $$slots.footer}
        <footer>
          <slot name="footer" />
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    top: 0; left: 0; width: 100vw; height: 100vh;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 9999;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .modal-window {
    background: var(--bg-header); 
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 10px 25px rgba(0,0,0,0.5);
    display: flex;
    flex-direction: column;
    max-width: 95vh; /* Safety margin */
    max-height: 95vh; /* Safety margin */
    overflow: hidden;
    color: var(--text-main);
  }

  header {
    padding: 15px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0; /* Prevents header from shrinking */
  }

  h2 { margin: 0; font-size: 1.1rem; font-weight: 600; }

  .content { 
    padding: 20px; 
    color: var(--text-muted);
    overflow: hidden;
    flex: 1; /* Takes up remaining space */
    position: relative;
  }

  footer {
    padding: 15px 20px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    flex-shrink: 0; /* Prevents footer from shrinking */
  }

  .close-btn {
    background: transparent; border: none; color: var(--text-muted);
    cursor: pointer; padding: 4px; border-radius: 4px; display: flex;
  }
  .close-btn:hover { background: var(--hover-bg); color: var(--text-main); }
</style>