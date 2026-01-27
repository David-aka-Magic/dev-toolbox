<script lang="ts">
  import { recentFiles } from '$lib/stores/recentFileStore';
  import Sidebar from '../ui/SideBar.svelte';
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  function openFile(path: string, name: string) {
    dispatch('openfile', { path, name });
  }
  
  function clearHistory() {
    if (confirm('Clear all recent files?')) {
      recentFiles.clear();
    }
  }
  
  function formatDate(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return new Date(timestamp).toLocaleDateString();
  }
</script>

<Sidebar title="Recent Files" width={250}>
  <div class="recent-files">
    {#if $recentFiles.length === 0}
      <div class="empty-state">
        <p>No recent files</p>
        <span>Files you open will appear here</span>
      </div>
    {:else}
      {#each $recentFiles as file}
        <button 
          class="file-item"
          on:click={() => openFile(file.path, file.name)}
          title={file.path}
        >
          <div class="file-icon">📄</div>
          <div class="file-info">
            <div class="file-name">{file.name}</div>
            <div class="file-time">{formatDate(file.lastOpened)}</div>
          </div>
        </button>
      {/each}
    {/if}
  </div>
  
  <div slot="footer">
    {#if $recentFiles.length > 0}
      <button class="clear-btn" on:click={clearHistory}>
        Clear History
      </button>
    {/if}
  </div>
</Sidebar>

<style>
  .recent-files {
    padding: 4px;
  }
  
  .empty-state {
    padding: 20px;
    text-align: center;
    color: var(--text-muted);
  }
  
  .empty-state p {
    margin: 0 0 4px 0;
    font-size: 13px;
  }
  
  .empty-state span {
    font-size: 11px;
    opacity: 0.7;
  }
  
  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-main);
    text-align: left;
    transition: background 0.15s;
  }
  
  .file-item:hover {
    background: var(--hover-bg);
  }
  
  .file-icon {
    font-size: 16px;
    flex-shrink: 0;
  }
  
  .file-info {
    flex: 1;
    min-width: 0;
  }
  
  .file-name {
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .file-time {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 2px;
  }
  
  .clear-btn {
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 0;
    font-size: 10px;
    cursor: pointer;
    text-align: center;
  }
  
  .clear-btn:hover {
    color: #ef4444;
  }
</style>