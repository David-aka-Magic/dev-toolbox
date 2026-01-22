<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  export let name: string;
  export let path: string;
  export let isDir: boolean;
  export let depth: number = 0;

  let isOpen = false;
  let children: any[] = [];
  let loading = false;

  async function toggle() {
    if (!isDir) return;
    isOpen = !isOpen;

    if (isOpen && children.length === 0) {
      loading = true;
      try {
        children = await invoke("read_directory", { path });
      } catch (err) {
        console.error(err);
      }
      loading = false;
    }
  }
</script>

<div 
  class="file-item"
  style="padding-left: {depth * 12 + 8}px"
  on:click={toggle}
>
  <span class="file-icon">
    {#if isDir}
      {isOpen ? '📂' : '📁'}
    {:else}
      📄
    {/if}
  </span>
  <span class="file-name">{name}</span>
</div>

{#if isOpen}
  {#if loading}
    <div class="file-loading" style="padding-left: {(depth + 1) * 12 + 8}px">Loading...</div>
  {:else}
    {#each children as child}
      <svelte:self 
        name={child.name} 
        path={child.path} 
        isDir={child.is_dir} 
        depth={depth + 1} 
      />
    {/each}
  {/if}
{/if}

<style>
  .file-item {
    display: flex;
    align-items: center;
    cursor: pointer;
    padding: 4px 0;
    white-space: nowrap;
    color: var(--text-primary); 
    font-size: 13px;
  }

  .file-item:hover {
    background-color: var(--hover-bg);
  }

  .file-icon {
    margin-right: 6px;
    font-size: 14px;
  }

  .file-name {
    user-select: none;
  }

  .file-loading {
    color: #888888;
    font-size: 11px;
    font-style: italic;
  }
</style>