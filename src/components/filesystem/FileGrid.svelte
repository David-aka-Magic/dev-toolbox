<script lang="ts">
  import { fileSelection, selectedFiles, focusedIndex } from './hooks/useFileSelection';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import { settings } from '$lib/stores/settingsStore';
  import FileGridItem from './FileGridItem.svelte';
  import FileCreationDialog from './FileCreationDialog.svelte';

  export let files: any[] = [];
  export let isLoading: boolean = false;
  export let renamingFile: string | null = null;
  export let creationType: 'folder' | 'file' | null = null;
  export let folderSizes: Map<string, number | null> = new Map();

  // Callback props
  export let onbackgroundclick: (() => void) | undefined = undefined;
  export let onitemdblclick: ((detail: any) => void) | undefined = undefined;
  export let onitemcontextmenu: ((detail: any) => void) | undefined = undefined;
  export let onitemdrop: ((detail: any) => void) | undefined = undefined;
  export let onrenamesubmit: ((detail: any) => void) | undefined = undefined;
  export let oncreationconfirm: ((detail: any) => void) | undefined = undefined;
  export let oncreationcancel: (() => void) | undefined = undefined;

  let gridContainer: HTMLDivElement;

  // Drag selection state
  let isDragSelecting = false;
  let dragStarted = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragCurrentX = 0;
  let dragCurrentY = 0;

  // Calculate grid item size based on icon size setting
  $: iconSize = $settings.fileGridIconSize;
  $: itemWidth = Math.max(90, iconSize + 42);
  $: itemHeight = iconSize + 60;

  function handleBackgroundClick(event: MouseEvent) {
    // If we just finished drag selection, don't clear - just reset the flag
    if (justFinishedDragSelection) {
      justFinishedDragSelection = false;
      return;
    }
    if (event.target === event.currentTarget) {
      onbackgroundclick?.();
    }
  }

  let justFinishedDragSelection = false;

  function handleMouseDown(event: MouseEvent) {
    // Allow drag selection if clicking on the grid container or empty space within it
    const target = event.target as HTMLElement;
    const isGridOrEmpty = target === gridContainer || target.classList.contains('grid-container');
    
    // Don't start drag selection if clicking on a file item
    if (!isGridOrEmpty) return;
    if (event.button !== 0) return;
    
    isDragSelecting = true;
    dragStarted = false;
    
    const rect = gridContainer.getBoundingClientRect();
    dragStartX = event.clientX - rect.left + gridContainer.scrollLeft;
    dragStartY = event.clientY - rect.top + gridContainer.scrollTop;
    dragCurrentX = dragStartX;
    dragCurrentY = dragStartY;
    
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragSelecting) return;
    if (!gridContainer) return;
    
    const rect = gridContainer.getBoundingClientRect();
    dragCurrentX = event.clientX - rect.left + gridContainer.scrollLeft;
    dragCurrentY = event.clientY - rect.top + gridContainer.scrollTop;
    
    const distance = Math.sqrt(
      Math.pow(dragCurrentX - dragStartX, 2) + 
      Math.pow(dragCurrentY - dragStartY, 2)
    );
    
    if (distance > 5) {
      dragStarted = true;
    }
    
    if (dragStarted) {
      updateDragSelection();
    }
  }

  function handleMouseUp(event: MouseEvent) {
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
    
    // If we actually dragged to select files, set the flag to prevent clearSelection on click
    if (dragStarted && $selectedFiles.size > 0) {
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

    // Clear and rebuild selection
    const filesToSelect: { name: string; index: number }[] = [];

    files.forEach((file, i) => {
      const itemEl = document.getElementById(`file-btn-${i}`);
      if (!itemEl) return;

      const itemRect = itemEl.getBoundingClientRect();
      const containerRect = gridContainer.getBoundingClientRect();
      
      const itemLeft = itemRect.left - containerRect.left + gridContainer.scrollLeft;
      const itemTop = itemRect.top - containerRect.top + gridContainer.scrollTop;
      const itemRight = itemLeft + itemRect.width;
      const itemBottom = itemTop + itemRect.height;

      const intersects = !(itemRight < minX || itemLeft > maxX || itemBottom < minY || itemTop > maxY);
      
      if (intersects) {
        filesToSelect.push({ name: file.name, index: i });
      }
    });

    // Set selection in one operation
    fileSelection.setSelection(new Set(filesToSelect.map(f => f.name)));
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

  export function getContainerWidth(): number {
    return gridContainer?.clientWidth || 800;
  }
</script>

<div
  class="grid-container"
  bind:this={gridContainer}
  on:click={handleBackgroundClick}
  on:mousedown={handleMouseDown}
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

<style>
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