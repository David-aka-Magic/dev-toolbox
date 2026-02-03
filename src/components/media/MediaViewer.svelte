<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { mediaViewer } from "$lib/stores/mediaViewerStore";
  import ImageViewer from "./ImageViewer.svelte";
  import VideoPlayer from "./VideoPlayer.svelte";

  let videoSrc = $state('');
  let isTranscoding = $state(false);
  let transcodeError = $state('');

  $effect(() => {
    if ($mediaViewer.isOpen && $mediaViewer.fileType === 'video' && $mediaViewer.filePath) {
      loadVideo($mediaViewer.filePath);
    }
  });

  async function loadVideo(filePath: string) {
    isTranscoding = true;
    transcodeError = '';
    videoSrc = '';
    
    try {
      const playablePath = await invoke<string>('get_playable_video', { path: filePath });
      videoSrc = convertFileSrc(playablePath);
    } catch (e) {
      console.error('Failed to load video:', e);
      transcodeError = String(e);
    } finally {
      isTranscoding = false;
    }
  }

  let imagePath = $derived($mediaViewer.filePath ? convertFileSrc($mediaViewer.filePath) : '');
  
  let imageMeta = $derived({
    width: 0,
    height: 0,
    size: '0 MB',
    type: getFileExtension($mediaViewer.fileName || '')
  });
  
  function getFileExtension(filename: string): string {
    const ext = filename.split('.').pop()?.toUpperCase() || '';
    return ext;
  }
  
  function handleClose() {
    mediaViewer.close();
    videoSrc = '';
    isTranscoding = false;
    transcodeError = '';
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
          src={imagePath}
          filename={$mediaViewer.fileName || 'image'}
          meta={imageMeta}
        />
      {:else if $mediaViewer.fileType === 'video'}
        {#if isTranscoding}
          <div class="loading-container">
            <div class="spinner"></div>
            <p>Preparing video...</p>
            <p class="hint">Transcoding to web-compatible format</p>
          </div>
        {:else if transcodeError}
          <div class="error-container">
            <p class="error-title">Failed to load video</p>
            <p class="error-message">{transcodeError}</p>
          </div>
        {:else if videoSrc}
          <VideoPlayer 
            src={videoSrc}
            filename={$mediaViewer.fileName || 'video'}
          />
        {/if}
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

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: white;
    gap: 16px;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 3px solid rgba(255, 255, 255, 0.2);
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-container p {
    margin: 0;
    font-size: 16px;
  }

  .loading-container .hint {
    font-size: 13px;
    color: #888;
  }

  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: white;
    gap: 12px;
    padding: 20px;
    text-align: center;
  }

  .error-title {
    font-size: 18px;
    font-weight: 600;
    color: #ef4444;
    margin: 0;
  }

  .error-message {
    font-size: 14px;
    color: #888;
    margin: 0;
    max-width: 500px;
    word-break: break-word;
  }
</style>