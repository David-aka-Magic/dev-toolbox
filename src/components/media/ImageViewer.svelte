<script lang="ts">
  import { onMount, tick } from 'svelte';
  
  // Props
  export let src: string = '';
  export let filename: string = 'image.png';
  export let meta = { width: 0, height: 0, size: '0 MB', type: 'PNG' };

  // --- STATE ---
  let container: HTMLDivElement;
  let imgElement: HTMLImageElement;
  let canvas: HTMLCanvasElement; // Hidden canvas for reading pixels
  let freezeCanvas: HTMLCanvasElement; // Visible canvas for "Pausing" GIFs

  let scale = 1;
  let panning = false;
  let pointX = 0;
  let pointY = 0;
  let startX = 0;
  let startY = 0;

  // GIF State
  let isPlaying = true;
  // Note: HTML img tags cannot be scrubbed without an external parser library.
  // We will implement Play/Pause by freezing the frame on a canvas.

  // --- INSPECTOR STATE ---
  let showInspector = false;
  let hoveredPixel = { x: 0, y: 0, r: 0, g: 0, b: 0, a: 0, hex: '#000000' };

  // --- LOGIC: PANNING ---
  function onMouseDown(e: MouseEvent) {
    if (showInspector) return; 
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
    container.style.cursor = showInspector ? 'crosshair' : 'grab';
  }

  function onMouseMove(e: MouseEvent) {
    e.preventDefault();
    
    if (panning) {
      pointX = e.clientX - startX;
      pointY = e.clientY - startY;
      return;
    }

    if (showInspector) {
      updateInspector(e.clientX, e.clientY);
    }
  }

  // --- LOGIC: PIXEL INSPECTION ---
  async function toggleInspector() {
    showInspector = !showInspector;
    if (showInspector) {
      await tick();
      drawToHiddenCanvas(); // Ensure canvas is ready when opening
    }
  }

  function drawToHiddenCanvas() {
    if (!imgElement || !canvas) return;
    // Set canvas to match natural image size
    canvas.width = imgElement.naturalWidth;
    canvas.height = imgElement.naturalHeight;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      // Clear before drawing
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(imgElement, 0, 0);
    }
  }

  function updateInspector(mouseX: number, mouseY: number) {
    if (!container || !imgElement) return;

    const rect = container.getBoundingClientRect();
    // Reverse the transform math: (ScreenPos - ContainerOffset - PanOffset) / Scale
    const relX = (mouseX - rect.left - pointX) / scale;
    const relY = (mouseY - rect.top - pointY) / scale;

    const x = Math.floor(relX);
    const y = Math.floor(relY);

    // Bounds check
    if (x >= 0 && x < meta.width && y >= 0 && y < meta.height) {
      const ctx = canvas?.getContext('2d');
      if (ctx) {
        try {
          const p = ctx.getImageData(x, y, 1, 1).data;
          const hex = "#" + ((1 << 24) + (p[0] << 16) + (p[1] << 8) + p[2]).toString(16).slice(1).toUpperCase();
          hoveredPixel = { x, y, r: p[0], g: p[1], b: p[2], a: p[3], hex };
        } catch (e) {
          // Canvas Taint error usually happens if CORS isn't set
          console.warn("Cannot read pixel data - Canvas Tainted", e);
        }
      }
    }
  }

  // --- LOGIC: GIF PLAYBACK (Freeze Frame Method) ---
  function toggleGifPlay() {
    isPlaying = !isPlaying;
    
    if (!isPlaying) {
      // PAUSE: Draw current img state to the freeze canvas
      if (freezeCanvas && imgElement) {
        freezeCanvas.width = imgElement.naturalWidth;
        freezeCanvas.height = imgElement.naturalHeight;
        const ctx = freezeCanvas.getContext('2d');
        if (ctx) ctx.drawImage(imgElement, 0, 0);
      }
    } 
    // PLAY: We just hide the canvas and show the img tag again
  }

  // --- LOGIC: ZOOMING ---
  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const rect = container.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const xs = (mouseX - pointX) / scale;
    const ys = (mouseY - pointY) / scale;
    
    const delta = -e.deltaY;
    const factor = delta > 0 ? 1.1 : 0.9;
    
    let newScale = scale * factor;
    if (newScale < 0.1) newScale = 0.1;
    if (newScale > 100) newScale = 100;

    pointX = mouseX - xs * newScale;
    pointY = mouseY - ys * newScale;
    scale = newScale;
  }

  function reset() {
    if (container && imgElement) {
      const containerRect = container.getBoundingClientRect();
      const imgWidth = imgElement.naturalWidth;
      const imgHeight = imgElement.naturalHeight;
      const scaleX = containerRect.width / imgWidth;
      const scaleY = containerRect.height / imgHeight;
      scale = Math.min(scaleX, scaleY, 1) * 0.9;
      
      const scaledWidth = imgWidth * scale;
      const scaledHeight = imgHeight * scale;
      pointX = (containerRect.width - scaledWidth) / 2;
      pointY = (containerRect.height - scaledHeight) / 2;
    } else {
      scale = 1; pointX = 0; pointY = 0;
    }
  }

  function handleImageLoad() {
    if (imgElement) {
      meta = { ...meta, width: imgElement.naturalWidth, height: imgElement.naturalHeight };
      drawToHiddenCanvas();
    }
    reset();
  }
  
  onMount(() => {
    if (imgElement && imgElement.complete) handleImageLoad();
  });
</script>

<svelte:window on:mouseup={onMouseUp} on:mousemove={onMouseMove} />

<div 
  class="viewer-container" 
  bind:this={container}
  on:wheel|nonpassive={onWheel}
  on:mousedown={onMouseDown}
>
  <canvas bind:this={canvas} style="display: none;"></canvas>

  <div 
    class="canvas-layer"
    style:transform="translate3d({pointX}px, {pointY}px, 0) scale({scale})"
  >
    <img 
      bind:this={imgElement}
      {src} 
      alt={filename} 
      class="media-content" 
      class:pixelated={scale > 4}
      draggable="false"
      crossorigin="anonymous"
      style:display={!isPlaying && meta.type === 'GIF' ? 'none' : 'block'}
      on:load={handleImageLoad}
    />

    <canvas 
      bind:this={freezeCanvas}
      class="media-content"
      class:pixelated={scale > 4}
      style:display={!isPlaying && meta.type === 'GIF' ? 'block' : 'none'}
    ></canvas>
    
    {#if scale > 8}
      <div 
        class="pixel-grid"
        style="
          width: {meta.width}px; 
          height: {meta.height}px;
          background-size: 1px 1px;
        "
      ></div>
    {/if}
    
    {#if showInspector}
      <div 
        class="pixel-highlight"
        style="
          left: {hoveredPixel.x}px; 
          top: {hoveredPixel.y}px;
          width: 1px; height: 1px;
        "
      ></div>
    {/if}
  </div>

  {#if showInspector}
    <div class="inspector-hud" style="border-color: {hoveredPixel.hex}">
      <div class="color-preview" style="background: {hoveredPixel.hex}"></div>
      <div class="info-col">
        <div class="hex-text">{hoveredPixel.hex}</div>
        <div class="rgb-text">
          X:{hoveredPixel.x} Y:{hoveredPixel.y}
          <span class="opacity">({hoveredPixel.r},{hoveredPixel.g},{hoveredPixel.b})</span>
        </div>
      </div>
    </div>
  {/if}

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
         <button class="icon-btn" on:click={toggleGifPlay}>
           {#if isPlaying}
             <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
           {:else}
             <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
           {/if}
         </button>
         <span class="meta-tag" style="margin-left: auto;">{isPlaying ? 'Playing' : 'Paused'}</span>
      </div>
    {/if}

    <div class="controls-row">
      <div class="zoom-readout">{(scale * 100).toFixed(0)}%</div>
      
      <button 
        class="control-btn" 
        class:active={showInspector}
        on:click={toggleInspector} 
        title="Inspect Pixels (I)"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 21.7C17.3 17 20 13 20 10a8 8 0 1 0-16 0c0 3 2.7 7 8 11.7z"/><circle cx="12" cy="10" r="3"/><path d="M12 2v2"/><path d="M12 18v2"/><path d="M4.93 4.93l1.41 1.41"/><path d="M17.66 17.66l1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="M4.93 19.07l1.41-1.41"/><path d="M17.66 6.34l1.41-1.41"/></svg>
      </button>

      <div class="separator"></div>

      <button class="control-btn" on:click={reset} title="Reset">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
      </button>

      <button class="control-btn" on:click={() => scale = scale * 1.5} title="Zoom In">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
      </button>

      <button class="control-btn" on:click={() => scale = scale * 0.75} title="Zoom Out">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
      </button>
    </div>
  </div>
</div>

<style>
  /* Use the exact same styles from the previous file */
  .viewer-container {
    width: 100%; height: 100%;
    background-image: 
      linear-gradient(45deg, #1a1a1a 25%, transparent 25%), 
      linear-gradient(-45deg, #1a1a1a 25%, transparent 25%), 
      linear-gradient(45deg, transparent 75%, #1a1a1a 75%), 
      linear-gradient(-45deg, transparent 75%, #1a1a1a 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    background-color: #111;
    overflow: hidden; position: relative;
    cursor: grab; user-select: none;
  }
  .canvas-layer {
    position: absolute; top: 0; left: 0;
    transform-origin: 0 0; will-change: transform;
  }
  .media-content {
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
    display: block; pointer-events: none;
    image-rendering: auto; 
  }
  .media-content.pixelated {
    image-rendering: pixelated; 
    image-rendering: crisp-edges;
  }
  .pixel-grid {
    position: absolute; top: 0; left: 0; pointer-events: none;
    background-image: 
      linear-gradient(to right, rgba(255,255,255,0.1) 1px, transparent 1px),
      linear-gradient(to bottom, rgba(255,255,255,0.1) 1px, transparent 1px);
    z-index: 10;
  }
  .pixel-highlight {
    position: absolute; pointer-events: none; z-index: 20;
    border: 1px solid rgba(255,255,255,0.8);
    box-shadow: 0 0 2px rgba(0,0,0,0.8);
    background: rgba(255,255,255,0.2);
  }
  .hud-overlay {
    position: absolute; left: 50%; transform: translateX(-50%);
    background: rgba(15, 15, 20, 0.85); backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1); border-radius: 30px;
    padding: 8px 16px; display: flex; align-items: center; gap: 15px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3); color: white;
    font-family: 'Segoe UI', sans-serif; font-size: 13px;
    pointer-events: auto; z-index: 100;
  }
  .top { top: 20px; justify-content: space-between; min-width: 300px; }
  .bottom { bottom: 20px; flex-direction: column; gap: 8px; padding: 10px 20px; }
  .inspector-hud {
    position: absolute; top: 80px; left: 50%; transform: translateX(-50%);
    background: rgba(0,0,0,0.9); border: 1px solid #444; border-left-width: 4px;
    padding: 8px 12px; border-radius: 6px; pointer-events: none;
    display: flex; gap: 10px; align-items: center; z-index: 200;
    font-family: 'Fira Code', monospace;
  }
  .color-preview { width: 24px; height: 24px; border-radius: 4px; border: 1px solid #555; }
  .info-col { display: flex; flex-direction: column; line-height: 1.2; }
  .hex-text { font-weight: 700; color: #fff; font-size: 14px; }
  .rgb-text { color: #888; font-size: 11px; }
  .meta-tag { color: #aaa; font-weight: 500; }
  .filename { color: #fff; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px; }
  .badge { background: #3b82f6; color: white; padding: 2px 6px; border-radius: 4px; font-size: 11px; font-weight: 700; }
  .controls-row { display: flex; align-items: center; gap: 10px; }
  .scrubber-row { display: flex; align-items: center; gap: 10px; width: 100%; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 8px; margin-bottom: 4px; }
  .control-btn, .icon-btn {
    background: transparent; border: none; color: #ccc;
    cursor: pointer; padding: 6px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    transition: all 0.2s;
  }
  .control-btn:hover { background: rgba(255,255,255,0.1); color: white; transform: scale(1.1); }
  .control-btn.active { color: #3b82f6; background: rgba(59, 130, 246, 0.15); }
  .zoom-readout { font-variant-numeric: tabular-nums; width: 45px; text-align: right; color: #888; margin-right: 5px; }
  .separator { width: 1px; height: 16px; background: rgba(255,255,255,0.2); margin: 0 5px; }
</style>