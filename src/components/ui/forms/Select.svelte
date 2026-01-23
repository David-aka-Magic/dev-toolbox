<script lang="ts">
  export let value: string;
  export let label: string = '';
  
  // Options can be passed as an array of objects
  export let options: { value: string; label: string }[] = [];
</script>

<div class="form-group">
  {#if label}
    <label>{label}</label>
  {/if}

  <select bind:value on:change>
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
</div>

<style>
  .form-group { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  label { font-size: 0.85rem; font-weight: 500; color: var(--text-main); }
  
  select {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 10px;
    border-radius: 4px;
    outline: none;
    cursor: pointer;
  }
  select:focus { border-color: var(--border-focus); }
</style>