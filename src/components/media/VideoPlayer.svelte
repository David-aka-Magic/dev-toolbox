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

  // Constants
  const FRAME_STEP = 1 / 30; // Approx 30fps step. For 60fps use 1/60

  // --- VIDEO LOGIC ---
  function togglePlay() {
    if (video.paused) video.play();
    else video.pause();
  }

  function stepFrame(direction: number) {
    video.pause(); // Always pause when stepping
    video.currentTime += (direction * FRAME_STEP);
  }

  function formatTime(seconds: number) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 100);
    return `${m}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
  }

  // --- PAN & ZOOM LOGIC (Reused from Image Viewer) ---
  function onMouseDown(e: MouseEvent) {
    if (e.button !== 0) return; // Only left click
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
    if (newScale < 0.5) newScale = 0.5; // Don't zoom out too far
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
</script>

<svelte:window on:mouseup={onMouseUp} on:mousemove={onMouseMove} />

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
      on:click|stopPropagation={togglePlay}
    ></video>
  </div>

  <div class="hud top">
    <span class="filename">{filename}</span>
    <span class="meta">{video?.videoWidth || 0}x{video?.videoHeight || 0}</span>
  </div>

  <div class="hud bottom" on:mousedown|stopPropagation>
    
    <div class="scrubber-track">
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
        
        <div class="select-wrapper">
          <select bind:value={playbackRate}>
            <option value={0.25}>0.25x</option>
            <option value={0.5}>0.5x</option>
            <option value={1.0}>1.0x</option>
            <option value={1.5}>1.5x</option>
            <option value={2.0}>2.0x</option>
          </select>
        </div>

        <div class="separator"></div>

        <button class="icon-btn sm" on:click={resetView} title="Reset View">
           <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
        </button>

      </div>
    </div>
  </div>

</div>

<style>
  .player-wrapper {
    width: 100%; height: 100%; position: relative; overflow: hidden;
    background: #000;
    user-select: none;
  }

  .video-layer {
    width: 100%; height: 100%;
    display: flex; align-items: center; justify-content: center;
    transform-origin: 0 0; will-change: transform;
  }

  video {
    max-width: 100%; max-height: 100%;
    box-shadow: 0 0 50px rgba(0,0,0,0.5);
    pointer-events: none; /* Let clicks pass to wrapper for pan/zoom */
  }

  /* --- HUD --- */
  .hud {
    position: absolute; left: 50%; transform: translateX(-50%);
    background: rgba(20, 20, 20, 0.85);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1);
    color: white; pointer-events: auto;
  }

  .top {
    top: 20px; border-radius: 20px; padding: 6px 16px;
    display: flex; gap: 15px; font-size: 13px;
  }
  .filename { font-weight: 600; color: #fff; }
  .meta { color: #888; font-family: monospace; }

  .bottom {
    bottom: 20px; width: 90%; max-width: 800px;
    border-radius: 12px; padding: 12px 20px;
    display: flex; flex-direction: column; gap: 8px;
  }

  /* --- CONTROLS --- */
  .scrubber-track { width: 100%; display: flex; align-items: center; }
  .scrubber {
    width: 100%; height: 4px; border-radius: 2px;
    accent-color: #3b82f6; cursor: pointer;
    transition: height 0.1s;
  }
  .scrubber:hover { height: 6px; }

  .controls-row {
    display: flex; justify-content: space-between; align-items: center;
  }

  .group { display: flex; align-items: center; gap: 10px; }

  .icon-btn {
    background: transparent; border: none; color: #ddd;
    cursor: pointer; padding: 6px; border-radius: 4px;
    display: flex; align-items: center; justify-content: center;
  }
  .icon-btn:hover { background: rgba(255,255,255,0.1); color: #fff; }
  
  .time-readout {
    font-family: 'Fira Code', monospace; font-size: 13px; color: #fff;
    margin-left: 10px;
  }
  .dim { color: #666; }

  .select-wrapper select {
    background: rgba(255,255,255,0.05); border: none;
    color: #ccc; font-size: 12px; padding: 4px 8px;
    border-radius: 4px; outline: none; cursor: pointer;
  }
  .select-wrapper select:hover { background: rgba(255,255,255,0.1); color: #fff; }

  .separator { width: 1px; height: 16px; background: rgba(255,255,255,0.2); margin: 0 5px; }
</style>