<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { thumbnailLoader } from '../filesystem/hooks/useThumbnailLoader';
  import { isVideoFile } from '../filesystem/hooks/fileUtils';
  import { settings } from '$lib/stores/settingsStore';

  export let file: any;
  export let index: number;
  export let isSelected: boolean = false;
  export let isFocused: boolean = false;
  export let isBeingDragged: boolean = false;
  export let isRenaming: boolean = false;
  export let renameValue: string = '';
  export let folderSize: number | null | undefined = undefined;

  // Callback props
  export let onclick: ((detail: any) => void) | undefined = undefined;
  export let onkeydown: ((detail: any) => void) | undefined = undefined;
  export let ondblclick: ((detail: any) => void) | undefined = undefined;
  export let oncontextmenu: ((detail: any) => void) | undefined = undefined;
  export let ondragstart: ((detail: any) => void) | undefined = undefined;
  export let ondragend: ((detail: any) => void) | undefined = undefined;
  export let ondragenter: ((detail: any) => void) | undefined = undefined;
  export let ondragover: ((detail: any) => void) | undefined = undefined;
  export let ondragleave: ((detail: any) => void) | undefined = undefined;
  export let ondrop: ((detail: any) => void) | undefined = undefined;
  export let onrenamesubmit: ((detail: any) => void) | undefined = undefined;

  let renameInput: HTMLInputElement;

  // Video hover state
  let videoPreviewPath: string | null = null;
  let isGeneratingPreview = false;
  let hoverTimeout: number | null = null;
  const HOVER_DELAY = 250;

  $: if (isRenaming && renameInput) {
    renameInput.focus();
    renameInput.select();
  }

  $: thumbnail = $thumbnailLoader.thumbnails.get(file.name);
  $: isLoadingThumbnail = $thumbnailLoader.loadingSet.has(file.name);
  $: isVideo = isVideoFile(file.name);
  $: iconSize = $settings.fileGridIconSize;
  $: showFolderSize = $settings.fileShowFolderSize;
  $: videoPreviewEnabled = $settings.fileEnableVideoPreview;
  $: videoPreviewDuration = $settings.fileVideoPreviewDuration;
  $: videoPreviewResolution = $settings.fileVideoPreviewResolution;
  $: useHardwareAccel = $settings.fileUseHardwareAccel;

  // Format size for display
  function formatSize(bytes: number | null | undefined): string {
    if (bytes === null || bytes === undefined) return '—';
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // Video preview generation
  async function generateVideoPreview(): Promise<string | null> {
    if (videoPreviewPath) return videoPreviewPath;
    if (isGeneratingPreview) return null;

    isGeneratingPreview = true;
    try {
      const previewPath = await invoke<string>('generate_video_preview', {
        path: file.path,
        maxDuration: videoPreviewDuration,
        resolution: videoPreviewResolution,
        fps: 15,
        useHardwareAccel: useHardwareAccel
      });
      videoPreviewPath = previewPath;
      return previewPath;
    } catch (err) {
      console.error('Failed to generate video preview:', err);
      return null;
    } finally {
      isGeneratingPreview = false;
    }
  }

  async function handleVideoHoverStart(videoEl: HTMLVideoElement) {
    if (!videoPreviewEnabled) return;
    
    hoverTimeout = window.setTimeout(async () => {
      const previewPath = await generateVideoPreview();
      if (previewPath && videoEl) {
        videoEl.src = convertFileSrc(previewPath);
        videoEl.play().catch(() => {});
      }
    }, HOVER_DELAY);
  }

  function handleVideoHoverEnd(videoEl: HTMLVideoElement) {
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
      hoverTimeout = null;
    }
    if (videoEl) {
      videoEl.pause();
      videoEl.currentTime = 0;
      videoEl.src = '';
    }
  }

  function handleClick(event: MouseEvent) {
    onclick?.({ event, index, fileName: file.name });
  }

  function handleKeyDown(event: KeyboardEvent) {
    onkeydown?.({ event, index });
  }

  function handleDblClick(event: MouseEvent) {
    ondblclick?.({ event, file });
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    oncontextmenu?.({ event, fileName: file.name });
  }

  function handleDragStart(event: DragEvent) {
    ondragstart?.({ event, file });
  }

  function handleDragEnd(event: DragEvent) {
    ondragend?.({ event });
  }

  function handleDragEnter(event: DragEvent) {
    ondragenter?.({ event, file });
  }

  function handleDragOver(event: DragEvent) {
    ondragover?.({ event, file });
  }

  function handleDragLeave(event: DragEvent) {
    ondragleave?.({ event });
  }

  function handleDrop(event: DragEvent) {
    ondrop?.({ event, file });
  }

  function handleRenameBlur() {
    onrenamesubmit?.({ newName: renameValue });
  }

  function handleRenameKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      onrenamesubmit?.({ newName: renameValue });
    }
  }

  function handleRenameClick(event: MouseEvent) {
    event.stopPropagation();
  }
</script>

<div
  id="file-btn-{index}"
  class="grid-item"
  class:selected={isSelected}
  class:focused={isFocused}
  class:being-dragged={isBeingDragged}
  draggable="true"
  on:dragstart={handleDragStart}
  on:dragend={handleDragEnd}
  on:dragenter={handleDragEnter}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  on:click={handleClick}
  on:keydown={handleKeyDown}
  on:dblclick={handleDblClick}
  on:contextmenu={handleContextMenu}
  role="button"
  tabindex="0"
>
  <div class="item-content" class:selected={isSelected}>
    <div 
      class="icon" 
      style="font-size: {iconSize * 0.67}px; min-width: {iconSize}px; height: {iconSize}px;"
    >
      {#if file.is_dir}
        📁
      {:else if isVideo}
        <div class="video-container" style="width: {iconSize}px; height: {iconSize}px;">
          {#if thumbnail}
            <img 
              src={thumbnail} 
              alt={file.name} 
              class="thumbnail video-poster"
              style="max-width: {iconSize}px; max-height: {iconSize}px;"
            />
          {:else if isLoadingThumbnail}
            <div class="loading-indicator">⏳</div>
          {:else}
            🎬
          {/if}
          
          <video
            class="thumbnail video-preview"
            muted
            loop
            preload="none"
            on:mouseenter={(e) => handleVideoHoverStart(e.currentTarget)}
            on:mouseleave={(e) => handleVideoHoverEnd(e.currentTarget)}
          >
            <track kind="captions" />
          </video>
        </div>
      {:else if thumbnail}
        <img 
          src={thumbnail} 
          alt={file.name} 
          class="thumbnail" 
          style="max-width: {iconSize}px; max-height: {iconSize}px;"
        />
      {:else if isLoadingThumbnail}
        <div class="loading-indicator">⏳</div>
      {:else}
        📄
      {/if}
    </div>

    {#if isRenaming}
      <input
        bind:this={renameInput}
        bind:value={renameValue}
        type="text"
        class="rename-input"
        on:click={handleRenameClick}
        on:blur={handleRenameBlur}
        on:keydown={handleRenameKeydown}
      />
    {:else}
      <span class="label" title={file.name}>{file.name}</span>
      {#if file.is_dir && showFolderSize && folderSize !== undefined}
        <span class="folder-size">{formatSize(folderSize)}</span>
      {/if}
    {/if}
  </div>
</div>

<style>
  .grid-item {
    background-color: rgba(0, 0, 0, 0.01);
    border: 2px solid transparent;
    box-sizing: border-box;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding-top: 8px;
    cursor: default;
    color: var(--text-muted);
    text-align: center;
    padding: 8px 5px 5px 5px;
    position: relative;
    outline: none;
    user-select: none;
    -webkit-user-select: none;
    transition: opacity 0.2s, transform 0.2s;
    overflow: visible;
  }

  .grid-item.being-dragged {
    opacity: 0.4;
  }

  .grid-item:hover {
    background-color: var(--hover-bg);
    color: var(--text-main);
  }

  .grid-item.selected {
    color: var(--text-main);
    z-index: 10;
    background-color: transparent;
  }

  .item-content {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .item-content.selected {
    background-color: var(--selection);
    border: 2px solid var(--border-focus);
    border-radius: 4px;
    padding: 8px;
    min-width: 90px;
    margin: -10px;
  }

  .item-content.selected .icon {
    margin-bottom: 4px;
  }

  .item-content.selected .label {
    -webkit-line-clamp: unset;
    display: block;
    white-space: normal;
    overflow: visible;
    width: 100%;
    text-align: center;
  }

  .grid-item.focused {
    /* No dotted border - rely on selection state */
  }

  .grid-item.selected.focused {
    background-color: transparent;
  }

  .grid-item.drag-over {
    background-color: rgba(59, 130, 246, 0.3);
    border: 2px dashed #3b82f6;
    transform: scale(1.05);
  }

  .icon {
    margin-bottom: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .video-container {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: auto;
  }

  .thumbnail {
    object-fit: cover;
    border-radius: 2px;
  }

  .video-poster {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 2px;
    z-index: 1;
  }

  .video-preview {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 2px;
    z-index: 2;
    opacity: 1 !important;
    pointer-events: auto !important;
  }

  .loading-indicator {
    font-size: 24px;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .label {
    font-size: 11px;
    word-break: break-word;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .folder-size {
    font-size: 9px;
    color: var(--text-muted);
    margin-top: 2px;
  }

  .rename-input {
    width: 80px;
    font-size: 11px;
    text-align: center;
    background: var(--bg-main);
    color: var(--text-main);
    border: 1px solid var(--border-focus);
    border-radius: 2px;
    padding: 2px 4px;
    outline: none;
  }

  .rename-input:focus {
    border-color: var(--border-focus);
  }
</style>