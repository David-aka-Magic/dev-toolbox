<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { activeTab, fileTabs } from "$lib/stores/fileTabStore";
  import { tick } from "svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { currentView } from '$lib/stores/viewStore';
  import { convertFileSrc } from "@tauri-apps/api/core";

  let files: any[] = [];
  let isLoading = false;
  let imageThumbnails = new Map<string, string>(); // Store image thumbnails
  let loadingThumbnails = new Set<string>(); // Track which thumbnails are currently loading

  // Image extensions to show as thumbnails
  const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'];
  // Video extensions
  const videoExtensions = ['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv', 'm4v'];
  
  // Limit concurrent thumbnail loads
  let thumbnailQueue: Array<{path: string, name: string}> = [];
  let activeLoads = 0;
  const MAX_CONCURRENT_LOADS = 5;

  // --- VIDEO HOVER PLAYBACK STATE ---
  let videoPreviewCache = new Map<string, string>(); // fileName -> preview video path
  let loadingVideoPreviews = new Set<string>();
  let activeVideos = new Set<HTMLVideoElement>();
  let hoverTimeouts = new Map<string, number>();
  const MAX_ACTIVE_VIDEOS = 3;
  const HOVER_DELAY = 250; // ms - debounce delay before starting playback
  const MAX_PREVIEW_CACHE = 10; // Max number of video previews to keep in memory

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
    
    // Clean up active videos
    console.log('🧹 Cleaning up active videos:', activeVideos.size);
    activeVideos.forEach(video => {
      video.pause();
      video.currentTime = 0;
    });
    activeVideos.clear();
    
    // Clear hover timeouts
    console.log('⏱️ Clearing hover timeouts:', hoverTimeouts.size);
    hoverTimeouts.forEach(timeout => clearTimeout(timeout));
    hoverTimeouts.clear();
    
    try {
      files = await invoke("read_directory", { path });
      
      // Queue image and video thumbnails for lazy loading
      for (const file of files) {
        if (!file.is_dir) {
          if (isImageFile(file.name)) {
            thumbnailQueue.push({ path: file.path, name: file.name });
          } else if (isVideoFile(file.name)) {
            console.log('📹 Found video file:', file.name);
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
        console.log('🎬 Loading video thumbnail for:', fileName);
        // Extract static video thumbnail for initial display
        base64 = await invoke<string>('extract_video_thumbnail', { path: filePath });
        mimeType = 'image/png';
        console.log('✅ Video thumbnail loaded for:', fileName);
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

  // --- VIDEO PREVIEW GENERATION ---
  async function generateVideoPreview(filePath: string, fileName: string): Promise<string> {
    console.log('🎥 generateVideoPreview called for:', fileName);
    
    if (videoPreviewCache.has(fileName)) {
      console.log('💾 Using cached preview for:', fileName);
      return videoPreviewCache.get(fileName)!;
    }

    if (loadingVideoPreviews.has(fileName)) {
      console.log('⏳ Already generating preview for:', fileName, '- waiting...');
      // Wait for existing generation to complete
      return new Promise((resolve) => {
        const checkInterval = setInterval(() => {
          if (videoPreviewCache.has(fileName)) {
            clearInterval(checkInterval);
            console.log('✅ Preview generation completed while waiting:', fileName);
            resolve(videoPreviewCache.get(fileName)!);
          }
        }, 100);
      });
    }

    loadingVideoPreviews.add(fileName);
    console.log('🔧 Starting preview generation for:', fileName);

    try {
      // Generate optimized preview clip using GPU-accelerated encoding
      console.log('📞 Calling generate_video_preview backend command...');
      const previewPath = await invoke<string>('generate_video_preview', {
        path: filePath,
        maxDuration: 3,
        resolution: 160,
        fps: 15,
        useHardwareAccel: true
      });
      
      console.log('✅ Backend returned preview path:', previewPath);

      // Cache management - remove oldest if cache is full
      if (videoPreviewCache.size >= MAX_PREVIEW_CACHE) {
        const firstKey = videoPreviewCache.keys().next().value;
        if (firstKey) {
          console.log('🗑️ Cache full, removing oldest:', firstKey);
          videoPreviewCache.delete(firstKey);
        }
      }

      videoPreviewCache.set(fileName, previewPath);
      console.log('💾 Cached preview for:', fileName);
      return previewPath;
    } catch (err) {
      console.error(`❌ Failed to generate video preview for ${fileName}:`, err);
      throw err;
    } finally {
      loadingVideoPreviews.delete(fileName);
    }
  }

  // --- VIDEO HOVER HANDLERS ---
  async function handleVideoHoverStart(videoElement: HTMLVideoElement, fileName: string, filePath: string) {
    console.log('🖱️ Mouse entered video element:', fileName);
    
    // Debounce: only start if user hovers for HOVER_DELAY ms
    const timeout = window.setTimeout(async () => {
      console.log('⏰ Debounce timeout triggered for:', fileName);
      
      try {
        // Generate or get cached preview
        console.log('🎬 Generating/getting preview for:', fileName);
        const previewPath = await generateVideoPreview(filePath, fileName);
        console.log('📁 Preview path:', previewPath);
        
        // Convert to file URL that Tauri can load
        const videoSrc = convertFileSrc(previewPath);
        console.log('🔗 Converted to asset URL:', videoSrc);
        
        // Set video source if not already set
        if (videoElement.src !== videoSrc) {
          console.log('📝 Setting video src attribute');
          videoElement.src = videoSrc;
        } else {
          console.log('ℹ️ Video src already set');
        }

        // Limit concurrent playback
        if (activeVideos.size >= MAX_ACTIVE_VIDEOS) {
          console.log('⚠️ Max active videos reached, pausing oldest');
          // Pause oldest video
          const oldestVideo = activeVideos.values().next().value;
          if (oldestVideo) {
            oldestVideo.pause();
            oldestVideo.currentTime = 0;
            activeVideos.delete(oldestVideo);
          }
        }

        activeVideos.add(videoElement);
        console.log('▶️ Attempting to play video for:', fileName);
        console.log('📊 Active videos:', activeVideos.size);
        
        await videoElement.play().catch(err => {
          console.warn('⚠️ Video play failed:', err);
        });
        
        console.log('✅ Video play() called successfully for:', fileName);
      } catch (err) {
        console.error('❌ Failed to start video preview:', err);
      }
    }, HOVER_DELAY);

    hoverTimeouts.set(fileName, timeout);
    console.log('⏱️ Hover timeout set for:', fileName, '- delay:', HOVER_DELAY, 'ms');
  }

  function handleVideoHoverEnd(videoElement: HTMLVideoElement, fileName: string) {
    console.log('🖱️ Mouse left video element:', fileName);
    
    // Clear debounce timeout
    const timeout = hoverTimeouts.get(fileName);
    if (timeout) {
      console.log('🚫 Clearing hover timeout for:', fileName);
      clearTimeout(timeout);
      hoverTimeouts.delete(fileName);
    }

    // Stop playback
    console.log('⏸️ Pausing and resetting video:', fileName);
    videoElement.pause();
    videoElement.currentTime = 0;
    activeVideos.delete(videoElement);
    console.log('📊 Active videos after hover end:', activeVideos.size);
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
            if (dotIndex > 0) {
                creationInput.setSelectionRange(0, dotIndex);
            } else {
                creationInput.select();
            }
        } else {
            creationInput.select();
        }
    }
  }

  async function confirmCreation() {
    if (!creationState || !creationName.trim()) {
        cancelCreation();
        return;
    }
    
    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    try {
        let actualName: string;
        if (creationState.type === 'folder') {
            actualName = await invoke<string>('create_directory', { path: currentPath, name: creationName });
        } else {
            actualName = await invoke<string>('create_file', { path: currentPath, name: creationName });
        }
        await loadFiles(currentPath, actualName);
    } catch (err) {
        alert('Error creating: ' + err);
    } finally {
        creationState = null;
        creationName = "";
    }
  }

  function cancelCreation() {
    creationState = null;
    creationName = "";
  }

  async function submitRename() {
    if (!renamingFile) return;
    const newName = renameInput?.value.trim();
    if (!newName || newName === renamingFile) {
        renamingFile = null;
        return;
    }

    const currentPath = $activeTab?.path;
    if (!currentPath) return;

    const fullPath = joinPath(currentPath, renamingFile);
    try {
        await invoke('rename_item', { path: fullPath, newName });
        await loadFiles(currentPath, newName);
    } catch (err) {
        alert('Rename error: ' + err);
    } finally {
        renamingFile = null;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (renamingFile || creationState || focusedIndex === -1) return;
    
    if (['ArrowUp','ArrowDown','ArrowLeft','ArrowRight'].includes(event.key)) {
      event.preventDefault();
      const itemWidth = 100;
      const containerWidth = gridContainer?.clientWidth || 800;
      const columns = Math.floor(containerWidth / itemWidth);
      
      if (event.key === 'ArrowRight') focusedIndex = Math.min(files.length - 1, focusedIndex + 1);
      if (event.key === 'ArrowLeft') focusedIndex = Math.max(0, focusedIndex - 1);
      if (event.key === 'ArrowDown') focusedIndex = Math.min(files.length - 1, focusedIndex + columns);
      if (event.key === 'ArrowUp') focusedIndex = Math.max(0, focusedIndex - columns);
      
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
              {:else if isVideoFile(file.name)}
                <!-- Video with hover playback -->
                <div class="video-container">
                  {#if imageThumbnails.has(file.name)}
                    <!-- Show static thumbnail initially -->
                    <img 
                      src={imageThumbnails.get(file.name)} 
                      alt={file.name} 
                      class="thumbnail video-poster" 
                      class:hidden={loadingVideoPreviews.has(file.name)}
                    />
                  {:else if loadingThumbnails.has(file.name)}
                    <div class="loading-indicator">⏳</div>
                  {:else}
                    🎬
                  {/if}
                  
                  <!-- Video element for hover playback (GPU-accelerated) -->
                  <video
                    class="thumbnail video-preview"
                    muted
                    loop
                    preload="none"
                    on:mouseenter={(e) => handleVideoHoverStart(e.currentTarget, file.name, file.path)}
                    on:mouseleave={(e) => handleVideoHoverEnd(e.currentTarget, file.name)}
                  >
                    <!-- Fallback content -->
                    <track kind="captions" />
                  </video>
                </div>
              {:else if imageThumbnails.has(file.name)}
                <img src={imageThumbnails.get(file.name)} alt={file.name} class="thumbnail" />
              {:else if loadingThumbnails.has(file.name)}
                <div class="loading-indicator">⏳</div>
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
                <span class="label">{file.name}</span>
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
    justify-content: center; 
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
    pointer-events: none;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 60px;
    height: 60px;
  }

  .video-container {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: auto; /* CRITICAL: Allow pointer events for video hover */
  }

  .thumbnail {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 4px;
    pointer-events: none;
  }

  .video-poster {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 1;
  }

  .video-poster.hidden {
    display: none;
  }

  .video-preview {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 2;
    opacity: 1 !important;
    transition: opacity 0.2s;
    pointer-events: auto !important; /* CRITICAL: Override .thumbnail's pointer-events: none */
  }

  .video-preview[src]:not([src=""]) {
    opacity: 1;
  }

  .loading-indicator {
    font-size: 24px;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
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
  

  
</style>