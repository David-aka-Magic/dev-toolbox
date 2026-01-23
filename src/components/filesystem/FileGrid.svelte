<script lang="ts">
  import { fileSelection, selectedFiles, focusedIndex } from './hooks/useFileSelection';
  import { fileDragDrop } from './hooks/useFileDragDrop';
  import FileGridItem from './FileGridItem.svelte';
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

  let gridContainer: HTMLDivElement;

  // Drag selection state
  let isDragSelecting = false;
  let dragStarted = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragCurrentX = 0;
  let dragCurrentY = 0;

  function handleBackgroundClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onbackgroundclick?.();
    }
  }

  function handleMouseDown(event: MouseEvent) {
    if (event.target !== gridContainer) return;
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

  function handleMouseUp() {
    isDragSelecting = false;
    dragStarted = false;
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }

  function updateDragSelection() {
    const minX = Math.min(dragStartX, dragCurrentX);
    const maxX = Math.max(dragStartX, dragCurrentX);
    const minY = Math.min(dragStartY, dragCurrentY);
    const maxY = Math.max(dragStartY, dragCurrentY);

    fileSelection.clearSelection();

    files.forEach((file, i) => {
      const element = document.getElementById(`file-btn-${i}`);
      if (!element) return;

      const rect = element.getBoundingClientRect();
      const containerRect = gridContainer.getBoundingClientRect();
      
      const itemLeft = rect.left - containerRect.left + gridContainer.scrollLeft;
      const itemRight = itemLeft + rect.width;
      const itemTop = rect.top - containerRect.top + gridContainer.scrollTop;
      const itemBottom = itemTop + rect.height;

      const intersects = !(itemRight < minX || itemLeft > maxX || itemBottom < minY || itemTop > maxY);
      
      if (intersects) {
        fileSelection.addToSelection(file.name, i);
      }
    });
  }

  // File item events
  function handleItemClick(detail: any) {
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

  // Drag & drop events
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

  // Rename events
  function handleRenameSubmit(detail: any) {
    onrenamesubmit?.(detail);
  }

  // Creation events
  function handleCreationConfirm(detail: any) {
    oncreationconfirm?.(detail);
  }

  function handleCreationCancel() {
    oncreationcancel?.();
  }

  // Expose grid container for keyboard navigation
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
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    grid-auto-rows: 100px;
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