<script lang="ts">
  import { onMount } from 'svelte';

  // Props
  export let src: string = '';
  export let filename: string = 'recording.mp4';
  
  // Video State
  let video: HTMLVideoElement;
  let container: HTMLDivElement;
  let duration = 0;
  let currentTime = 0;
  let paused = true;
  let volume = 1;
  let playbackRate = 1.0;
  
  // Developer Tools State
  let scale = 1;
  let panning = false;
  let pointX = 0; 
  let pointY = 0;
  let startX = 0; 
  let startY = 0;

  // LOOP STATE (NEW)
  let loopStart = 0;
  let loopEnd = 0;
  let isLooping = false;

  const FRAME_STEP = 1 / 30; 

  // --- VIDEO LOGIC ---
  function togglePlay() {
    if (video.paused) video.play();
    else video.pause();
  }

  // UPDATED: Handle Looping Logic
  function handleTimeUpdate() {
    if (isLooping && loopEnd > 0) {
      if (currentTime >= loopEnd || currentTime < loopStart) {
        video.currentTime = loopStart;
        // If we hit the end, ensure we keep playing
        if (video.paused) video.play();
      }
    }
  }

  function toggleLoop() {
    isLooping = !isLooping;
    if (!isLooping) {
      // Reset loop points when turning off? Optional. 
      // Keeping them lets user toggle back on easily.
    } else {
      // If turning on and no points set, set to full duration
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
    video.currentTime = loopStart; // Jump to start to test loop
    video.play();
  }

  // NEW: Snapshot Feature
  function takeSnapshot() {
    if (!video) return;
    const canvas = document.createElement('canvas');
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
      const dataURL = canvas.toDataURL('image/png');
      
      // Create fake link to download
      const a = document.createElement('a');
      a.href = dataURL;
      a.download = `snapshot_${Math.floor(currentTime)}.png`;
      a.click();
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

  // --- PAN & ZOOM LOGIC ---
  function onMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    panning = true;
    startX = e.clientX - pointX;
    startY = e.clientY - pointY;
    container.style.cursor = 'grabbing';
  }

  function onMouseUp() {
    panning = false;
    container.style.cursor = 'default';
  }

  function onMouseMove(e: MouseEvent) {
    if (!panning) return;
    e.preventDefault();
    pointX = e.clientX - startX;
    pointY = e.clientY - startY;
  }

  function onWheel(e: WheelEvent) {
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
    scale = 1; pointX = 0; pointY = 0; playbackRate = 1.0;
  }

  // Keyboard Shortcuts
  function onKeyDown(e: KeyboardEvent) {
    if (e.key.toLowerCase() === 'i') setInPoint();
    if (e.key.toLowerCase() === 'o') setOutPoint();
    if (e.key.toLowerCase() === 's') takeSnapshot();
  }
</script>

<svelte:window on:mouseup={onMouseUp} on:mousemove={onMouseMove} on:keydown={onKeyDown}/>

<div 
  class="player-wrapper" 
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
      on:timeupdate={handleTimeUpdate}
      on:click|stopPropagation={togglePlay}
    ></video>
  </div>

  <div class="hud top">
    <span class="filename">{filename}</span>
    <span class="meta">{video?.videoWidth || 0}x{video?.videoHeight || 0}</span>
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
        <button class="icon-btn sm" class:active={isLooping} on:click={toggleLoop} title="Toggle Loop">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/></svg>
        </button>
        
        <button class="text-btn" on:click={setInPoint} title="Set In Point (I)">IN</button>
        <button class="text-btn" on:click={setOutPoint} title="Set Out Point (O)">OUT</button>

        <div class="separator"></div>
        
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
    width: 100%; height: 100%; position: relative; overflow: hidden;
    background: #000; user-select: none;
  }
  .video-layer {
    width: 100%; height: 100%;
    display: flex; align-items: center; justify-content: center;
    transform-origin: 0 0; will-change: transform;
  }
  video {
    max-width: 100%; max-height: 100%;
    box-shadow: 0 0 50px rgba(0,0,0,0.5); pointer-events: none;
  }

  /* --- HUD --- */
  .hud {
    position: absolute; left: 50%; transform: translateX(-50%);
    background: rgba(20, 20, 20, 0.85); backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1); color: white; pointer-events: auto;
  }
  .top { top: 20px; border-radius: 20px; padding: 6px 16px; display: flex; gap: 15px; font-size: 13px; }
  .filename { font-weight: 600; color: #fff; }
  .meta { color: #888; font-family: monospace; }
  .bottom { bottom: 20px; width: 90%; max-width: 800px; border-radius: 12px; padding: 12px 20px; display: flex; flex-direction: column; gap: 8px; }

  /* --- CONTROLS --- */
  .scrubber-track { width: 100%; position: relative; display: flex; align-items: center; height: 10px; }
  
  .scrubber {
    width: 100%; height: 4px; border-radius: 2px;
    accent-color: #3b82f6; cursor: pointer; position: relative; z-index: 10;
  }
  
  /* Loop Region Indicator */
  .loop-region {
    position: absolute; top: 3px; height: 4px;
    background: rgba(59, 130, 246, 0.5); /* Blue tint */
    border-left: 2px solid #3b82f6;
    border-right: 2px solid #3b82f6;
    z-index: 5; pointer-events: none;
  }

  .controls-row { display: flex; justify-content: space-between; align-items: center; }
  .group { display: flex; align-items: center; gap: 10px; }

  .icon-btn {
    background: transparent; border: none; color: #ddd;
    cursor: pointer; padding: 6px; border-radius: 4px;
    display: flex; align-items: center; justify-content: center;
  }
  .icon-btn:hover { background: rgba(255,255,255,0.1); color: #fff; }
  .icon-btn.active { color: #3b82f6; background: rgba(59, 130, 246, 0.15); }
  
  .text-btn {
    background: transparent; border: 1px solid rgba(255,255,255,0.2);
    color: #aaa; font-size: 10px; padding: 2px 6px; border-radius: 3px;
    cursor: pointer; font-weight: 600;
  }
  .text-btn:hover { border-color: #fff; color: #fff; }

  .time-readout { font-family: 'Fira Code', monospace; font-size: 13px; color: #fff; margin-left: 10px; }
  .dim { color: #666; }
  .select-wrapper select {
    background: rgba(255,255,255,0.05); border: none; color: #ccc;
    font-size: 12px; padding: 4px 8px; border-radius: 4px; outline: none; cursor: pointer;
  }
  .separator { width: 1px; height: 16px; background: rgba(255,255,255,0.2); margin: 0 5px; }
</style>