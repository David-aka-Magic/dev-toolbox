<script lang="ts">
  import { onMount } from 'svelte';
  
  // Props
  export let src: string = ''; // The image source
  export let filename: string = 'image.png';
  export let meta = { width: 0, height: 0, size: '0 MB', type: 'PNG' };

  // --- CANVAS STATE ---
  let container: HTMLDivElement;
  let scale = 1;
  let panning = false;
  let pointX = 0;
  let pointY = 0;
  let startX = 0;
  let startY = 0;

  // GIF State (Placeholder for now)
  let isPlaying = true;
  let currentFrame = 0;
  let totalFrames = 100;

  // --- LOGIC: PANNING ---
  function onMouseDown(e: MouseEvent) {
    e.preventDefault();
    startPanning(e.clientX, e.clientY);
  }

  function startPanning(clientX: number, clientY: number) {
    panning = true;
    startX = clientX - pointX;
    startY = clientY - pointY;
    container.style.cursor = 'grabbing';
  }

  function onMouseUp() {
    panning = false;
    container.style.cursor = 'grab';
  }

  function onMouseMove(e: MouseEvent) {
    e.preventDefault();
    if (!panning) return;
    pointX = e.clientX - startX;
    pointY = e.clientY - startY;
  }

  // --- LOGIC: ZOOMING (The Math Heavy Part) ---
  function onWheel(e: WheelEvent) {
    e.preventDefault();
    
    const xs = (e.clientX - pointX) / scale;
    const ys = (e.clientY - pointY) / scale;
    
    // Zoom factor: 10% per scroll tick
    const delta = -e.deltaY;
    const factor = delta > 0 ? 1.1 : 0.9;
    
    // Limit Zoom
    let newScale = scale * factor;
    if (newScale < 0.1) newScale = 0.1;
    if (newScale > 50) newScale = 50;

    pointX = e.clientX - xs * newScale;
    pointY = e.clientY - ys * newScale;
    scale = newScale;
  }

  // Reset View to Center
  function reset() {
    scale = 1;
    pointX = 0;
    pointY = 0;
    
    // Simple centering math
    if (container) {
       const rect = container.getBoundingClientRect();
       // This centers 0,0. You might want to offset based on image size later.
       pointX = (rect.width / 2) - (meta.width / 2); 
       pointY = (rect.height / 2) - (meta.height / 2);
    }
  }
  
  onMount(() => {
      // Auto-center on load
      reset();
  });
</script>

<svelte:window on:mouseup={onMouseUp} on:mousemove={onMouseMove} />

<div 
  class="viewer-container" 
  bind:this={container}
  on:wheel|nonpassive={onWheel}
  on:mousedown={onMouseDown}
>
  
  <div 
    class="canvas-layer"
    style:transform="translate3d({pointX}px, {pointY}px, 0) scale({scale})"
  >
    <img {src} alt={filename} class="media-content" draggable="false" />
  </div>

  <div class="hud-overlay top">
    <div class="meta-tag filename">{filename}</div>
    <div class="meta-group">
      <span class="meta-tag">{meta.width}x{meta.height}</span>
      <span class="meta-tag">{meta.size}</span>
      <span class="meta-tag badge">{meta.type}</span>
    </div>
  </div>

  <div class="hud-overlay bottom">
    
    {#if meta.type === 'GIF'}
      <div class="scrubber-row">
         <button class="icon-btn" on:click={() => isPlaying = !isPlaying}>
           {#if isPlaying}
             <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
           {:else}
             <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
           {/if}
         </button>
         <input type="range" min="0" max={totalFrames} bind:value={currentFrame} class="scrub-slider" />
         <span class="frame-counter">{currentFrame}/{totalFrames}</span>
      </div>
    {/if}

    <div class="controls-row">
      <div class="zoom-readout">{(scale * 100).toFixed(0)}%</div>
      
      <button class="control-btn" on:click={reset} title="Reset View (R)">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
      </button>

      <button class="control-btn" on:click={() => scale = scale * 1.1} title="Zoom In (+)">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
      </button>

      <button class="control-btn" on:click={() => scale = scale * 0.9} title="Zoom Out (-)">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
      </button>
    </div>
  </div>

</div>

<style>
  .viewer-container {
    width: 100%;
    height: 100%;
    background-image: 
      linear-gradient(45deg, #1a1a1a 25%, transparent 25%), 
      linear-gradient(-45deg, #1a1a1a 25%, transparent 25%), 
      linear-gradient(45deg, transparent 75%, #1a1a1a 75%), 
      linear-gradient(-45deg, transparent 75%, #1a1a1a 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    background-color: #111; /* Checkboard pattern for transparency */
    overflow: hidden;
    position: relative;
    cursor: grab;
    user-select: none;
  }

  .canvas-layer {
    position: absolute;
    top: 0; left: 0;
    transform-origin: 0 0;
    will-change: transform; /* Performance Hint */
  }

  .media-content {
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
    max-width: none; /* Allow it to be huge */
    display: block;
    pointer-events: none; /* Let clicks pass through to container for dragging */
  }

  /* --- HUD OVERLAYS --- */
  .hud-overlay {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(15, 15, 20, 0.85);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 30px; /* Pill shape */
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 15px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    color: white;
    font-family: 'Segoe UI', sans-serif;
    font-size: 13px;
    pointer-events: auto; /* Re-enable clicking on buttons */
  }

  .top { top: 20px; justify-content: space-between; min-width: 300px; }
  .bottom { bottom: 20px; flex-direction: column; gap: 8px; padding: 10px 20px; }

  .meta-tag { color: #aaa; font-weight: 500; }
  .filename { color: #fff; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px; }
  .badge { background: #3b82f6; color: white; padding: 2px 6px; border-radius: 4px; font-size: 11px; font-weight: 700; }

  /* --- CONTROLS --- */
  .controls-row { display: flex; align-items: center; gap: 10px; }
  .scrubber-row { display: flex; align-items: center; gap: 10px; width: 100%; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 8px; margin-bottom: 4px; }

  .control-btn, .icon-btn {
    background: transparent; border: none; color: #ccc;
    cursor: pointer; padding: 6px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    transition: all 0.2s;
  }
  .control-btn:hover { background: rgba(255,255,255,0.1); color: white; transform: scale(1.1); }
  
  .zoom-readout { font-variant-numeric: tabular-nums; width: 45px; text-align: right; color: #888; margin-right: 5px; }

  .scrub-slider {
    flex: 1; height: 4px; border-radius: 2px;
    accent-color: #3b82f6; cursor: pointer;
  }
</style>