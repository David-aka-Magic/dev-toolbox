<script lang="ts">
  import { thumbnailLoader } from '../filesystem/fileoperations/useThumbnailLoader';
  import { isVideoFile } from '../filesystem/fileoperations/fileUtils';

  export let file: any;
    export let index: number;
    export let isSelected: boolean = false;
    export let isFocused: boolean = false;
    export let isBeingDragged: boolean = false;
    export let isRenaming: boolean = false;
    export let renameValue: string = '';
  
    // Callback props (Svelte 5 style)
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
  
    $: if (isRenaming && renameInput) {
      renameInput.focus();
      renameInput.select();
    }
  
    $: thumbnail = $thumbnailLoader.thumbnails.get(file.name);
    $: isLoadingThumbnail = $thumbnailLoader.loadingSet.has(file.name);
  
    function handleClick(event: MouseEvent) {
      event.stopPropagation(); // Prevent bubbling to grid container
      onclick?.({ event, index, fileName: file.name });
    }
  
    function handleKeyDown(event: KeyboardEvent) {
      onkeydown?.({ event, index, fileName: file.name });
    }
  
    function handleDblClick() {
      ondblclick?.({ file });
    }
  
    function handleContextMenu(event: MouseEvent) {
      event.preventDefault();
      event.stopPropagation();
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
    <div class="icon">
      {#if file.is_dir}
        📁
      {:else if thumbnail}
        <img src={thumbnail} alt={file.name} class="thumbnail" />
      {:else if isLoadingThumbnail}
        <div class="loading-indicator">⏳</div>
      {:else if isVideoFile(file.name)}
        🎬
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
      <span class="label">{file.name}</span>
    {/if}
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
  
    .icon {
      font-size: 32px;
      margin-bottom: 5px;
      display: flex;
      align-items: center;
      justify-content: center;
      width: 48px;
      height: 48px;
    }
  
    .thumbnail {
      max-width: 48px;
      max-height: 48px;
      object-fit: cover;
      border-radius: 2px;
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