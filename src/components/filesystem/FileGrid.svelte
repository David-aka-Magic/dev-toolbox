<script lang="ts">
    import FileGridItem from "./FileGridItem.svelte";
    import FileCreationDialog from "./FileCreationDialog.svelte";
    import {
        fileSelection,
        selectedFiles,
        focusedIndex,
    } from ".//fileoperations/useFileSelection";
    import { fileDragDrop } from ".//fileoperations/useFileDragDrop";

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
     let dragStartX = 0;
     let dragStartY = 0;
     let dragCurrentX = 0;
     let dragCurrentY = 0;
     let dragStarted = false; // Track if we've actually started dragging
     let justFinishedDrag = false; // Track if we just finished a drag (to ignore click)
     const DRAG_THRESHOLD = 5; // Pixels to move before considering it a drag
   
     // Handle background clicks
     function handleBackgroundClick(event: MouseEvent) {
       console.log('FileGrid handleBackgroundClick - justFinishedDrag:', justFinishedDrag);
       
       // If we just finished dragging, ignore the click event and stop it from bubbling
       if (justFinishedDrag) {
         justFinishedDrag = false;
         console.log('Ignoring click after drag');
         event.stopPropagation(); // ADD THIS LINE
         return;
       }
       
       if (event.target === event.currentTarget) {
         console.log('Calling onbackgroundclick callback');
         onbackgroundclick?.();
       }
     }
   
     // Drag selection handlers
     function handleMouseDown(event: MouseEvent) {
       console.log('handleMouseDown called');
       // Only start drag selection if clicking directly on the grid background
       // Check if we clicked on an item or any of its children
       const clickedElement = event.target as HTMLElement;
       const clickedOnItem = clickedElement.closest('.grid-item');
       
       if (clickedOnItem) return; // Don't start drag selection if clicking on an item
       
       // Don't start if right-click
       if (event.button !== 0) return;
       
       console.log('Starting potential drag');
       // Record start position but don't start selecting yet
       const rect = gridContainer.getBoundingClientRect();
       dragStartX = event.clientX - rect.left + gridContainer.scrollLeft;
       dragStartY = event.clientY - rect.top + gridContainer.scrollTop;
       dragCurrentX = dragStartX;
       dragCurrentY = dragStartY;
       dragStarted = false;
       isDragSelecting = true; // Mark that we're potentially dragging
       
       // Add window listeners for mousemove and mouseup
       console.log('Adding window event listeners');
       window.addEventListener('mousemove', handleMouseMove);
       window.addEventListener('mouseup', handleMouseUp);
       console.log('Event listeners added');
     }
   
     function handleMouseMove(event: MouseEvent) {
       console.log('handleMouseMove - isDragSelecting:', isDragSelecting, 'dragStarted:', dragStarted);
       
       if (!isDragSelecting) return;
       
       const rect = gridContainer.getBoundingClientRect();
       dragCurrentX = event.clientX - rect.left + gridContainer.scrollLeft;
       dragCurrentY = event.clientY - rect.top + gridContainer.scrollTop;
       
       // Check if we've moved enough to consider it a drag
       const distanceMoved = Math.sqrt(
         Math.pow(dragCurrentX - dragStartX, 2) + 
         Math.pow(dragCurrentY - dragStartY, 2)
       );
       
       if (!dragStarted && distanceMoved > DRAG_THRESHOLD) {
         // Now we've actually started dragging
         dragStarted = true;
         console.log('Drag started! Distance:', distanceMoved);
         // Clear selection if not holding Ctrl
         if (!event.ctrlKey && !event.metaKey) {
           fileSelection.clearSelection();
         }
       }
       
       if (dragStarted) {
         console.log('About to call updateDragSelection');
         // Calculate which files are in the selection box
         updateDragSelection();
       }
     }
   
     function handleMouseUp(event: MouseEvent) {
       console.log('handleMouseUp - isDragSelecting:', isDragSelecting, 'dragStarted:', dragStarted);
       
       if (isDragSelecting) {
         // If we actually dragged, mark it so we can ignore the upcoming click event
         if (dragStarted) {
           justFinishedDrag = true;
           console.log('Set justFinishedDrag = true');
         }
         
         isDragSelecting = false;
         dragStarted = false;
         
         // Remove window listeners
         window.removeEventListener('mousemove', handleMouseMove);
         window.removeEventListener('mouseup', handleMouseUp);
       }
     }
   
     function updateDragSelection() {
       console.log('updateDragSelection called!');
       const minX = Math.min(dragStartX, dragCurrentX);
       const maxX = Math.max(dragStartX, dragCurrentX);
       const minY = Math.min(dragStartY, dragCurrentY);
       const maxY = Math.max(dragStartY, dragCurrentY);
       
       console.log('Selection box:', { minX, maxX, minY, maxY });
       console.log('Files to check:', files.length);
       
       // Check each file item to see if it intersects with selection box
       files.forEach((file, i) => {
         const element = document.getElementById(`file-btn-${i}`);
         if (!element) {
           console.log(`Element file-btn-${i} not found`);
           return;
         }
         
         const rect = element.getBoundingClientRect();
         const containerRect = gridContainer.getBoundingClientRect();
         
         const itemLeft = rect.left - containerRect.left + gridContainer.scrollLeft;
         const itemRight = itemLeft + rect.width;
         const itemTop = rect.top - containerRect.top + gridContainer.scrollTop;
         const itemBottom = itemTop + rect.height;
         
         const intersects = !(itemRight < minX || itemLeft > maxX || itemBottom < minY || itemTop > maxY);
         
         console.log(`File ${file.name}:`, { itemLeft, itemRight, itemTop, itemBottom, intersects });
         
         if (intersects) {
           console.log('Adding to selection:', file.name);
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