<script lang="ts">
  export let value: string | number;
  export let label: string = '';
  export let type: 'text' | 'number' | 'password' = 'text';
  export let placeholder: string = '';
  export let hint: string = '';
</script>

<div class="form-group">
  {#if label}
    <label for="input-{label}">{label}</label>
  {/if}
  
  <input 
    id="input-{label}"
    {type} 
    bind:value 
    {placeholder}
    {...$$restProps} 
  />

  {#if hint}
    <span class="hint">{hint}</span>
  {/if}
</div>

<style>
  .form-group { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  
  label { font-size: 0.85rem; font-weight: 500; color: var(--text-main); }
  
  input {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 10px;
    border-radius: 4px;
    outline: none;
    font-family: inherit;
    font-size: 0.9rem;
    transition: border-color 0.2s;
  }
  
  input:focus { border-color: var(--border-focus); }
  
  .hint { font-size: 0.75rem; color: var(--text-muted); }
</style>