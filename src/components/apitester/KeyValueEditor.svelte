<script lang="ts">
  interface KeyValueItem {
    key: string;
    value: string;
    enabled: boolean;
  }

  let { 
    items = [], 
    placeholder = { key: 'Key', value: 'Value' },
    onupdate = (items: KeyValueItem[]) => {}
  }: {
    items: KeyValueItem[];
    placeholder?: { key: string; value: string };
    onupdate?: (items: KeyValueItem[]) => void;
  } = $props();

  function addRow() {
    const newItems = [...items, { key: '', value: '', enabled: true }];
    onupdate(newItems);
  }

  function removeRow(index: number) {
    const newItems = items.filter((_, i) => i !== index);
    onupdate(newItems);
  }

  function updateRow(index: number, field: 'key' | 'value' | 'enabled', value: string | boolean) {
    const newItems = items.map((item, i) => 
      i === index ? { ...item, [field]: value } : item
    );
    onupdate(newItems);
  }

  function handleKeyInput(e: Event, index: number) {
    const value = (e.target as HTMLInputElement).value;
    updateRow(index, 'key', value);
    
    if (index === items.length - 1 && value) {
      addRow();
    }
  }
</script>

<div class="kv-editor">
  <div class="header-row">
    <div class="col-checkbox"></div>
    <div class="col-key">KEY</div>
    <div class="col-value">VALUE</div>
    <div class="col-actions"></div>
  </div>

  {#each items as item, index}
    <div class="item-row" class:disabled={!item.enabled}>
      <div class="col-checkbox">
        <input 
          type="checkbox" 
          checked={item.enabled}
          onchange={(e) => updateRow(index, 'enabled', (e.target as HTMLInputElement).checked)}
        />
      </div>
      <div class="col-key">
        <input 
          type="text" 
          value={item.key}
          placeholder={placeholder.key}
          oninput={(e) => handleKeyInput(e, index)}
        />
      </div>
      <div class="col-value">
        <input 
          type="text" 
          value={item.value}
          placeholder={placeholder.value}
          oninput={(e) => updateRow(index, 'value', (e.target as HTMLInputElement).value)}
        />
      </div>
      <div class="col-actions">
        {#if items.length > 1 || item.key || item.value}
          <button class="delete-btn" onclick={() => removeRow(index)} title="Remove">
            ×
          </button>
        {/if}
      </div>
    </div>
  {/each}

  {#if items.length === 0}
    <button class="add-btn" onclick={addRow}>
      + Add
    </button>
  {/if}
</div>

<style>
  .kv-editor {
    width: 100%;
  }

  .header-row {
    display: flex;
    align-items: center;
    padding: 8px 4px;
    border-bottom: 1px solid var(--border);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
  }

  .item-row {
    display: flex;
    align-items: center;
    padding: 4px;
    border-bottom: 1px solid var(--border);
    transition: opacity 0.15s;
  }

  .item-row.disabled {
    opacity: 0.5;
  }

  .item-row:hover {
    background: var(--hover-bg);
  }

  .col-checkbox {
    width: 32px;
    display: flex;
    justify-content: center;
  }

  .col-checkbox input[type="checkbox"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: var(--border-focus);
  }

  .col-key {
    flex: 1;
    padding-right: 8px;
  }

  .col-value {
    flex: 2;
    padding-right: 8px;
  }

  .col-actions {
    width: 32px;
    display: flex;
    justify-content: center;
  }

  .item-row input[type="text"] {
    width: 100%;
    padding: 6px 8px;
    background: var(--bg-input);
    border: 1px solid transparent;
    border-radius: 3px;
    color: var(--text-main);
    font-size: 12px;
    font-family: 'Fira Code', monospace;
    outline: none;
    transition: border-color 0.15s;
  }

  .item-row input[type="text"]:focus {
    border-color: var(--border-focus);
  }

  .item-row input[type="text"]::placeholder {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .delete-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: all 0.15s;
  }

  .item-row:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .add-btn {
    padding: 8px 16px;
    margin-top: 8px;
    background: transparent;
    border: 1px dashed var(--border);
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .add-btn:hover {
    border-color: var(--border-focus);
    color: var(--text-main);
  }
</style>