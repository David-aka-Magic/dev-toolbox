<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { fileTabs } from '$lib/stores/fileTabStore';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import Sidebar from '../ui/SideBar.svelte';
  import FileNode from './FileNode.svelte';

  interface DriveInfo {
    letter: string;
    path: string;
    label: string | null;
    drive_type: string;
  }

  interface RecentFile {
    path: string;
    name: string;
    timestamp: number;
  }

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  let drives: DriveInfo[] = [];
  let recentFiles: RecentFile[] = [];
  let loading = true;
  let expandedDrives: Set<string> = new Set();
  let driveContents: Map<string, FileEntry[]> = new Map();
  let dragOverDrive: string | null = null;

  $: currentPath = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId)?.path || '';
  
  function isPathActive(checkPath: string): boolean {
    if (!currentPath) return false;
    const normCurrent = currentPath.toLowerCase().replace(/\//g, '\\');
    const normCheck = checkPath.toLowerCase().replace(/\//g, '\\');
    return normCurrent === normCheck || normCurrent.startsWith(normCheck + '\\');
  }

  onMount(async () => {
    try {
      drives = await invoke<DriveInfo[]>('get_available_drives');
      loadRecentFiles();
    } catch (err) {
      console.error('Failed to get drives:', err);
    } finally {
      loading = false;
    }
  });

  function loadRecentFiles() {
    const stored = localStorage.getItem('recent-files');
    if (stored) {
      recentFiles = JSON.parse(stored);
    }
  }

  function saveRecentFiles() {
    localStorage.setItem('recent-files', JSON.stringify(recentFiles));
  }

  export function addRecentFile(path: string) {
    const name = path.split(/[\\/]/).pop() || path;
    const existing = recentFiles.findIndex(f => f.path === path);
    
    if (existing !== -1) {
      recentFiles.splice(existing, 1);
    }
    
    recentFiles.unshift({
      path,
      name,
      timestamp: Date.now()
    });
    
    if (recentFiles.length > 10) {
      recentFiles = recentFiles.slice(0, 10);
    }
    
    saveRecentFiles();
  }

  async function toggleDrive(drivePath: string) {
    if (expandedDrives.has(drivePath)) {
      expandedDrives.delete(drivePath);
      expandedDrives = expandedDrives;
    } else {
      expandedDrives.add(drivePath);
      expandedDrives = expandedDrives;
      
      if (!driveContents.has(drivePath)) {
        try {
          const contents = await invoke<FileEntry[]>("read_directory", { path: drivePath });
          driveContents.set(drivePath, contents);
          driveContents = driveContents;
        } catch (err) {
          console.error('Failed to load drive contents:', err);
        }
      }
    }
  }

  function navigateToDrive(drive: DriveInfo) {
    fileTabs.updateActivePath(drive.path);
  }

  function openRecentFile(file: RecentFile) {
    const isDirectory = file.path.endsWith('\\') || file.path.endsWith('/');
    if (isDirectory) {
      fileTabs.updateActivePath(file.path);
    } else {
      const parentPath = file.path.split(/[\\/]/).slice(0, -1).join('\\');
      fileTabs.updateActivePath(parentPath);
    }
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
      return `${drive.letter}: (${drive.label})`;
    }
    return `${drive.letter}:`;
  }

  function formatTimestamp(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  }

  function getDriveContents(drivePath: string): FileEntry[] {
    return driveContents.get(drivePath) || [];
  }

  function handleDriveDragEnter(event: DragEvent, drive: DriveInfo) {
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    dragOverDrive = drive.path;
  }

  function handleDriveDragOver(event: DragEvent, drive: DriveInfo) {
    const state = $fileDragDrop;
    if (!state.draggedFile) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    // Ensure dragOverDrive stays set during dragging
    dragOverDrive = drive.path;
    
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  function handleDriveDragLeave(event: DragEvent) {
    // Only clear if we're actually leaving this element
    const relatedTarget = event.relatedTarget as HTMLElement;
    const currentTarget = event.currentTarget as HTMLElement;
    
    // Check if we're leaving to a child element
    if (relatedTarget && currentTarget.contains(relatedTarget)) {
      return;
    }
    
    dragOverDrive = null;
  }

  async function handleDriveDrop(event: DragEvent, drive: DriveInfo) {
    event.preventDefault();
    event.stopPropagation();
    dragOverDrive = null;

    const state = $fileDragDrop;
    if (!state.draggedFile) return;

    const destPath = drive.path;
    
    const activeTab = $fileTabs.tabs.find(t => t.id === $fileTabs.activeId);
    const currentPath = activeTab?.path;
    
    if (!currentPath) return;
    
    if (destPath === currentPath) {
      console.log("⚠️ Cannot drop into the same folder");
      return;
    }

    console.log(`📦 Moving ${state.draggedFiles.length} file(s) to drive:`, destPath);
    console.log('Source directory:', currentPath);
    console.log('Destination directory:', destPath);

    try {
      for (const fileName of state.draggedFiles) {
        // Always construct the source path from current directory + filename
        // Don't trust the cached path in draggedFilePaths since the file might have moved
        const sourcePath = `${currentPath}\\${fileName}`;
        
        console.log(`📦 MOVE_ITEM: source="${sourcePath}" destination="${destPath}"`);
        const result = await invoke('move_item', { src: sourcePath, dest: destPath });
        console.log('Move result:', result);
      }
      
      // Invalidate cache for both source and destination
      const { directoryCache } = await import('$lib/stores/directoryCacheStore');
      directoryCache.invalidate(currentPath);
      directoryCache.invalidate(destPath);
      
      // Trigger immediate reload
      window.dispatchEvent(new CustomEvent('force-file-refresh'));
      
      // Clear the drag state since files have moved
      fileDragDrop.handleDragEnd();
      
      if (expandedDrives.has(destPath)) {
        const contents = await invoke<FileEntry[]>("read_directory", { path: destPath });
        driveContents.set(destPath, contents);
        driveContents = driveContents;
      }
    } catch (err) {
      console.error("❌ Move error:", err);
      alert("Move failed: " + err);
    }
  }
</script>

<Sidebar title="Explorer" width={250}>
  {#if loading}
    <div class="loading">Loading...</div>
  {:else}
    <div class="navigator-content">
      <!-- Drives Section -->
      <div class="section">
        <div class="section-header">DRIVES</div>
        <div class="drives-list">
          {#each drives as drive}
            <div class="drive-container">
              <div class="drive-header">
                <button 
                  class="expand-btn"
                  on:click={() => toggleDrive(drive.path)}
                >
                  {#if expandedDrives.has(drive.path)}
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  {:else}
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="9 18 15 12 9 6"></polyline>
                    </svg>
                  {/if}
                </button>
                <button 
                  class="drive-item"
                  class:drag-over={dragOverDrive === drive.path}
                  class:active={isPathActive(drive.path)}
                  on:click={() => navigateToDrive(drive)}
                  on:dragenter={(e) => handleDriveDragEnter(e, drive)}
                  on:dragover={(e) => handleDriveDragOver(e, drive)}
                  on:dragleave={handleDriveDragLeave}
                  on:drop={(e) => handleDriveDrop(e, drive)}
                  title={drive.path}
                >
                  <span class="drive-icon">{getDriveIcon(drive.drive_type)}</span>
                  <span class="drive-label">{getDriveLabel(drive)}</span>
                </button>
              </div>
              
              {#if expandedDrives.has(drive.path)}
                <div class="drive-contents">
                  {#if driveContents.has(drive.path)}
                    {#each getDriveContents(drive.path) as child}
                      <FileNode 
                        name={child.name} 
                        path={child.path} 
                        isDir={child.is_dir} 
                        depth={0}
                      />
                    {/each}
                  {:else}
                    <div class="file-loading">Loading...</div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Recent Files Section -->
      <div class="section">
        <div class="section-header">RECENT</div>
        {#if recentFiles.length === 0}
          <div class="empty-message">No recent files</div>
        {:else}
          <div class="recent-list">
            {#each recentFiles as file}
              <button 
                class="recent-item"
                on:click={() => openRecentFile(file)}
                title={file.path}
              >
                <span class="recent-icon">📄</span>
                <div class="recent-info">
                  <div class="recent-name">{file.name}</div>
                  <div class="recent-time">{formatTimestamp(file.timestamp)}</div>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</Sidebar>

<style>
  .loading {
    padding: 16px 12px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .navigator-content {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 8px 0;
  }

  .section {
    display: flex;
    flex-direction: column;
  }

  .section-header {
    padding: 8px 12px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.7;
    color: var(--text-main);
  }

  .drives-list {
    display: flex;
    flex-direction: column;
  }

  .drive-container {
    display: flex;
    flex-direction: column;
  }

  .drive-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 4px;
  }

  .expand-btn {
    background: transparent;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-main);
    opacity: 0.6;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .expand-btn:hover {
    opacity: 1;
    background: var(--hover-bg);
    border-radius: 3px;
  }

  .drive-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-main);
    font-size: 13px;
    text-align: left;
    transition: background 0.15s;
    flex: 1;
    min-width: 0;
  }

  .drive-item:hover {
    background: var(--hover-bg);
  }

  .drive-item.active {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
  }

  .drive-item.drag-over {
    background: rgba(59, 130, 246, 0.3);
    outline: 2px dashed #3b82f6;
    outline-offset: -2px;
  }

  .drive-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .drive-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .drive-contents {
    padding-left: 8px;
  }

  .file-loading {
    color: var(--text-muted);
    font-size: 11px;
    font-style: italic;
    padding: 8px 12px;
  }

  .recent-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 4px;
  }

  .recent-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-main);
    font-size: 13px;
    text-align: left;
    transition: background 0.15s;
  }

  .recent-item:hover {
    background: var(--hover-bg);
  }

  .recent-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .recent-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .recent-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .recent-time {
    font-size: 11px;
    color: var(--text-muted);
  }

  .empty-message {
    padding: 8px 12px;
    color: var(--text-muted);
    font-size: 12px;
    font-style: italic;
  }
</style>