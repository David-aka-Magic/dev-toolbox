<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

  export let src: string = '';
  export let filename: string = 'recording.mp4';
  
  let video: HTMLVideoElement;
  let container: HTMLDivElement;
  let duration = 0;
  let currentTime = 0;
  let paused = true;
  let volume = 1;
  let playbackRate = 1.0;
  let muted = false;
  
  let scale = 1;
  let panning = false;
  let pointX = 0; 
  let pointY = 0;
  let startX = 0; 
  let startY = 0;

  let loopStart = 0;
  let loopEnd = 0;
  let isLooping = false;
  
  let isLocked = false;
  let isFullscreen = false;
  let showVolumeSlider = false;

  let videoAspect = 16/9;
  let containerAspect = 16/9;
  let fitMode: 'width' | 'height' = 'width';

  const FRAME_STEP = 1 / 30;

  onMount(() => {
    updateContainerSize();
    window.addEventListener('resize', updateContainerSize);
    return () => window.removeEventListener('resize', updateContainerSize);
  });

  function updateContainerSize() {
    if (container) {
      containerAspect = container.clientWidth / container.clientHeight;
      calculateFitMode();
    }
  }

  function handleVideoLoaded() {
    if (video) {
      videoAspect = video.videoWidth / video.videoHeight;
      calculateFitMode();
    }
  }

  function calculateFitMode() {
    if (videoAspect > containerAspect) {
      fitMode = 'width';
    } else {
      fitMode = 'height';
    }
  }

  function togglePlay() {
    if (video.paused) video.play();
    else video.pause();
  }

  function handleTimeUpdate() {
    if (isLooping && loopEnd > 0) {
      if (currentTime >= loopEnd || currentTime < loopStart) {
        video.currentTime = loopStart;
        if (video.paused) video.play();
      }
    }
  }

  function toggleLoop() {
    isLooping = !isLooping;
    if (isLooping) {
      if (loopEnd === 0) loopEnd = duration;
    }
  }

  function setInPoint() {
    loopStart = currentTime;
    isLooping = true;
    if (loopEnd === 0 || loopEnd < loopStart) loopEnd = duration;
  }

  function setOutPoint() {
    loopEnd = currentTime;
    isLooping = true;
    if (loopStart > loopEnd) loopStart = 0;
    video.currentTime = loopStart;
    video.play();
  }

  function takeSnapshot() {
    if (!video) return;
    const canvas = document.createElement('canvas');
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
      const dataURL = canvas.toDataURL('image/png');
      const a = document.createElement('a');
      a.href = dataURL;
      const timestamp = formatTime(currentTime).replace(/:/g, '-').replace(/\./g, '_');
      a.download = `${filename.replace(/\.[^/.]+$/, '')}_${timestamp}.png`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
    }
  }

  function stepFrame(direction: number) {
    video.pause();
    video.currentTime += (direction * FRAME_STEP);
  }

  function formatTime(seconds: number) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 100);
    return `${m}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
  }

  function toggleMute() {
    muted = !muted;
    video.muted = muted;
  }

  function handleVolumeChange(e: Event) {
    const target = e.target as HTMLInputElement;
    volume = parseFloat(target.value);
    video.volume = volume;
    if (volume > 0 && muted) {
      muted = false;
      video.muted = false;
    }
  }

  function toggleLock() {
    isLocked = !isLocked;
    if (isLocked) {
      resetView();
    }
  }

  async function toggleFullscreen() {
    try {
      const appWindow = getCurrentWebviewWindow();
      isFullscreen = !isFullscreen;
      
      if (isFullscreen) {
        await appWindow.setDecorations(false);
        await appWindow.setFullscreen(true);
      } else {
        await appWindow.setFullscreen(false);
        await appWindow.setDecorations(true);
      }
    } catch (e) {
      console.error('Fullscreen toggle failed:', e);
      if (container) {
        if (!document.fullscreenElement) {
          container.requestFullscreen();
          isFullscreen = true;
        } else {
          document.exitFullscreen();
          isFullscreen = false;
        }
      }
    }
  }

  function onMouseDown(e: MouseEvent) {
    if (isLocked) return;
    if (e.button !== 0) return;
    e.preventDefault();
    panning = true;
    startX = e.clientX - pointX;
    startY = e.clientY - pointY;
    container.style.cursor = 'grabbing';
  }

  function onMouseUp() {
    panning = false;
    if (container) container.style.cursor = isLocked ? 'default' : 'grab';
  }

  function onMouseMove(e: MouseEvent) {
    if (!panning || isLocked) return;
    e.preventDefault();
    pointX = e.clientX - startX;
    pointY = e.clientY - startY;
  }

  function onWheel(e: WheelEvent) {
    if (isLocked) return;
    e.preventDefault();
    const xs = (e.clientX - pointX) / scale;
    const ys = (e.clientY - pointY) / scale;
    const delta = -e.deltaY;
    const factor = delta > 0 ? 1.1 : 0.9;
    
    let newScale = scale * factor;
    if (newScale < 0.5) newScale = 0.5;
    if (newScale > 10) newScale = 10;

    pointX = e.clientX - xs * newScale;
    pointY = e.clientY - ys * newScale;
    scale = newScale;
  }

  function resetView() {
    scale = 1; 
    pointX = 0; 
    pointY = 0; 
    playbackRate = 1.0;
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key.toLowerCase() === 'i') setInPoint();
    if (e.key.toLowerCase() === 'o') setOutPoint();
    if (e.key.toLowerCase() === 's') takeSnapshot();
    if (e.key.toLowerCase() === 'l') toggleLock();
    if (e.key.toLowerCase() === 'f') toggleFullscreen();
    if (e.key.toLowerCase() === 'm') toggleMute();
    if (e.key === ' ') { e.preventDefault(); togglePlay(); }
    if (e.key === 'Escape' && isFullscreen) toggleFullscreen();
  }
</script>

<svelte:window on:mouseup={onMouseUp} on:mousemove={onMouseMove} on:keydown={onKeyDown}/>

<div 
  class="player-wrapper" 
  class:locked={isLocked}
  bind:this={container}
  on:mousedown={onMouseDown}
  on:wheel|nonpassive={onWheel}
>
  
  <div 
    class="video-layer"
    style:transform="translate3d({pointX}px, {pointY}px, 0) scale({scale})"
  >
    <video 
      bind:this={video}
      {src}
      bind:currentTime
      bind:duration
      bind:paused
      bind:volume
      bind:playbackRate
      on:loadedmetadata={handleVideoLoaded}
      on:timeupdate={handleTimeUpdate}
      on:click|stopPropagation={togglePlay}
      class:fit-width={fitMode === 'width'}
      class:fit-height={fitMode === 'height'}
    ></video>
  </div>

  <div class="hud top">
    <span class="filename">{filename}</span>
    <span class="meta">{video?.videoWidth || 0}x{video?.videoHeight || 0}</span>
    {#if isLocked}
      <span class="badge locked-badge">LOCKED</span>
    {/if}
    {#if isFullscreen}
      <span class="badge fullscreen-badge">FULLSCREEN</span>
    {/if}
  </div>

  <div class="hud bottom" on:mousedown|stopPropagation>
    
    <div class="scrubber-track">
      {#if isLooping && duration > 0}
        <div 
          class="loop-region"
          style="
            left: {(loopStart / duration) * 100}%; 
            width: {((loopEnd - loopStart) / duration) * 100}%;
          "
        ></div>
      {/if}

      <input 
        type="range" 
        min="0" 
        max={duration} 
        step="0.01"
        bind:value={currentTime} 
        class="scrubber"
      />
    </div>

    <div class="controls-row">
      <div class="group">
        <button class="icon-btn" on:click={togglePlay}>
          {#if paused}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
          {:else}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
          {/if}
        </button>

        <button class="icon-btn sm" on:click={() => stepFrame(-1)} title="Prev Frame">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 19l-7-7 7-7m8 14l-7-7 7-7" /></svg>
        </button>
        <button class="icon-btn sm" on:click={() => stepFrame(1)} title="Next Frame">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 5l7 7-7 7M5 5l7 7-7 7" /></svg>
        </button>
        
        <span class="time-readout">
          {formatTime(currentTime)} <span class="dim">/ {formatTime(duration)}</span>
        </span>
      </div>

      <div class="group">
        <div 
          class="volume-control"
          on:mouseenter={() => showVolumeSlider = true}
          on:mouseleave={() => showVolumeSlider = false}
        >
          <button class="icon-btn sm" on:click={toggleMute} title="Toggle Mute (M)">
            {#if muted || volume === 0}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>
            {:else if volume < 0.5}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
            {:else}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
            {/if}
          </button>
          
          {#if showVolumeSlider}
            <div class="volume-slider-popup">
              <input 
                type="range" 
                min="0" 
                max="1" 
                step="0.01"
                value={volume}
                on:input={handleVolumeChange}
                class="volume-slider"
              />
              <span class="volume-value">{Math.round(volume * 100)}%</span>
            </div>
          {/if}
        </div>

        <div class="separator"></div>

        <button class="icon-btn sm" class:active={isLooping} on:click={toggleLoop} title="Toggle Loop">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/></svg>
        </button>
        
        <button class="text-btn" on:click={setInPoint} title="Set In Point (I)">IN</button>
        <button class="text-btn" on:click={setOutPoint} title="Set Out Point (O)">OUT</button>

        <div class="separator"></div>

        <button class="icon-btn sm" class:active={isLocked} on:click={toggleLock} title="Toggle Lock (L)">
          {#if isLocked}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 9.9-1"/></svg>
          {/if}
        </button>

        <button class="icon-btn sm" on:click={toggleFullscreen} title="Fullscreen Borderless (F)">
          {#if isFullscreen}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"/></svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/></svg>
          {/if}
        </button>
        
        <button class="icon-btn sm" on:click={takeSnapshot} title="Take Snapshot (S)">
           <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>
        </button>

        <div class="select-wrapper">
          <select bind:value={playbackRate}>
            <option value={0.25}>0.25x</option>
            <option value={0.5}>0.5x</option>
            <option value={1.0}>1.0x</option>
            <option value={1.5}>1.5x</option>
            <option value={2.0}>2.0x</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .player-wrapper {
    width: 100%; 
    height: 100%; 
    position: relative; 
    overflow: hidden;
    background: #000; 
    user-select: none; 
    cursor: grab;
  }
  .player-wrapper.locked {
    cursor: default;
  }
  .video-layer {
    width: 100%; 
    height: 100%;
    display: flex; 
    align-items: center; 
    justify-content: center;
    transform-origin: 0 0; 
    will-change: transform;
  }
  video {
    box-shadow: 0 0 50px rgba(0,0,0,0.5); 
    pointer-events: none;
  }
  video.fit-width {
    width: 100%;
    height: auto;
    max-height: 100%;
  }
  video.fit-height {
    height: 100%;
    width: auto;
    max-width: 100%;
  }

  .hud {
    position: absolute; 
    left: 50%; 
    transform: translateX(-50%);
    background: rgba(20, 20, 20, 0.85); 
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1); 
    color: white; 
    pointer-events: auto;
  }
  .top { 
    top: 20px; 
    border-radius: 20px; 
    padding: 6px 16px; 
    display: flex; 
    gap: 15px; 
    font-size: 13px; 
    align-items: center; 
  }
  .filename { font-weight: 600; color: #fff; }
  .meta { color: #888; font-family: monospace; }
  .bottom { 
    bottom: 20px; 
    width: 90%; 
    max-width: 800px; 
    border-radius: 12px; 
    padding: 12px 20px; 
    display: flex; 
    flex-direction: column; 
    gap: 8px; 
  }

  .badge {
    font-size: 10px; 
    font-weight: 700; 
    padding: 2px 6px; 
    border-radius: 4px;
  }
  .locked-badge {
    background: #ef4444; 
    color: white;
  }
  .fullscreen-badge {
    background: #8b5cf6; 
    color: white;
  }

  .scrubber-track { 
    width: 100%; 
    position: relative; 
    display: flex; 
    align-items: center; 
    height: 10px; 
  }
  
  .scrubber {
    width: 100%; 
    height: 4px; 
    border-radius: 2px;
    accent-color: #3b82f6; 
    cursor: pointer; 
    position: relative; 
    z-index: 10;
  }
  
  .loop-region {
    position: absolute; 
    top: 3px; 
    height: 4px;
    background: rgba(59, 130, 246, 0.5);
    border-left: 2px solid #3b82f6;
    border-right: 2px solid #3b82f6;
    z-index: 5; 
    pointer-events: none;
  }

  .controls-row { 
    display: flex; 
    justify-content: space-between; 
    align-items: center; 
  }
  .group { 
    display: flex; 
    align-items: center; 
    gap: 10px; 
  }

  .icon-btn {
    background: transparent; 
    border: none; 
    color: #ddd;
    cursor: pointer; 
    padding: 6px; 
    border-radius: 4px;
    display: flex; 
    align-items: center; 
    justify-content: center;
  }
  .icon-btn:hover { 
    background: rgba(255,255,255,0.1); 
    color: #fff; 
  }
  .icon-btn.active { 
    color: #3b82f6; 
    background: rgba(59, 130, 246, 0.15); 
  }
  
  .text-btn {
    background: transparent; 
    border: 1px solid rgba(255,255,255,0.2);
    color: #aaa; 
    font-size: 10px; 
    padding: 2px 6px; 
    border-radius: 3px;
    cursor: pointer; 
    font-weight: 600;
  }
  .text-btn:hover { 
    border-color: #fff; 
    color: #fff; 
  }

  .time-readout { 
    font-family: 'Fira Code', monospace; 
    font-size: 13px; 
    color: #fff; 
    margin-left: 10px; 
  }
  .dim { color: #666; }
  .select-wrapper select {
    background: rgba(255,255,255,0.05); 
    border: none; 
    color: #ccc;
    font-size: 12px; 
    padding: 4px 8px; 
    border-radius: 4px; 
    outline: none; 
    cursor: pointer;
  }
  .separator { 
    width: 1px; 
    height: 16px; 
    background: rgba(255,255,255,0.2); 
    margin: 0 5px; 
  }

  .volume-control {
    position: relative;
    display: flex;
    align-items: center;
  }

  .volume-slider-popup {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(20, 20, 20, 0.95);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 8px;
    padding: 12px 8px;
    margin-bottom: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .volume-slider {
    width: 80px;
    height: 4px;
    accent-color: #3b82f6;
    cursor: pointer;
  }

  .volume-value {
    font-size: 11px;
    color: #888;
    font-family: monospace;
  }
</style>