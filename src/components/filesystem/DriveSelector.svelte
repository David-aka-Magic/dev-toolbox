<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { fileTabs } from '$lib/stores/fileTabStore';

  interface DriveInfo {
    letter: string;
    path: string;
    label: string | null;
    drive_type: string;
  }

  let drives: DriveInfo[] = [];
  let loading = true;

  onMount(async () => {
    try {
      drives = await invoke<DriveInfo[]>('get_available_drives');
    } catch (err) {
      console.error('Failed to get drives:', err);
    } finally {
      loading = false;
    }
  });

  function navigateToDrive(drive: DriveInfo) {
    fileTabs.updateActivePath(drive.path);
  }

  function getDriveIcon(driveType: string): string {
    switch (driveType) {
      case 'fixed': return '💾';
      case 'removable': return '🔌';
      case 'network': return '🌐';
      case 'cdrom': return '💿';
      default: return '💽';
    }
  }

  function getDriveLabel(drive: DriveInfo): string {
    if (drive.label) {
      return `${drive.label} (${drive.letter}:)`;
    }
    return `${drive.letter}: Drive`;
  }
</script>

<div class="drives-container">
  <div class="drives-header">
    <span class="header-text">DRIVES</span>
  </div>

  {#if loading}
    <div class="loading">Loading drives...</div>
  {:else}
    <div class="drives-list">
      {#each drives as drive}
        <button 
          class="drive-item"
          on:click={() => navigateToDrive(drive)}
          title={drive.path}
        >
          <span class="drive-icon">{getDriveIcon(drive.drive_type)}</span>
          <span class="drive-label">{getDriveLabel(drive)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .drives-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    color: var(--text-main);
  }

  .drives-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }

  .header-text {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.7;
  }

  .loading {
    padding: 16px 12px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .drives-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px;
  }

  .drive-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-main);
    font-size: 13px;
    text-align: left;
    transition: background 0.15s;
  }

  .drive-item:hover {
    background: var(--hover-bg);
  }

  .drive-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .drive-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>