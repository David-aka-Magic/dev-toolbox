<script lang="ts">
  export let checked: boolean = false;
  export let label: string = '';
</script>

<label class="checkbox-wrapper">
  <input type="checkbox" bind:checked />
  <span class="custom-check">
    {#if checked}
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
        <polyline points="20 6 9 17 4 12"></polyline>
      </svg>
    {/if}
  </span>
  <span class="label-text">{label}</span>
</label>

<style>
  .checkbox-wrapper { display: flex; align-items: center; gap: 10px; cursor: pointer; user-select: none; }
  
  /* Hide default checkbox but keep it accessible */
  input { position: absolute; opacity: 0; cursor: pointer; height: 0; width: 0; }

  .custom-check {
    height: 18px; width: 18px;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 4px;
    display: flex; align-items: center; justify-content: center;
    transition: all 0.2s;
    color: white;
  }

  /* Checked State */
  input:checked ~ .custom-check {
    background: var(--border-focus);
    border-color: var(--border-focus);
  }

  svg { width: 12px; height: 12px; }
  .label-text { font-size: 0.9rem; color: var(--text-main); }
</style>