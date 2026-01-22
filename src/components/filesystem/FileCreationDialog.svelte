<script lang="ts">
  import { tick } from 'svelte';

  export let type: 'folder' | 'file';
  export let initialName: string = '';

  // Callback props
  export let onconfirm: ((detail: any) => void) | undefined = undefined;
  export let oncancel: (() => void) | undefined = undefined;

  let creationInput: HTMLInputElement;
  let creationName = initialName || (type === 'folder' ? 'New Folder' : 'New Text Document.txt');

  // Auto-focus and select on mount
  async function initializeInput() {
    await tick();
    if (creationInput) {
      creationInput.focus();
      
      if (type === 'file') {
        const dotIndex = creationName.lastIndexOf('.');
        if (dotIndex > 0) {
          creationInput.setSelectionRange(0, dotIndex);
        } else {
          creationInput.select();
        }
      } else {
        creationInput.select();
      }
      
      creationInput.scrollIntoView({ block: 'nearest' });
    }
  }

  initializeInput();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      onconfirm?.({ name: creationName.trim() });
    }
    if (event.key === 'Escape') {
      oncancel?.();
    }
    event.stopPropagation();
  }

  function handleBlur() {
    onconfirm?.({ name: creationName.trim() });
  }

  function handleClick(event: MouseEvent) {
    event.stopPropagation();
  }

  function handleInputClick(event: MouseEvent) {
    event.stopPropagation();
  }
</script>

<div
  class="grid-item selected creating"
  on:click={handleClick}
  role="button"
  tabindex="0"
>
  <div class="icon">
    {type === 'folder' ? '📁' : '📄'}
  </div>
  <input
    bind:this={creationInput}
    bind:value={creationName}
    type="text"
    class="rename-input"
    on:click={handleInputClick}
    on:keydown={handleKeydown}
    on:blur={handleBlur}
  />
</div>

<style>
  .grid-item {
    background-color: rgba(0, 0, 0, 0.01);
    border: 2px solid transparent;
    box-sizing: border-box;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: default;
    color: var(--text-muted);
    text-align: center;
    padding: 5px;
    position: relative;
    outline: none;
    user-select: none;
    -webkit-user-select: none;
  }

  .grid-item.selected {
    background-color: var(--selection);
    border-color: var(--border-focus);
    color: var(--text-main);
  }

  .grid-item.creating {
    opacity: 1;
  }

  .icon {
    font-size: 32px;
    margin-bottom: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
  }

  .rename-input {
    width: 80px;
    font-size: 11px;
    text-align: center;
    background: var(--bg-main);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
    border-radius: 2px;
    padding: 2px 4px;
    outline: none;
  }

  .rename-input:focus {
    border-color: var(--border-focus);
  }
</style>