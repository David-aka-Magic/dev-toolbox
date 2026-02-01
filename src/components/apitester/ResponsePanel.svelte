<script lang="ts">
  import { apiStore } from '$lib/stores/apiStore';

  let activeTab: 'body' | 'headers' | 'cookies' = $state('body');
  let bodyView: 'pretty' | 'raw' = $state('pretty');

  let response = $derived($apiStore.response);

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function getStatusColor(status: number): string {
    if (status >= 200 && status < 300) return '#22c55e';
    if (status >= 300 && status < 400) return '#3b82f6';
    if (status >= 400 && status < 500) return '#eab308';
    if (status >= 500) return '#ef4444';
    return '#6b7280';
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  function downloadResponse() {
    if (!response?.body) return;
    const blob = new Blob([response.body], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'response.json';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="response-panel">
  {#if !response}
    <div class="empty-state">
      <div class="empty-icon">📡</div>
      <h3>No Response Yet</h3>
      <p>Send a request to see the response here</p>
    </div>
  {:else}
    <div class="response-header">
      <div class="status-info">
        <span 
          class="status-badge" 
          class:error={response.error}
          style="background: {response.error ? '#ef4444' : getStatusColor(response.status)}"
        >
          {response.status} {response.statusText}
        </span>
        <span class="meta-item">
          <span class="meta-label">Time:</span>
          <span class="meta-value">{response.time} ms</span>
        </span>
        <span class="meta-item">
          <span class="meta-label">Size:</span>
          <span class="meta-value">{formatSize(response.size)}</span>
        </span>
      </div>
      <div class="actions">
        <button class="action-btn" onclick={() => copyToClipboard(response.body)} title="Copy">
          📋
        </button>
        <button class="action-btn" onclick={downloadResponse} title="Download">
          ⬇️
        </button>
      </div>
    </div>

    <div class="tabs">
      <button 
        class="tab" 
        class:active={activeTab === 'body'}
        onclick={() => activeTab = 'body'}
      >
        Body
      </button>
      <button 
        class="tab" 
        class:active={activeTab === 'headers'}
        onclick={() => activeTab = 'headers'}
      >
        Headers
        <span class="badge">{Object.keys(response.headers).length}</span>
      </button>
      <button 
        class="tab" 
        class:active={activeTab === 'cookies'}
        onclick={() => activeTab = 'cookies'}
      >
        Cookies
      </button>

      {#if activeTab === 'body'}
        <div class="tab-spacer"></div>
        <div class="view-toggle">
          <button 
            class="toggle-btn" 
            class:active={bodyView === 'pretty'}
            onclick={() => bodyView = 'pretty'}
          >
            Pretty
          </button>
          <button 
            class="toggle-btn" 
            class:active={bodyView === 'raw'}
            onclick={() => bodyView = 'raw'}
          >
            Raw
          </button>
        </div>
      {/if}
    </div>

    <div class="tab-content">
      {#if activeTab === 'body'}
        <div class="body-content" class:raw={bodyView === 'raw'}>
          <pre><code>{response.body}</code></pre>
        </div>
      {:else if activeTab === 'headers'}
        <div class="headers-list">
          {#each Object.entries(response.headers) as [key, value]}
            <div class="header-row">
              <span class="header-key">{key}</span>
              <span class="header-value">{value}</span>
            </div>
          {/each}
        </div>
      {:else if activeTab === 'cookies'}
        <div class="cookies-content">
          {#if response.headers['set-cookie']}
            <pre><code>{response.headers['set-cookie']}</code></pre>
          {:else}
            <p class="no-content">No cookies in response</p>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .response-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-main);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.5;
  }

  .empty-state h3 {
    margin: 0 0 8px 0;
    font-size: 16px;
    font-weight: 500;
    color: var(--text-main);
  }

  .empty-state p {
    margin: 0;
    font-size: 13px;
  }

  .response-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
  }

  .status-info {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .status-badge {
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    color: white;
  }

  .status-badge.error {
    background: #ef4444 !important;
  }

  .meta-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
  }

  .meta-label {
    color: var(--text-muted);
  }

  .meta-value {
    color: var(--text-main);
    font-family: 'Fira Code', monospace;
  }

  .actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: var(--hover-bg);
    border-color: var(--border-focus);
  }

  .tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-panel);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-muted);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .tab:hover {
    color: var(--text-main);
  }

  .tab.active {
    color: var(--text-main);
    border-bottom-color: var(--border-focus);
  }

  .badge {
    padding: 2px 6px;
    background: var(--bg-input);
    border-radius: 10px;
    font-size: 10px;
    color: var(--text-muted);
  }

  .tab-spacer {
    flex: 1;
  }

  .view-toggle {
    display: flex;
    background: var(--bg-input);
    border-radius: 4px;
    padding: 2px;
  }

  .toggle-btn {
    padding: 4px 12px;
    background: transparent;
    border: none;
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .toggle-btn.active {
    background: var(--bg-panel);
    color: var(--text-main);
  }

  .tab-content {
    flex: 1;
    overflow: auto;
  }

  .body-content {
    padding: 12px;
  }

  .body-content pre {
    margin: 0;
    font-family: 'Fira Code', monospace;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .body-content.raw pre {
    white-space: pre;
    word-break: normal;
  }

  .body-content code {
    color: var(--text-main);
  }

  .headers-list {
    padding: 12px;
  }

  .header-row {
    display: flex;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .header-row:last-child {
    border-bottom: none;
  }

  .header-key {
    flex: 0 0 200px;
    font-weight: 500;
    color: var(--border-focus);
    font-family: 'Fira Code', monospace;
  }

  .header-value {
    flex: 1;
    color: var(--text-main);
    font-family: 'Fira Code', monospace;
    word-break: break-all;
  }

  .cookies-content {
    padding: 12px;
  }

  .cookies-content pre {
    margin: 0;
    font-family: 'Fira Code', monospace;
    font-size: 12px;
    line-height: 1.5;
  }

  .no-content {
    color: var(--text-muted);
    font-size: 13px;
    text-align: center;
    padding: 24px;
  }
</style>