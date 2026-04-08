<script lang="ts">
  import type { GanttTask } from '$lib/stores/ganttStore';

  let {
    task,
    x,
    y,
    width,
    height,
    isSelected    = false,
    isDragging    = false,
    isSummary     = false,
    isCritical    = false,
    showProgress  = true,
    onbardown     = (_e: MouseEvent, _mode: 'move'|'resize-left'|'resize-right') => {},
    ondblclick    = (_e: MouseEvent) => {},
    oncontextmenu = (_e: MouseEvent) => {},
  }: {
    task:           GanttTask;
    x:              number;
    y:              number;
    width:          number;
    height:         number;
    isSelected?:    boolean;
    isDragging?:    boolean;
    isSummary?:     boolean;
    isCritical?:    boolean;
    showProgress?:  boolean;
    onbardown?:     (e: MouseEvent, mode: 'move'|'resize-left'|'resize-right') => void;
    ondblclick?:    (e: MouseEvent) => void;
    oncontextmenu?: (e: MouseEvent) => void;
  } = $props();

  function handleMouseDown(e: MouseEvent) {
    if (isSummary) return;
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const relX = e.clientX - rect.left;
    const EDGE = 8;

    let mode: 'move' | 'resize-left' | 'resize-right';
    if (relX < EDGE)                mode = 'resize-left';
    else if (relX > rect.width - EDGE) mode = 'resize-right';
    else                            mode = 'move';

    onbardown(e, mode);
  }

  function handleDblClick(e: MouseEvent) {
    e.stopPropagation();
    ondblclick(e);
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    oncontextmenu(e);
  }

  // Dynamic cursor based on hover position
  let cursorStyle = $state('grab');

  function updateCursor(e: MouseEvent) {
    if (isSummary) { cursorStyle = 'default'; return; }
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const relX = e.clientX - rect.left;
    const EDGE = 8;
    if (relX < EDGE || relX > rect.width - EDGE) cursorStyle = 'ew-resize';
    else cursorStyle = 'grab';
  }
</script>

{#if isSummary}
  <!-- Summary bar: thin bar spanning children range, with triangle caps -->
  <div
    class="bar bar-summary"
    style:left="{x}px"
    style:top="{y}px"
    style:width="{width}px"
    style:background={task.color}
    style:opacity={isDragging ? 0.35 : 1}
    oncontextmenu={handleContextMenu}
    role="presentation"
  >
    <div class="summary-cap summary-cap-l" style:border-top-color={task.color}></div>
    <div class="summary-cap summary-cap-r" style:border-top-color={task.color}></div>
  </div>
{:else}
  <!-- Normal / sub-task bar -->
  <div
    class="bar"
    class:bar-selected={isSelected}
    class:bar-dragging={isDragging}
    class:bar-critical={isCritical}
    style:left="{x}px"
    style:top="{y}px"
    style:width="{width}px"
    style:height="{height}px"
    style:background={task.color}
    style:cursor={cursorStyle}
    title={task.title}
    onmousedown={handleMouseDown}
    ondblclick={handleDblClick}
    oncontextmenu={handleContextMenu}
    onmousemove={updateCursor}
    role="button"
    tabindex="0"
    aria-label={task.title}
    onkeydown={(e) => e.key === 'Enter' && ondblclick(e as unknown as MouseEvent)}
  >
    <!-- Critical path indicator -->
    {#if isCritical}
      <div class="cp-indicator"></div>
    {/if}

    <!-- Progress fill (darker overlay) -->
    {#if showProgress && task.progress > 0}
      <div class="bar-progress" style:width="{task.progress}%"></div>
    {/if}

    <!-- Resize handles (invisible, just for cursor hint) -->
    <div class="handle handle-l"></div>
    <div class="handle handle-r"></div>

    <!-- Label -->
    {#if width > 32}
      <span class="bar-label">{task.title}</span>
    {/if}

    <!-- CP badge -->
    {#if isCritical && width > 48}
      <span class="cp-badge">CP</span>
    {/if}
  </div>
{/if}

<style>
  .bar {
    position: absolute;
    border-radius: 4px;
    overflow: visible;
    display: flex;
    align-items: center;
    transition: filter 0.1s, opacity 0.1s;
    user-select: none;
  }
  .bar:hover {
    filter: brightness(1.15);
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
    transform: translateY(-1px);
    z-index: 10;
  }
  .bar.bar-selected {
    outline: 2px solid #fff;
    outline-offset: 1px;
    box-shadow: 0 0 0 3px var(--border-focus);
  }
  .bar.bar-dragging { opacity: 0.35; }

  /* Summary bar */
  .bar-summary {
    position: absolute;
    height: 8px;
    border-radius: 2px;
    overflow: visible;
  }

  /* Triangle caps on summary bar */
  .summary-cap {
    position: absolute;
    bottom: -7px;
    width: 0;
    height: 0;
    border-left: 7px solid transparent;
    border-right: 7px solid transparent;
    border-top-width: 8px;
    border-top-style: solid;
    /* border-top-color set via style prop */
  }
  .summary-cap-l { left: -1px; }
  .summary-cap-r { right: -7px; }

  /* Progress fill */
  .bar-progress {
    position: absolute;
    left: 0; top: 0; bottom: 0;
    background: rgba(0, 0, 0, 0.22);
    border-radius: 4px 0 0 4px;
    pointer-events: none;
  }

  /* Invisible edge handles for cursor */
  .handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 8px;
    pointer-events: none; /* handled via mousemove + mousedown on parent */
  }
  .handle-l { left: 0; }
  .handle-r { right: 0; }

  /* Critical path */
  .bar-critical {
    box-shadow: inset 3px 0 0 #ef4444;
  }
  .cp-indicator {
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 3px;
    background: #ef4444;
    border-radius: 4px 0 0 4px;
    pointer-events: none;
  }
  .cp-badge {
    position: absolute;
    right: 5px;
    top: 50%;
    transform: translateY(-50%);
    font-size: 0.55rem;
    font-weight: 800;
    letter-spacing: 0.05em;
    color: #ef4444;
    background: rgba(0,0,0,0.35);
    border-radius: 2px;
    padding: 1px 3px;
    pointer-events: none;
    z-index: 2;
  }

  /* Label */
  .bar-label {
    position: relative;
    z-index: 1;
    font-size: 0.68rem;
    font-weight: 600;
    color: #fff;
    padding: 0 7px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    max-width: 100%;
  }
</style>
