<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { tick } from "svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { currentView } from '$lib/stores/viewStore';

  let files: any[] = [];
  let isLoading = false;
  let imageThumbnails = new Map<string, string>(); // Store image thumbnails
  let loadingThumbnails = new Set<string>(); // Track which thumbnails are currently loading

  // Image extensions to show as thumbnails
  const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'];
  // Video extensions - just show icon, don't load
  const videoExtensions = ['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv', 'm4v'];
  
  // Limit concurrent thumbnail loads
  let thumbnailQueue: Array<{path: string, name: string}> = [];
  let activeLoads = 0;
  const MAX_CONCURRENT_LOADS = 5;

  // --- SELECTION STATE ---
  let selectedFiles = new Set<string>();
  let focusedIndex: number = -1;
  let lastSelectedIndex: number = -1;
  let gridContainer: HTMLDivElement;

  // --- CONTEXT MENU STATE ---
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let menuTargetFile: string | null = null;

  // --- RENAME / CREATION STATE ---
  let renamingFile: string | null = null;
  let renameInput: HTMLInputElement;
  let creationState: { type: 'folder' | 'file' } | null = null;
  let creationInput: HTMLInputElement;
  let creationName = "";

  // --- DRAG AND DROP STATE ---
  let draggedFile: string | null = null;
  let draggedFilePath: string | null = null;
  let draggedIsDir: boolean = false;
  let currentDropTarget: string | null = null;

  // --- DRAG SELECTION STATE ---
  let isDragSelecting = false;
  let dragSelectStart: { x: number; y: number } | null = null;
  let dragSelectEnd: { x: number; y: number } | null = null;
  let dragSelectBox: DOMRect | null = null;

  // --- LOADING ---
  $: if ($activeTab && $activeTab.path) {
    loadFiles($activeTab.path);
  }

  async function loadFiles(path: string, targetSelect: string | null = null) {
    isLoading = true;
    if (!targetSelect) {
        selectedFiles.clear();
        focusedIndex = -1;
    }
    renamingFile = null;
    creationState = null;
    draggedFile = null;
    draggedFilePath = null;
    imageThumbnails.clear();
    loadingThumbnails.clear();
    thumbnailQueue = [];
    activeLoads = 0;
    
    try {
      files = await invoke("read_directory", { path });
      
      // Queue image and video thumbnails for lazy loading
      for (const file of files) {
        if (!file.is_dir) {
          if (isImageFile(file.name)) {
            thumbnailQueue.push({ path: file.path, name: file.name });
          } else if (isVideoFile(file.name)) {
            thumbnailQueue.push({ path: file.path, name: file.name });
          }
        }
      }
      
      // Start loading thumbnails
      processQueue();
      
      if (targetSelect) {
          const index = files.findIndex(f => f.name === targetSelect);
          if (index !== -1) {
              selectedFiles.clear();
              selectedFiles.add(targetSelect);
              focusedIndex = index;
              await tick();
              const el = document.getElementById(`file-btn-${index}`);
              el?.scrollIntoView({ block: 'center' });
          }
      }
    } catch (err) {
      console.error("Failed to load directory:", err);
    } finally {
      isLoading = false;
    }
  }

  async function processQueue() {
    while (thumbnailQueue.length > 0 && activeLoads < MAX_CONCURRENT_LOADS) {
      const item = thumbnailQueue.shift();
      if (item) {
        activeLoads++;
        loadThumbnail(item.path, item.name);
      }
    }
  }

  function isImageFile(filename: string): boolean {
    const ext = filename.split('.').pop()?.toLowerCase();
    return ext ? imageExtensions.includes(ext) : false;
  }

  function isVideoFile(filename: string): boolean {
    const ext = filename.split('.').pop()?.toLowerCase();
    return ext ? videoExtensions.includes(ext) : false;
  }

  async function loadThumbnail(filePath: string, fileName: string) {
    if (loadingThumbnails.has(fileName)) return;
    loadingThumbnails.add(fileName);
    
    try {
      let base64: string;
      let mimeType: string;
      
      if (isVideoFile(fileName)) {
        // Extract video thumbnail
        base64 = await invoke<string>('extract_video_thumbnail', { path: filePath });
        mimeType = 'image/png';
      } else {
        // Load image directly
        base64 = await invoke<string>('read_file_base64', { path: filePath });
        const ext = fileName.split('.').pop()?.toLowerCase();
        mimeType = ext === 'svg' ? 'image/svg+xml' : `image/${ext}`;
      }
      
      imageThumbnails.set(fileName, `data:${mimeType};base64,${base64}`);
      imageThumbnails = imageThumbnails;
    } catch (err) {
      console.error(`Failed to load thumbnail for ${fileName}:`, err);
    } finally {
      loadingThumbnails.delete(fileName);
      activeLoads--;
      processQueue();
    }
  }

  // Helper function for cross-platform path joining
  function joinPath(base: string, name: string): string {
    base = base.replace(/[/\\]$/, '');
    const separator = base.includes('/') ? '/' : '\\';
    return `${base}${separator}${name}`;
  }

  // --- MOUSE HANDLERS ---
  function handleItemClick(event: MouseEvent, index: number, fileName: string) {
    if (creationState) confirmCreation();
    if (showMenu) showMenu = false;
    
    const isCtrl = event.ctrlKey || event.metaKey;
    const isShift = event.shiftKey;

    if (isShift && lastSelectedIndex !== -1) {
      const start = Math.min(lastSelectedIndex, index);
      const end = Math.max(lastSelectedIndex, index);
      if (!isCtrl) selectedFiles.clear();
      for (let i = start; i <= end; i++) selectedFiles.add(files[i].name);
    } else if (isCtrl) {
      if (selectedFiles.has(fileName)) selectedFiles.delete(fileName);
      else { selectedFiles.add(fileName); lastSelectedIndex = index; }
    } else {
      selectedFiles.clear(); selectedFiles.add(fileName); lastSelectedIndex = index;
    }
    focusedIndex = index;
    selectedFiles = selectedFiles; 
  }

  function handleItemKey(event: KeyboardEvent, index: number, fileName: string) {
    if (event.key === 'Enter') {
        if (selectedFiles.size > 0) handleDblClick(files[index]);
    }
    if (event.key === ' ') {
        event.preventDefault();
        handleItemClick(new MouseEvent('click'), index, fileName);
    }
  }

  function handleBackgroundClick(event: MouseEvent) {
    if (creationState) confirmCreation();
    if (event.target === event.currentTarget) {
      selectedFiles.clear();
      selectedFiles = selectedFiles;
      focusedIndex = -1;
      showMenu = false;
    }
  }

  function handleDblClick(file: any) {
    if (renamingFile || creationState) return;
    if (file.is_dir) {
      fileTabs.updateActivePath(file.path);
    } else {
      // Double-click on file opens it in editor
      openInEditor(file);
    }
  }

  async function openInEditor(file: any) {
    try {
      const content = await invoke<string>('read_file', { path: file.path });
      
      const newTab = {
        id: crypto.randomUUID(),
        name: file.name,
        path: file.path,
        content: `<p>${content.replace(/\n/g, '</p><p>')}</p>`,
        isDirty: false
      };
      
      editorTabs.update(tabs => [...tabs, newTab]);
      activeEditorTabId.set(newTab.id);
      currentView.set('editor'); // Switch to editor view
    } catch (err) {
      alert(`Failed to open file: ${err}`);
    }
  }

  function handleContextMenu(e: MouseEvent, fileName: string | null) {
    e.preventDefault();
    e.stopPropagation();
    if (creationState) return;

    menuX = e.clientX;
    menuY = e.clientY;
    menuTargetFile = fileName;

    if (fileName && !selectedFiles.has(fileName)) {
        selectedFiles.clear();
        selectedFiles.add(fileName);
        selectedFiles = selectedFiles;
    }
    showMenu = true;
  }

  // --- DRAG SELECTION HANDLERS ---
  function handleMouseDown(e: MouseEvent) {
    // Only start drag selection on background clicks (not on items)
    if (e.target !== e.currentTarget) return;
    
    // Don't interfere with context menu
    if (e.button === 2) return;
    
    isDragSelecting = true;
    dragSelectStart = { x: e.clientX, y: e.clientY };
    dragSelectEnd = { x: e.clientX, y: e.clientY };
    
    // Clear selection unless ctrl/cmd is held
    if (!e.ctrlKey && !e.metaKey) {
      selectedFiles.clear();
      selectedFiles = selectedFiles;
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragSelecting || !dragSelectStart) return;
    
    dragSelectEnd = { x: e.clientX, y: e.clientY };
    
    // Calculate selection box
    const left = Math.min(dragSelectStart.x, dragSelectEnd.x);
    const top = Math.min(dragSelectStart.y, dragSelectEnd.y);
    const width = Math.abs(dragSelectEnd.x - dragSelectStart.x);
    const height = Math.abs(dragSelectEnd.y - dragSelectStart.y);
    
    dragSelectBox = new DOMRect(left, top, width, height);
    
    // Check which items intersect with selection box
    const tempSelection = new Set<string>();
    files.forEach((file, i) => {
      const element = document.getElementById(`file-btn-${i}`);
      if (element) {
        const rect = element.getBoundingClientRect();
        if (boxesIntersect(dragSelectBox!, rect)) {
          tempSelection.add(file.name);
        }
      }
    });
    
    selectedFiles = tempSelection;
  }

  function handleMouseUp(e: MouseEvent) {
    isDragSelecting = false;
    dragSelectStart = null;
    dragSelectEnd = null;
    dragSelectBox = null;
  }

  function boxesIntersect(box1: DOMRect, box2: DOMRect): boolean {
    return !(
      box1.right < box2.left ||
      box1.left > box2.right ||
      box1.bottom < box2.top ||
      box1.top > box2.bottom
    );
  }

  // --- DRAG AND DROP HANDLERS (CONTAINER-BASED) ---

  function handleDragStart(event: DragEvent, file: any) {
    if (!event.dataTransfer) return;
    
    console.log("🚀 DRAG START:", file.name, "IsDir:", file.is_dir);

    draggedFile = file.name;
    draggedFilePath = file.path;
    draggedIsDir = file.is_dir;

    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", file.name);

    // Create a custom drag image
    const dragImg = document.createElement('div');
    dragImg.style.position = 'absolute';
    dragImg.style.top = '-1000px';
    dragImg.style.padding = '8px 12px';
    dragImg.style.background = 'rgba(59, 130, 246, 0.9)';
    dragImg.style.color = 'white';
    dragImg.style.borderRadius = '4px';
    dragImg.style.fontSize = '12px';
    dragImg.textContent = `${file.is_dir ? '📁' : '📄'} ${file.name}`;
    document.body.appendChild(dragImg);
    event.dataTransfer.setDragImage(dragImg, 0, 0);
    setTimeout(() => document.body.removeChild(dragImg), 0);
  }

  function handleDragEnd(event: DragEvent) {
    console.log("🏁 DRAG END");
    
    draggedFile = null;
    draggedFilePath = null;
    draggedIsDir = false;
    currentDropTarget = null;
  }

  // Individual item drag handlers
  function handleItemDragEnter(event: DragEvent, file: any) {
    console.log("👋 ITEM ENTER:", file.name);
    if (!draggedFile || !file.is_dir || file.name === draggedFile) return;
    event.preventDefault();
    event.stopPropagation();
  }

  function handleItemDragOver(event: DragEvent, file: any) {
    if (!draggedFile || !file.is_dir || file.name === draggedFile) return;
    
    console.log("🔄 ITEM OVER:", file.name);
    event.preventDefault();
    event.stopPropagation();
    
    const element = event.currentTarget as HTMLElement;
    element.classList.add('drag-over');
    
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  function handleItemDragLeave(event: DragEvent) {
    const element = event.currentTarget as HTMLElement;
    element.classList.remove('drag-over');
  }

  async function handleItemDrop(event: DragEvent, targetFolder: any) {
    event.preventDefault();
    event.stopPropagation();
    
    const element = event.currentTarget as HTMLElement;
    element.classList.remove('drag-over');

    if (!targetFolder.is_dir || !draggedFile) return;

    console.log("💧 DROP on:", targetFolder.name);

    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    const sourcePath = draggedFilePath || joinPath(currentPath, draggedFile);
    const destPath = targetFolder.path;

    if (sourcePath === destPath) {
      console.log("⚠️ Cannot drop folder into itself");
      return;
    }

    if (draggedIsDir && destPath.startsWith(sourcePath)) {
      alert("Cannot move a folder into its own subdirectory");
      return;
    }

    console.log("📦 Moving:", sourcePath, "->", destPath);

    try {
      await invoke('move_item', { source: sourcePath, destination: destPath });
      await loadFiles(currentPath);
    } catch (err) {
      console.error("❌ Move error:", err);
      alert("Move failed: " + err);
    }
  }

  // --- MENU ACTIONS ---
  async function handleMenuAction(action: string) {
    showMenu = false;
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    if (action === 'refresh') loadFiles(currentPath);

    if (menuTargetFile) {
        const fullPath = joinPath(currentPath, menuTargetFile);
        const targetFile = files.find(f => f.name === menuTargetFile);
        
        if (action === 'open_in_editor' && targetFile) {
            openInEditor(targetFile);
        }
        if (action === 'rename') {
            renamingFile = menuTargetFile;
            await tick();
            renameInput?.focus();
            renameInput?.select();
        }
        if (action === 'delete') {
            if (confirm(`Delete "${menuTargetFile}"?`)) {
                try {
                    await invoke('delete_item', { path: fullPath });
                    loadFiles(currentPath);
                } catch (err) { alert('Error deleting: ' + err); }
            }
        }
    } else {
        if (action === 'new_folder') startCreation('folder');
        if (action === 'new_file') startCreation('file');
    }
  }

  // --- CREATION & RENAME LOGIC ---
  async function startCreation(type: 'folder' | 'file') {
    creationState = { type };
    creationName = type === 'folder' ? "New Folder" : "New Text Document.txt";
    await tick();
    if (creationInput) {
        creationInput.focus();
        if (type === 'file') {
            const dotIndex = creationName.lastIndexOf('.');
            if (dotIndex > 0) creationInput.setSelectionRange(0, dotIndex);
            else creationInput.select();
        } else { creationInput.select(); }
        creationInput.scrollIntoView({ block: "nearest" });
    }
  }

  async function confirmCreation() {
    if (!creationState || !creationName.trim()) { creationState = null; return; }
    const currentPath = $activeTab?.path;
    const name = creationName.trim();
    const type = creationState.type;
    creationState = null; 

    if (currentPath) {
        try {
            if (type === 'folder') await invoke('create_directory', { path: currentPath, name });
            else await invoke('create_file', { path: currentPath, name });
            await loadFiles(currentPath, name);
        } catch (err) { alert(`Error creating ${type}: ${err}`); }
    }
  }

  function cancelCreation() { creationState = null; }

  async function submitRename() {
    if (!renamingFile || !renameInput) return;
    const newName = renameInput.value.trim();
    if (newName && newName !== renamingFile) {
        const currentPath = $activeTab?.path;
        const oldPath = joinPath(currentPath!, renamingFile);
        try {
            await invoke('rename_item', { path: oldPath, newName });
            await loadFiles(currentPath!, newName); 
        } catch (err) { alert("Rename failed: " + err); }
    }
    renamingFile = null;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (renamingFile || creationState) {
        if (event.key === 'Escape') { renamingFile = null; cancelCreation(); }
        return; 
    }
    if (files.length === 0) return;
    if (event.key === 'Backspace') { fileTabs.goBack(); return; }
    
    if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
      event.preventDefault();
      if (focusedIndex === -1) focusedIndex = 0; 
      else {
        const itemWidth = 100; 
        const containerWidth = gridContainer?.clientWidth || 800;
        const columns = Math.floor(containerWidth / itemWidth);
        if (event.key === 'ArrowRight') focusedIndex = Math.min(files.length - 1, focusedIndex + 1);
        if (event.key === 'ArrowLeft') focusedIndex = Math.max(0, focusedIndex - 1);
        if (event.key === 'ArrowDown') focusedIndex = Math.min(files.length - 1, focusedIndex + columns);
        if (event.key === 'ArrowUp') focusedIndex = Math.max(0, focusedIndex - columns);
      }
      if (!event.ctrlKey) {
        if (event.shiftKey) selectedFiles.add(files[focusedIndex].name);
        else { selectedFiles.clear(); selectedFiles.add(files[focusedIndex].name); }
        selectedFiles = selectedFiles;
        const button = document.getElementById(`file-btn-${focusedIndex}`);
        button?.scrollIntoView({ block: 'nearest' });
      }
    }
    if (event.key === 'a' && (event.ctrlKey || event.metaKey)) {
        event.preventDefault();
        files.forEach(f => selectedFiles.add(f.name));
        selectedFiles = selectedFiles;
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} on:dragover={(e) => { console.log('WINDOW DRAGOVER'); e.preventDefault(); }} />

<div 
  class="file-manager" 
  on:click={handleBackgroundClick}
  on:contextmenu={(e) => handleContextMenu(e, null)} 
  role="presentation"
>
  <div 
    class="grid-container" 
    bind:this={gridContainer}
    on:mousedown={handleMouseDown}
    on:mousemove={handleMouseMove}
    on:mouseup={handleMouseUp}
    on:mouseleave={handleMouseUp}
  >
    {#if isLoading && !files.length}
        <div class="loading">Loading...</div>
    {:else}
        {#each files as file, i}
        <div 
            id="file-btn-{i}"
            class="grid-item" 
            class:selected={selectedFiles.has(file.name)}
            class:focused={focusedIndex === i}
            class:being-dragged={draggedFile === file.name}
            
            draggable="true"
            on:dragstart={(e) => handleDragStart(e, file)}
            on:dragend={handleDragEnd}
            on:dragenter={(e) => handleItemDragEnter(e, file)}
            on:dragover={(e) => handleItemDragOver(e, file)}
            on:dragleave={handleItemDragLeave}
            on:drop={(e) => handleItemDrop(e, file)}
            
            on:click={(e) => handleItemClick(e, i, file.name)}
            on:keydown={(e) => handleItemKey(e, i, file.name)}
            on:dblclick={() => handleDblClick(file)}
            on:contextmenu|stopPropagation={(e) => handleContextMenu(e, file.name)}
            
            role="button"
            tabindex="0"
        >
          
            <div class="icon">
              {#if file.is_dir}
                📁
              {:else if imageThumbnails.has(file.name)}
                <img src={imageThumbnails.get(file.name)} alt={file.name} class="thumbnail" />
              {:else if loadingThumbnails.has(file.name)}
                <div class="loading-indicator">⏳</div>
              {:else if isVideoFile(file.name)}
                🎬
              {:else}
                📄
              {/if}
            </div>

            {#if renamingFile === file.name}
                <input 
                    bind:this={renameInput}
                    type="text" 
                    value={file.name} 
                    class="rename-input"
                    on:click|stopPropagation
                    on:blur={submitRename}
                    on:keydown={(e) => e.key === 'Enter' && submitRename()}
                />
            {:else}
                <span 
                    class="label" 
                    class:label-selected={selectedFiles.has(file.name)}
                >
                    {file.name}
                </span>
            {/if}
        </div>
        {/each}

        {#if creationState}
            <div 
                class="grid-item selected creating" 
                on:click|stopPropagation
                role="button"
                tabindex="0"
            >
                <div class="icon">
                    {creationState.type === 'folder' ? '📁' : '📄'}
                </div>
                <input 
                    bind:this={creationInput}
                    bind:value={creationName}
                    type="text" 
                    class="rename-input"
                    on:click|stopPropagation
                    on:keydown={(e) => {
                        if (e.key === 'Enter') confirmCreation();
                        if (e.key === 'Escape') cancelCreation();
                        e.stopPropagation();
                    }}
                    on:blur={confirmCreation} 
                />
            </div>
        {/if}

    {/if}

    <!-- Drag Selection Box -->
    {#if isDragSelecting && dragSelectStart && dragSelectEnd}
      <div 
        class="selection-box"
        style="
          left: {Math.min(dragSelectStart.x, dragSelectEnd.x)}px;
          top: {Math.min(dragSelectStart.y, dragSelectEnd.y)}px;
          width: {Math.abs(dragSelectEnd.x - dragSelectStart.x)}px;
          height: {Math.abs(dragSelectEnd.y - dragSelectStart.y)}px;
        "
      ></div>
    {/if}
  </div>

  {#if showMenu}
    <ContextMenu 
      x={menuX} 
      y={menuY} 
      options={menuTargetFile ? 
        (files.find(f => f.name === menuTargetFile)?.is_dir ? 
          [
            { label: 'Rename', action: 'rename' },
            { label: 'Delete', action: 'delete', danger: true }
          ] : 
          [
            { label: 'Open in Editor', action: 'open_in_editor' },
            { label: 'Rename', action: 'rename' },
            { label: 'Delete', action: 'delete', danger: true }
          ]
        ) : 
        [
          { label: 'New Folder', action: 'new_folder' },
          { label: 'New File', action: 'new_file' },
          { label: 'SEPARATOR', action: '' },
          { label: 'Refresh', action: 'refresh' }
        ]
      }
      on:close={() => showMenu = false}
      on:click={(e) => handleMenuAction(e.detail)}
    />
  {/if}
</div>

<style>
  .file-manager { 
    height: 100%; 
    width: 100%; 
    display: flex; 
    flex-direction: column; 
    background-color: var(--bg-main); 
    color: var(--text-main);
    position: relative;
    z-index: 10000; /* CRITICAL: Higher than drag-handle-top */
  }

  .grid-container { 
    flex: 1; 
    overflow-y: auto; 
    padding: 10px; 
    display: grid; 
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr)); 
    grid-auto-rows: 100px; 
    gap: 10px; 
    align-content: start;
  }

  .loading { 
    padding: 20px; 
    color: var(--text-muted); 
  }

  .grid-item { 
    background-color: rgba(0,0,0,0.01);
    border: 2px solid transparent; 
    box-sizing: border-box; 
    border-radius: 4px; 
    display: flex; 
    flex-direction: column; 
    align-items: center; 
    justify-content: flex-start; /* Changed from center to flex-start */
    cursor: default; 
    color: var(--text-muted); 
    text-align: center; 
    padding: 5px; 
    position: relative; 
    outline: none;
    user-select: none; 
    -webkit-user-select: none;
    transition: opacity 0.2s, transform 0.2s;
  }

  .grid-item.being-dragged {
    opacity: 0.4;
  }

  .grid-item:hover { 
    background-color: var(--hover-bg); 
    color: var(--text-main); 
  }

  .grid-item.selected { 
    background-color: var(--selection); 
    border-color: var(--border-focus); 
    color: var(--text-main); 
    z-index: 100; /* Bring selected items to front so full name appears above other items */
    min-height: 120px; /* Expand height to accommodate longer filenames */
    height: auto; /* Allow height to grow dynamically */
  }

  .grid-item.focused { 
    border: 2px dotted var(--text-muted); 
  }
  
  .grid-item.selected.focused { 
    border: 2px solid var(--border-focus); 
    background-color: rgba(59, 130, 246, 0.4); 
  }

  .grid-item.drag-over {
    background-color: rgba(59, 130, 246, 0.3);
    border: 2px dashed #3b82f6;
    transform: scale(1.05);
  }

  .grid-item.creating {
    background-color: var(--selection); 
    border-color: var(--border-focus); 
    color: var(--text-main);
    opacity: 1; 
  }

  .icon { 
    font-size: 32px; 
    margin-bottom: 6px; 
    margin-top: 5px; /* Add consistent top margin */
    pointer-events: none;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 60px;
    height: 60px;
    flex-shrink: 0; /* Prevent icon from shrinking */
  }

  .thumbnail {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 4px;
    pointer-events: none;
  }

  .loading-indicator {
    font-size: 24px;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  /* Default label - truncated with ellipsis */
  .label { 
    font-size: 12px; 
    word-break: break-word; 
    max-width: 100%; 
    overflow: hidden; 
    text-overflow: ellipsis; 
    display: -webkit-box; 
    -webkit-line-clamp: 2; 
    -webkit-box-orient: vertical;
    pointer-events: none;
  }

  /* Selected label - shows full name expanding downward */
  .label-selected {
    overflow: visible;
    text-overflow: clip;
    display: block;
    -webkit-line-clamp: unset;
    -webkit-box-orient: unset;
    white-space: normal;
    word-wrap: break-word;
    max-height: none;
    position: relative; /* Changed from absolute to relative */
    padding: 4px;
    margin-top: 0; /* Remove extra spacing */
  }

  .rename-input {
    width: 100%;
    box-sizing: border-box; 
    font-size: 12px;
    text-align: center;
    background: var(--bg-input);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
    border-radius: 2px;
    outline: none;
    padding: 2px 0;
    margin-top: 2px;
    pointer-events: auto;
    user-select: text;
    -webkit-user-select: text;
  }

  .grid-item.creating .rename-input {
    background: transparent; 
    color: white;
    border: 1px solid rgba(255,255,255, 0.3);
    box-shadow: none;
  }

  /* Drag Selection Box */
  .selection-box {
    position: fixed;
    border: 2px solid var(--border-focus);
    background-color: rgba(59, 130, 246, 0.2);
    pointer-events: none;
    z-index: 10001;
  }
</style>