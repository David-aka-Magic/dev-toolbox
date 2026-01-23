<script lang="ts">
  import { fileSelection, selectedFiles, focusedIndex } from './hooks/useFileSelection';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { thumbnails } from './hooks/useThumbnailLoader';
  import { sortConfig, toggleSort, type SortField } from '$lib/stores/viewModeStore';
  import FileCreationDialog from './FileCreationDialog.svelte';

  export let files: any[] = [];
  export let isLoading: boolean = false;
  export let renamingFile: string | null = null;
  export let creationType: 'folder' | 'file' | null = null;

  // Callback props
  export let onbackgroundclick: (() => void) | undefined = undefined;
  export let onitemdblclick: ((detail: any) => void) | undefined = undefined;
  export let onitemcontextmenu: ((detail: any) => void) | undefined = undefined;
  export let onitemdrop: ((detail: any) => void) | undefined = undefined;
  export let onrenamesubmit: ((detail: any) => void) | undefined = undefined;
  export let oncreationconfirm: ((detail: any) => void) | undefined = undefined;
  export let oncreationcancel: (() => void) | undefined = undefined;

  let detailsContainer: HTMLDivElement;
  let renameInput: HTMLInputElement;
  let renameValue = '';

  // Get file icon
  function getIcon(file: any): string {
    if (file.is_dir) return '📁';
    const ext = file.name.split('.').pop()?.toLowerCase() || '';
    const iconMap: Record<string, string> = {
      'txt': '📄', 'md': '📝', 'pdf': '📕',
      'doc': '📘', 'docx': '📘', 'xls': '📗', 'xlsx': '📗',
      'jpg': '🖼️', 'jpeg': '🖼️', 'png': '🖼️', 'gif': '🖼️', 'webp': '🖼️', 'svg': '🖼️',
      'mp4': '🎬', 'mkv': '🎬', 'avi': '🎬', 'mov': '🎬', 'webm': '🎬',
      'mp3': '🎵', 'wav': '🎵', 'flac': '🎵', 'ogg': '🎵',
      'zip': '📦', 'rar': '📦', '7z': '📦', 'tar': '📦', 'gz': '📦',
      'js': '📜', 'ts': '📜', 'py': '🐍', 'rs': '🦀', 'go': '🔵',
      'html': '🌐', 'css': '🎨', 'json': '📋', 'xml': '📋', 'yaml': '📋',
      'exe': '⚙️', 'msi': '⚙️', 'dll': '⚙️',
    };
    return iconMap[ext] || '📄';
  }

  // Get file type description
  function getFileType(file: any): string {
    if (file.is_dir) return 'Folder';
    const ext = file.name.split('.').pop()?.toLowerCase() || '';
    const typeMap: Record<string, string> = {
      'txt': 'Text Document', 'md': 'Markdown', 'pdf': 'PDF Document',
      'doc': 'Word Document', 'docx': 'Word Document', 
      'xls': 'Excel Spreadsheet', 'xlsx': 'Excel Spreadsheet',
      'jpg': 'JPEG Image', 'jpeg': 'JPEG Image', 'png': 'PNG Image', 
      'gif': 'GIF Image', 'webp': 'WebP Image', 'svg': 'SVG Image',
      'mp4': 'MP4 Video', 'mkv': 'MKV Video', 'avi': 'AVI Video', 
      'mov': 'MOV Video', 'webm': 'WebM Video',
      'mp3': 'MP3 Audio', 'wav': 'WAV Audio', 'flac': 'FLAC Audio', 'ogg': 'OGG Audio',
      'zip': 'ZIP Archive', 'rar': 'RAR Archive', '7z': '7z Archive',
      'js': 'JavaScript', 'ts': 'TypeScript', 'py': 'Python', 
      'rs': 'Rust', 'go': 'Go',
      'html': 'HTML', 'css': 'CSS', 'json': 'JSON', 'xml': 'XML', 'yaml': 'YAML',
      'exe': 'Executable', 'msi': 'Installer', 'dll': 'DLL',
    };
    return typeMap[ext] || `${ext.toUpperCase()} File`;
  }

  // Format file size
  function formatSize(bytes: number | undefined): string {
    if (bytes === undefined || bytes === null) return '';
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // Format date
  function formatDate(timestamp: number | undefined): string {
    if (!timestamp) return '';
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  // Event handlers
  function handleBackgroundClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onbackgroundclick?.();
    }
  }

  function handleItemClick(event: MouseEvent, index: number, fileName: string) {
    fileSelection.handleClick(event, index, fileName, files);
  }

  function handleItemDblClick(file: any) {
    if (renamingFile || creationType) return;
    onitemdblclick?.({ file });
  }

  function handleItemContextMenu(event: MouseEvent, fileName: string) {
    event.preventDefault();
    onitemcontextmenu?.({ event, fileName });
  }

  function handleDragStart(event: DragEvent, file: any) {
    fileDragDrop.handleDragStart(event, file, $selectedFiles, files);
  }

  function handleDragEnd() {
    fileDragDrop.handleDragEnd();
  }

  function handleDragEnter(event: DragEvent, file: any) {
    fileDragDrop.handleItemDragEnter(event, file);
  }

  function handleDragOver(event: DragEvent, file: any) {
    fileDragDrop.handleItemDragOver(event, file);
  }

  function handleDragLeave(event: DragEvent) {
    fileDragDrop.handleItemDragLeave(event);
  }

  function handleDrop(event: DragEvent, file: any) {
    onitemdrop?.({ event, file });
  }

  // Sorting
  function handleHeaderClick(field: SortField) {
    toggleSort(field);
  }

  function getSortIndicator(field: SortField): string {
    if ($sortConfig.field !== field) return '';
    return $sortConfig.direction === 'asc' ? ' ▲' : ' ▼';
  }

  // Rename handling
  function startRename(fileName: string) {
    renameValue = fileName;
  }

  $: if (renamingFile) {
    startRename(renamingFile);
  }

  function handleRenameKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      onrenamesubmit?.({ newName: renameValue.trim() });
    }
    if (event.key === 'Escape') {
      onrenamesubmit?.({ newName: renamingFile });
    }
    event.stopPropagation();
  }

  function handleRenameBlur() {
    onrenamesubmit?.({ newName: renameValue.trim() });
  }

  export function getContainerWidth(): number {
    return detailsContainer?.clientWidth || 800;
  }
</script>

<div
  class="details-container"
  bind:this={detailsContainer}
  on:click={handleBackgroundClick}
  role="presentation"
>
  <!-- Column Headers -->
  <div class="header-row">
    <div class="col-icon"></div>
    <button class="col-name header-btn" on:click={() => handleHeaderClick('name')}>
      Name{getSortIndicator('name')}
    </button>
    <button class="col-modified header-btn" on:click={() => handleHeaderClick('modified')}>
      Date Modified{getSortIndicator('modified')}
    </button>
    <button class="col-type header-btn" on:click={() => handleHeaderClick('type')}>
      Type{getSortIndicator('type')}
    </button>
    <button class="col-size header-btn" on:click={() => handleHeaderClick('size')}>
      Size{getSortIndicator('size')}
    </button>
  </div>

  <!-- File List -->
  <div class="file-list">
    {#if isLoading && !files.length}
      <div class="loading">Loading...</div>
    {:else}
      {#each files as file, i (file.name)}
        <div
          id="file-btn-{i}"
          class="details-row"
          class:selected={$selectedFiles.has(file.name)}
          class:focused={$focusedIndex === i}
          class:being-dragged={$fileDragDrop.draggedFiles.includes(file.name)}
          draggable={true}
          on:click={(e) => handleItemClick(e, i, file.name)}
          on:dblclick={() => handleItemDblClick(file)}
          on:contextmenu={(e) => handleItemContextMenu(e, file.name)}
          on:dragstart={(e) => handleDragStart(e, file)}
          on:dragend={handleDragEnd}
          on:dragenter={(e) => handleDragEnter(e, file)}
          on:dragover={(e) => handleDragOver(e, file)}
          on:dragleave={handleDragLeave}
          on:drop={(e) => handleDrop(e, file)}
          role="button"
          tabindex="0"
        >
          <span class="col-icon">
            {#if $thumbnails.has(file.path)}
              <img src={$thumbnails.get(file.path)} alt="" class="thumbnail" />
            {:else}
              {getIcon(file)}
            {/if}
          </span>
          
          <span class="col-name">
            {#if renamingFile === file.name}
              <input
                bind:this={renameInput}
                bind:value={renameValue}
                type="text"
                class="rename-input"
                on:keydown={handleRenameKeydown}
                on:blur={handleRenameBlur}
                on:click|stopPropagation
              />
            {:else}
              <span class="name-text" title={file.name}>{file.name}</span>
            {/if}
          </span>
          
          <span class="col-modified">{formatDate(file.modified)}</span>
          <span class="col-type">{getFileType(file)}</span>
          <span class="col-size">{file.is_dir ? '' : formatSize(file.size)}</span>
        </div>
      {/each}

      {#if creationType}
        <FileCreationDialog
          type={creationType}
          onconfirm={oncreationconfirm}
          oncancel={oncreationcancel}
        />
      {/if}
    {/if}
  </div>
</div>

<style>
  .details-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header-row {
    display: flex;
    align-items: center;
    padding: 8px 10px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    gap: 8px;
    flex-shrink: 0;
  }

  .header-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    padding: 4px 8px;
    text-align: left;
    border-radius: 4px;
  }

  .header-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .loading {
    padding: 20px;
    color: var(--text-muted);
  }

  .details-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: default;
    color: var(--text-muted);
    user-select: none;
    -webkit-user-select: none;
    transition: background-color 0.1s;
  }

  .details-row:hover {
    background-color: var(--hover-bg);
    color: var(--text-main);
  }

  .details-row.selected {
    background-color: var(--selection);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
  }

  .details-row.focused {
    outline: 1px dotted var(--border-focus);
    outline-offset: -1px;
  }

  .details-row.being-dragged {
    opacity: 0.4;
  }

  .col-icon {
    width: 24px;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .thumbnail {
    width: 20px;
    height: 20px;
    object-fit: cover;
    border-radius: 2px;
  }

  .col-name {
    flex: 1;
    min-width: 200px;
    overflow: hidden;
  }

  .name-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .col-modified {
    width: 150px;
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .col-type {
    width: 120px;
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .col-size {
    width: 80px;
    text-align: right;
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .rename-input {
    width: 100%;
    font-size: 13px;
    background: var(--bg-main);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
    border-radius: 2px;
    padding: 2px 6px;
    outline: none;
  }
</style>