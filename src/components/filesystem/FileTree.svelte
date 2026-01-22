<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import FileNode from "./FileNode.svelte";
  import Sidebar from "..//ui/SideBar.svelte";
  import { toggleTheme } from "$lib/stores/theme";
  import { currentPath } from "$lib/stores/path";

  let rootFiles: any[] = [];
  let unsubscribePath: () => void;

  async function loadRoot(path: string) {
    try {
      rootFiles = await invoke("read_directory", { path });
    } catch (err) {
      console.error("Error loading root:", err);
    }
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

<Sidebar title="Explorer" width={250}>
  
  <!-- Main content: the file tree -->
  <div class="tree-list">
    {#each rootFiles as file}
      <FileNode 
        name={file.name} 
        path={file.path} 
        isDir={file.is_dir} 
      />
    {/each}
  </div>

  <!-- Footer slot: theme toggle -->
  <svelte:fragment slot="footer">
    <button class="footer-btn" on:click={toggleTheme}>
      <span class="theme-icon">🌓</span>
      Theme
    </button>
  </svelte:fragment>
  
</Sidebar>

<style>
  .tree-list {
    padding: 5px;
  }

  .footer-btn {
    background: none;
    border: none;
    color: var(--text-main);
    opacity: 0.7;
    font-size: 11px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 100%;
    white-space: nowrap;
  }

  .footer-btn:hover {
    opacity: 1;
    background-color: var(--hover-bg);
  }

  .theme-icon {
    font-size: 12px;
  }
</style>