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

  export let onbackgroundclick: (() => void) | undefined = undefined;
  export let onitemdblclick: ((detail: any) => void) | undefined = undefined;
  export let onitemcontextmenu: ((detail: any) => void) | undefined = undefined;
  export let onitemdrop: ((detail: any) => void) | undefined = undefined;
  export let onrenamesubmit: ((detail: any) => void) | undefined = undefined;
  export let oncreationconfirm: ((detail: any) => void) | undefined = undefined;
  export let oncreationcancel: (() => void) | undefined = undefined;

  let listContainer: HTMLDivElement;
  let renameInput: HTMLInputElement;
  let renameValue = '';

  const sortOptions: { value: SortField; label: string; icon: string }[] = [
    { value: 'name', label: 'Name', icon: 'Aa' },
    { value: 'modified', label: 'Date', icon: '📅' },
    { value: 'size', label: 'Size', icon: '📊' },
    { value: 'type', label: 'Type', icon: '📎' }
  ];

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

  function formatSize(bytes: number | undefined): string {
    if (bytes === undefined || bytes === null) return '';
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

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

  function handleSortClick(field: SortField) {
    toggleSort(field);
  }

  export function getContainerWidth(): number {
    return listContainer?.clientWidth || 800;
  }
</script>

<div class="list-wrapper">
  <div class="list-toolbar">
    <div class="sort-buttons">
      {#each sortOptions as option}
        <button 
          class="sort-btn"
          class:active={$sortConfig.field === option.value}
          on:click={() => handleSortClick(option.value)}
          title="Sort by {option.label}"
        >
          <span class="sort-icon">{option.icon}</span>
          <span class="sort-label">{option.label}</span>
          {#if $sortConfig.field === option.value}
            <span class="sort-arrow">{$sortConfig.direction === 'asc' ? '↑' : '↓'}</span>
          {/if}
        </button>
      {/each}
    </div>
    
    <div class="toolbar-right">
      <span class="file-count">{files.length} items</span>
    </div>
  </div>

  <div
    class="list-container"
    bind:this={listContainer}
    on:click={handleBackgroundClick}
    role="presentation"
  >
    {#if isLoading && !files.length}
      <div class="loading">Loading...</div>
    {:else}
      {#each files as file, i (file.name)}
        <div
          id="file-btn-{i}"
          class="list-item"
          class:selected={$selectedFiles.has(file.name)}
          class:focused={$focusedIndex === i}
          class:being-dragged={$fileDragDrop.draggedFiles.includes(file.name)}
          class:drag-over={$fileDragDrop.currentDropTarget === file.name && file.is_dir}
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
          <span class="icon">
            {#if $thumbnails.has(file.path)}
              <img src={$thumbnails.get(file.path)} alt="" class="thumbnail" />
            {:else}
              {getIcon(file)}
            {/if}
          </span>
          
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
            <span class="name" title={file.name}>{file.name}</span>
          {/if}
          
          <span class="size">{file.is_dir ? '' : formatSize(file.size)}</span>
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
  .list-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .list-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: var(--bg-main);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    min-height: 32px;
  }

  .sort-buttons {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .sort-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .sort-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .sort-btn.active {
    background: var(--selection);
    border-color: var(--border-focus);
    color: var(--text-main);
  }

  .sort-icon {
    font-size: 11px;
    opacity: 0.8;
  }

  .sort-label {
    font-weight: 500;
  }

  .sort-arrow {
    font-size: 10px;
    opacity: 0.7;
    margin-left: 2px;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .file-count {
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0.8;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .loading {
    padding: 20px;
    color: var(--text-muted);
  }

  .list-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: default;
    color: var(--text-muted);
    user-select: none;
    -webkit-user-select: none;
    transition: background-color 0.1s;
  }

  .list-item:hover {
    background-color: var(--hover-bg);
    color: var(--text-main);
  }

  .list-item.selected {
    background-color: var(--selection);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
  }

  .list-item.focused {
    outline: 1px dotted var(--border-focus);
    outline-offset: -1px;
  }

  .list-item.being-dragged {
    opacity: 0.4;
  }

  .list-item.drag-over {
    background-color: rgba(59, 130, 246, 0.3);
    border: 2px dashed #3b82f6;
  }

  .icon {
    font-size: 20px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .thumbnail {
    width: 24px;
    height: 24px;
    object-fit: cover;
    border-radius: 2px;
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .size {
    width: 80px;
    text-align: right;
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .rename-input {
    flex: 1;
    font-size: 13px;
    background: var(--bg-main);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
    border-radius: 2px;
    padding: 2px 6px;
    outline: none;
  }
</style>