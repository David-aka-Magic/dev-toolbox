<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import FileNode from "../filesystem/FileNode.svelte";
  import { onMount } from "svelte";

  let rootFiles: any[] = [];
  let currentPath = "C:/Users/dcarl/Fun-Projects"; 

  async function loadRoot() {
    try {
      rootFiles = await invoke("read_directory", { path: currentPath });
    } catch (err) {
      console.error(err);
    }
  }

  onMount(loadRoot);
</script>

<div class="h-full w-64 bg-[var(--bg-secondary)] border-r border-[var(--border)] flex flex-col">
  <div class="p-2 font-bold text-xs uppercase tracking-wider opacity-50">Explorer</div>
  <div class="flex-1 overflow-y-auto overflow-x-hidden">
    {#each rootFiles as file}
      <FileNode 
        name={file.name} 
        path={file.path} 
        isDir={file.is_dir} 
      />
    {/each}
  </div>
</div>