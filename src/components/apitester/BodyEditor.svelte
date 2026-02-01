<script lang="ts">
  type BodyType = 'none' | 'json' | 'text' | 'form';

  let { 
    body = '', 
    bodyType = 'json' as BodyType,
    onupdate = (data: { body: string; bodyType: BodyType }) => {}
  }: {
    body: string;
    bodyType: BodyType;
    onupdate?: (data: { body: string; bodyType: BodyType }) => void;
  } = $props();

  const bodyTypes = [
    { value: 'none', label: 'None' },
    { value: 'json', label: 'JSON' },
    { value: 'text', label: 'Text' },
    { value: 'form', label: 'Form URL Encoded' },
  ];

  function updateBody(newBody: string) {
    onupdate({ body: newBody, bodyType });
  }

  function updateBodyType(newType: BodyType) {
    onupdate({ body, bodyType: newType });
  }

  function formatJson() {
    try {
      const parsed = JSON.parse(body);
      updateBody(JSON.stringify(parsed, null, 2));
    } catch {
      // Invalid JSON
    }
  }

  function minifyJson() {
    try {
      const parsed = JSON.parse(body);
      updateBody(JSON.stringify(parsed));
    } catch {
      // Invalid JSON
    }
  }

  let isValidJson = $derived((() => {
    if (bodyType !== 'json' || !body.trim()) return true;
    try {
      JSON.parse(body);
      return true;
    } catch {
      return false;
    }
  })());
</script>

<div class="body-editor">
  <div class="type-selector">
    {#each bodyTypes as type}
      <button 
        class="type-btn" 
        class:active={bodyType === type.value}
        onclick={() => updateBodyType(type.value as BodyType)}
      >
        {type.label}
      </button>
    {/each}

    {#if bodyType === 'json'}
      <div class="json-actions">
        <button class="action-btn" onclick={formatJson} title="Format JSON">
          Format
        </button>
        <button class="action-btn" onclick={minifyJson} title="Minify JSON">
          Minify
        </button>
      </div>
    {/if}
  </div>

  {#if bodyType === 'none'}
    <div class="no-body">
      <p>This request does not have a body</p>
    </div>
  {:else}
    <div class="editor-container">
      <textarea
        class="body-textarea"
        class:error={!isValidJson}
        value={body}
        oninput={(e) => updateBody((e.target as HTMLTextAreaElement).value)}
        placeholder={bodyType === 'json' ? '{\n  "key": "value"\n}' : 'Enter request body...'}
        spellcheck="false"
      ></textarea>
      {#if bodyType === 'json' && !isValidJson && body.trim()}
        <div class="error-message">Invalid JSON syntax</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .body-editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .type-selector {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .type-btn {
    padding: 6px 14px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .type-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .type-btn.active {
    background: var(--border-focus);
    border-color: var(--border-focus);
    color: white;
  }

  .json-actions {
    margin-left: auto;
    display: flex;
    gap: 4px;
  }

  .action-btn {
    padding: 4px 10px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
    border-color: var(--border-focus);
  }

  .no-body {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }

  .editor-container {
    position: relative;
  }

  .body-textarea {
    width: 100%;
    min-height: 200px;
    padding: 12px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-main);
    font-family: 'Fira Code', monospace;
    font-size: 12px;
    line-height: 1.5;
    resize: vertical;
    outline: none;
    transition: border-color 0.15s;
  }

  .body-textarea:focus {
    border-color: var(--border-focus);
  }

  .body-textarea.error {
    border-color: #ef4444;
  }

  .body-textarea::placeholder {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .error-message {
    margin-top: 8px;
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 4px;
    color: #ef4444;
    font-size: 12px;
  }
</style>