<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import ImageViewer from '../../components/media/ImageViewer.svelte';
  import VideoPlayer from '../../components/media/VideoPlayer.svelte';

  let filePath = '';
  let fileName = '';
  let fileType: 'image' | 'video' = 'image';
  let mediaSrc = '';
  let imageMeta = { width: 0, height: 0, size: '0 MB', type: '' };

  const appWindow = getCurrentWebviewWindow();

  onMount(() => {
    // Get parameters from URL
    const params = $page.url.searchParams;
    filePath = params.get('filePath') || '';
    fileName = params.get('fileName') || '';
    fileType = (params.get('type') as 'image' | 'video') || 'image';

    // Convert file path for web display
    mediaSrc = convertFileSrc(filePath);

    // Set image metadata
    const ext = fileName.split('.').pop()?.toUpperCase() || '';
    imageMeta = { width: 0, height: 0, size: '0 MB', type: ext };
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      appWindow.close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="media-window">
  {#if mediaSrc}
    {#if fileType === 'image'}
      <ImageViewer 
        src={mediaSrc}
        filename={fileName}
        meta={imageMeta}
      />
    {:else if fileType === 'video'}
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
</style>