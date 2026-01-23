<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { mediaViewer } from "$lib/stores/mediaViewerStore";
  import ImageViewer from "./ImageViewer.svelte";
  import VideoPlayer from "./VideoPlayer.svelte";

  // Convert file path to web-compatible format and prepare metadata
  $: mediaPath = $mediaViewer.filePath ? convertFileSrc($mediaViewer.filePath) : '';
  
  // For ImageViewer, we need to prepare metadata
  // You can enhance this by actually reading file info
  $: imageMeta = {
    width: 0,  // Will be filled by img.onload
    height: 0, // Will be filled by img.onload
    size: '0 MB', // Could be calculated from file system
    type: getFileExtension($mediaViewer.fileName || '')
  };
  
  function getFileExtension(filename: string): string {
    const ext = filename.split('.').pop()?.toUpperCase() || '';
    return ext;
  }
  
  function handleClose() {
    mediaViewer.close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $mediaViewer.isOpen}
  <div class="media-overlay">
    <button class="close-btn" on:click={handleClose} title="Close (Esc)">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>

    <div class="media-content">
      {#if $mediaViewer.fileType === 'image'}
        <ImageViewer 
          src={mediaPath}
          filename={$mediaViewer.fileName || 'image'}
          meta={imageMeta}
        />
      {:else if $mediaViewer.fileType === 'video'}
        <VideoPlayer 
          src={mediaPath}
          filename={$mediaViewer.fileName || 'video'}
        />
      {/if}
    </div>
  </div>
{/if}

<style>
  .media-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.95);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn {
    position: absolute;
    top: 20px;
    right: 20px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: white;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    z-index: 1001;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: scale(1.1);
  }

  .media-content {
    width: 100%;
    height: 100%;
  }
</style>