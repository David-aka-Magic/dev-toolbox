<script lang="ts">
  import { fileSelection, selectedFiles, focusedIndex } from './hooks/useFileSelection';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { settings } from '$lib/stores/settingsStore';
  import { sortConfig, toggleSort, type SortField } from '$lib/stores/viewModeStore';
  import FileGridItem from './FileGridItem.svelte';
  import FileCreationDialog from './FileCreationDialog.svelte';

  export let files: any[] = [];
  export let isLoading: boolean = false;
  export let renamingFile: string | null = null;
  export let creationType: 'folder' | 'file' | null = null;
  export let folderSizes: Map<string, number | null> = new Map();

  export let onbackgroundclick: (() => void) | undefined = undefined;
  export let onitemdblclick: ((detail: any) => void) | undefined = undefined;
  export let onitemcontextmenu: ((detail: any) => void) | undefined = undefined;
  export let onitemdrop: ((detail: any) => void) | undefined = undefined;
  export let onrenamesubmit: ((detail: any) => void) | undefined = undefined;
  export let oncreationconfirm: ((detail: any) => void) | undefined = undefined;
  export let oncreationcancel: (() => void) | undefined = undefined;

  let gridContainer: HTMLDivElement;

  let isDragSelecting = false;
  let dragStarted = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragCurrentX = 0;
  let dragCurrentY = 0;

  $: iconSize = $settings.fileGridIconSize;
  $: itemWidth = Math.max(90, iconSize + 42);
  $: itemHeight = iconSize + 60;

  let justFinishedDragSelection = false;

  const sortOptions: { value: SortField; label: string; icon: string }[] = [
    { value: 'name', label: 'Name', icon: 'Aa' },
    { value: 'modified', label: 'Date', icon: '📅' },
    { value: 'size', label: 'Size', icon: '📊' },
    { value: 'type', label: 'Type', icon: '📎' }
  ];

  function handleBackgroundClick(event: MouseEvent) {
    if (justFinishedDragSelection) {
      justFinishedDragSelection = false;
      return;
    }
    if (event.target === event.currentTarget) {
      onbackgroundclick?.();
    }
  }

  function handleItemClick(detail: any) {
    justFinishedDragSelection = false;
    const { event: mouseEvent, index, fileName } = detail;
    fileSelection.handleClick(mouseEvent, index, fileName, files);
  }

  function handleItemKeyDown(detail: any) {
    const { event: keyEvent, index } = detail;
    
    if (keyEvent.key === 'Enter') {
      if ($selectedFiles.size > 0) {
        onitemdblclick?.({ file: files[index] });
      }
    }
    if (keyEvent.key === ' ') {
      keyEvent.preventDefault();
      fileSelection.handleClick(new MouseEvent('click'), index, files[index].name, files);
    }
  }

  function handleItemDblClick(detail: any) {
    onitemdblclick?.(detail);
  }

  function handleItemContextMenu(detail: any) {
    onitemcontextmenu?.(detail);
  }

  function handleDragStart(detail: any) {
    const { event: dragEvent, file } = detail;
    fileDragDrop.handleDragStart(dragEvent, file, $selectedFiles, files);
  }

  function handleDragEnd(detail: any) {
    fileDragDrop.handleDragEnd();
  }

  function handleDragEnter(detail: any) {
    const { event: dragEvent, file } = detail;
    fileDragDrop.handleItemDragEnter(dragEvent, file);
  }

  function handleDragOver(detail: any) {
    const { event: dragEvent, file } = detail;
    fileDragDrop.handleItemDragOver(dragEvent, file);
  }

  function handleDragLeave(detail: any) {
    const { event: dragEvent } = detail;
    fileDragDrop.handleItemDragLeave(dragEvent);
  }

  function handleDrop(detail: any) {
    onitemdrop?.(detail);
  }

  function handleRenameSubmit(detail: any) {
    onrenamesubmit?.(detail);
  }

  function handleCreationConfirm(detail: any) {
    oncreationconfirm?.(detail);
  }

  function handleCreationCancel() {
    oncreationcancel?.();
  }

  function handleMouseDown(event: MouseEvent) {
    if (event.button !== 0) return;
    if (event.target !== gridContainer) return;
    
    const rect = gridContainer.getBoundingClientRect();
    dragStartX = event.clientX - rect.left + gridContainer.scrollLeft;
    dragStartY = event.clientY - rect.top + gridContainer.scrollTop;
    dragCurrentX = dragStartX;
    dragCurrentY = dragStartY;
    isDragSelecting = true;
    dragStarted = false;

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragSelecting) return;
    
    const rect = gridContainer.getBoundingClientRect();
    dragCurrentX = event.clientX - rect.left + gridContainer.scrollLeft;
    dragCurrentY = event.clientY - rect.top + gridContainer.scrollTop;
    
    if (!dragStarted && (Math.abs(dragCurrentX - dragStartX) > 5 || Math.abs(dragCurrentY - dragStartY) > 5)) {
      dragStarted = true;
    }
    
    if (dragStarted) {
      updateDragSelection();
    }
  }

  function handleMouseUp(event: MouseEvent) {
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);

    if (isDragSelecting && dragStarted) {
      justFinishedDragSelection = true;
    }
    isDragSelecting = false;
    dragStarted = false;
  }

  function updateDragSelection() {
    const minX = Math.min(dragStartX, dragCurrentX);
    const maxX = Math.max(dragStartX, dragCurrentX);
    const minY = Math.min(dragStartY, dragCurrentY);
    const maxY = Math.max(dragStartY, dragCurrentY);

    const filesToSelect: { name: string; index: number }[] = [];

    const gridItems = gridContainer.querySelectorAll('.grid-item');
    const containerRect = gridContainer.getBoundingClientRect();

    gridItems.forEach((item, i) => {
      const rect = item.getBoundingClientRect();
      const file = files[i];
      if (!file) return;

      const itemLeft = rect.left - containerRect.left + gridContainer.scrollLeft;
      const itemRight = rect.right - containerRect.left + gridContainer.scrollLeft;
      const itemTop = rect.top - containerRect.top + gridContainer.scrollTop;
      const itemBottom = rect.bottom - containerRect.top + gridContainer.scrollTop;

      const intersects = !(itemRight < minX || itemLeft > maxX || itemBottom < minY || itemTop > maxY);
      
      if (intersects) {
        filesToSelect.push({ name: file.name, index: i });
      }
    });

    fileSelection.setSelection(new Set(filesToSelect.map(f => f.name)));
  }

  function handleWheel(event: WheelEvent) {
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      
      const delta = event.deltaY > 0 ? -8 : 8;
      const newSize = Math.max(32, Math.min(128, $settings.fileGridIconSize + delta));
      
      settings.update(s => ({
        ...s,
        fileGridIconSize: newSize
      }));
    }
  }

  function handleSortClick(field: SortField) {
    toggleSort(field);
  }

  export function getContainerWidth(): number {
    return gridContainer?.clientWidth || 800;
  }
</script>

<div class="grid-wrapper">
  <div class="grid-toolbar">
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
      <span class="zoom-hint" title="Use Ctrl+Scroll to adjust icon size">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          <line x1="11" y1="8" x2="11" y2="14"></line>
          <line x1="8" y1="11" x2="14" y2="11"></line>
        </svg>
      </span>
    </div>
  </div>

  <div
    class="grid-container"
    bind:this={gridContainer}
    on:click={handleBackgroundClick}
    on:mousedown={handleMouseDown}
    on:wheel={handleWheel}
    role="presentation"
    style="grid-template-columns: repeat(auto-fill, minmax({itemWidth}px, 1fr)); grid-auto-rows: {itemHeight}px;"
  >
    {#if isLoading && !files.length}
      <div class="loading">Loading...</div>
    {:else}
      {#each files as file, i (file.name)}
        <FileGridItem
          {file}
          index={i}
          isSelected={$selectedFiles.has(file.name)}
          isFocused={$focusedIndex === i}
          isBeingDragged={$fileDragDrop.draggedFiles.includes(file.name)}
          isRenaming={renamingFile === file.name}
          renameValue={file.name}
          folderSize={file.is_dir ? folderSizes.get(file.path) : undefined}
          onclick={handleItemClick}
          onkeydown={handleItemKeyDown}
          ondblclick={handleItemDblClick}
          oncontextmenu={handleItemContextMenu}
          ondragstart={handleDragStart}
          ondragend={handleDragEnd}
          ondragenter={handleDragEnter}
          ondragover={handleDragOver}
          ondragleave={handleDragLeave}
          ondrop={handleDrop}
          onrenamesubmit={handleRenameSubmit}
        />
      {/each}

      {#if creationType}
        <FileCreationDialog
          type={creationType}
          onconfirm={handleCreationConfirm}
          oncancel={handleCreationCancel}
        />
      {/if}
    {/if}
    
    {#if isDragSelecting && dragStarted}
      <div 
        class="selection-box"
        style="
          left: {Math.min(dragStartX, dragCurrentX)}px;
          top: {Math.min(dragStartY, dragCurrentY)}px;
          width: {Math.abs(dragCurrentX - dragStartX)}px;
          height: {Math.abs(dragCurrentY - dragStartY)}px;
        "
      ></div>
    {/if}
  </div>
</div>

<style>
  .grid-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .grid-toolbar {
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

  .zoom-hint {
    display: flex;
    align-items: center;
    color: var(--text-muted);
    opacity: 0.5;
    cursor: help;
    transition: opacity 0.15s;
  }

  .zoom-hint:hover {
    opacity: 0.8;
  }

  .grid-container {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: grid;
    gap: 10px;
    align-content: start;
    position: relative;
  }

  .loading {
    padding: 20px;
    color: var(--text-muted);
  }
  
  .selection-box {
    position: absolute;
    border: 2px solid var(--border-focus);
    background-color: rgba(59, 130, 246, 0.1);
    pointer-events: none;
    z-index: 100;
  }
</style>