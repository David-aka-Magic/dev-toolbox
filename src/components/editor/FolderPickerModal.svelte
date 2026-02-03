<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher, onMount } from "svelte";

  export let initialPath: string = 'C:\\';

  const dispatch = createEventDispatcher();

  let currentPath = initialPath;
  let files: any[] = [];
  let pathInput = currentPath;

  onMount(() => {
    loadFiles(currentPath);
  });

  async function loadFiles(path: string) {
    try {
      const allFiles = await invoke<any[]>("read_directory", { path });
      files = allFiles.filter(f => f.is_dir);
      currentPath = path;
      pathInput = path;
    } catch (err) {
      console.error("Failed to load directory:", err);
    }
  }

  function handleFolderClick(file: any) {
    loadFiles(file.path);
  }

  function handleFolderDblClick(file: any) {
    loadFiles(file.path);
  }

  function goUp() {
    const parts = currentPath.split(/[\\/]/).filter(p => p);
    if (parts.length > 1) {
      parts.pop();
      const newPath = parts.join('\\') + '\\';
      loadFiles(newPath);
    }
  }

  function handlePathSubmit() {
    loadFiles(pathInput);
  }

  function confirm() {
    dispatch('select', currentPath);
  }

  function cancel() {
    dispatch('cancel');
  }
</script>

<div class="modal-backdrop" on:click={cancel}>
  <div class="modal" on:click|stopPropagation>
    <div class="modal-header">
      <h3>Select Folder</h3>
      <button class="close-btn" on:click={cancel}>×</button>
    </div>

    <div class="modal-body">
      <div class="path-bar">
        <button class="up-btn" on:click={goUp} title="Go up">↑</button>
        <input 
          type="text" 
          bind:value={pathInput}
          on:keydown={(e) => e.key === 'Enter' && handlePathSubmit()}
          class="path-input"
        />
      </div>

      <div class="file-list">
        {#each files as file}
          <div 
            class="file-item"
            on:click={() => handleFolderClick(file)}
            on:dblclick={() => handleFolderDblClick(file)}
            role="button"
            tabindex="0"
          >
            <span class="file-icon">📁</span>
            <span class="file-name">{file.name}</span>
          </div>
        {/each}
        {#if files.length === 0}
          <div class="empty-message">No subfolders</div>
        {/if}
      </div>

      <div class="selected-path">
        <span class="label">Selected:</span>
        <span class="path">{currentPath}</span>
      </div>
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" on:click={cancel}>Cancel</button>
      <button class="btn btn-primary" on:click={confirm}>Select Folder</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }

  .modal {
    width: 600px;
    height: 500px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    color: var(--text-main);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 24px;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .modal-body {
    flex: 1;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }

  .path-bar {
    display: flex;
    gap: 8px;
  }

  .up-btn {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
  }

  .up-btn:hover {
    background: var(--hover-bg);
  }

  .path-input {
    flex: 1;
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 13px;
    outline: none;
  }

  .path-input:focus {
    border-color: var(--border-focus);
  }

  .file-list {
    flex: 1;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow-y: auto;
    padding: 8px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    cursor: pointer;
    border-radius: 4px;
    color: var(--text-main);
  }

  .file-item:hover {
    background: var(--hover-bg);
  }

  .file-icon {
    font-size: 18px;
  }

  .file-name {
    font-size: 13px;
  }

  .empty-message {
    color: var(--text-muted);
    font-size: 13px;
    text-align: center;
    padding: 20px;
  }

  .selected-path {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 4px;
  }

  .selected-path .label {
    color: var(--text-muted);
    font-size: 13px;
  }

  .selected-path .path {
    color: var(--text-main);
    font-size: 13px;
    font-family: monospace;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    border: none;
    transition: background 0.2s;
  }

  .btn-secondary {
    background: var(--bg-main);
    color: var(--text-main);
  }

  .btn-secondary:hover {
    background: var(--hover-bg);
  }

  .btn-primary {
    background: var(--border-focus);
    color: white;
  }

  .btn-primary:hover {
    background: #2563eb;
  }
</style>