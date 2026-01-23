<script lang="ts">
  export let value: number;
  export let min: number = 0;
  export let max: number = 100;
  export let step: number = 1;
  export let label: string = '';
  export let unit: string = '';
</script>

<div class="form-group">
  {#if label}
    <div class="header">
      <label>{label}</label>
      <span class="value">{value}{unit}</span>
    </div>
  {/if}

  <input type="range" {min} {max} {step} bind:value />
</div>

<style>
  .form-group { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  
  .header { display: flex; justify-content: space-between; align-items: center; }
  label { font-size: 0.85rem; font-weight: 500; color: var(--text-main); }
  .value { font-size: 0.85rem; color: var(--text-muted); font-family: monospace; }
  
  input[type=range] {
    width: 100%; cursor: pointer; accent-color: var(--border-focus);
  }
</style>