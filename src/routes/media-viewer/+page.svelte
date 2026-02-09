<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import ImageViewer from '../../components/media/ImageViewer.svelte';
  import VideoPlayer from '../../components/media/VideoPlayer.svelte';

  let filePath = '';
  let fileName = '';
  let fileType: 'image' | 'video' = 'image';
  let mediaSrc = '';
  let imageMeta = { width: 0, height: 0, size: '0 MB', type: '' };
  
  let isTranscoding = false;
  let transcodeError = '';

  const appWindow = getCurrentWebviewWindow();

  onMount(async () => {
    const params = $page.url.searchParams;
    filePath = params.get('filePath') || '';
    fileName = params.get('fileName') || '';
    fileType = (params.get('type') as 'image' | 'video') || 'image';

    const ext = fileName.split('.').pop()?.toUpperCase() || '';
    imageMeta = { width: 0, height: 0, size: '0 MB', type: ext };

    if (fileType === 'video') {
      await loadVideo(filePath);
    } else {
      mediaSrc = convertFileSrc(filePath);
    }
  });

  async function loadVideo(path: string) {
    isTranscoding = true;
    transcodeError = '';
    
    try {
      console.log('[MediaViewerPage] Calling get_playable_video for:', path);
      const playablePath = await invoke<string>('get_playable_video', { path });
      console.log('[MediaViewerPage] Got playable path:', playablePath);
      mediaSrc = convertFileSrc(playablePath);
      console.log('[MediaViewerPage] Final mediaSrc:', mediaSrc);
    } catch (e) {
      console.error('[MediaViewerPage] Failed to load video:', e);
      transcodeError = String(e);
    } finally {
      isTranscoding = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      appWindow.close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="media-window">
  {#if fileType === 'image'}
    {#if mediaSrc}
      <ImageViewer 
        src={mediaSrc}
        filename={fileName}
        meta={imageMeta}
      />
    {/if}
  {:else if fileType === 'video'}
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
    {:else if mediaSrc}
      <VideoPlayer 
        src={mediaSrc}
        filename={fileName}
      />
    {/if}
  {/if}
</div>

<style>
  .media-window {
    width: 100vw;
    height: 100vh;
    background: #000;
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