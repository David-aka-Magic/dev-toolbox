<script lang="ts">
  import { apiStore } from '$lib/stores/apiStore';
  import { settings } from '$lib/stores/settingsStore';
  import KeyValueEditor from './KeyValueEditor.svelte';
  import BodyEditor from './BodyEditor.svelte';

  const HTTP_METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];

  let activeTab: 'params' | 'headers' | 'body' | 'auth' = $state('params');
  let isLoading = $state(false);

  let activeRequest = $derived($apiStore.activeRequest);

  function updateMethod(e: Event) {
    const method = (e.target as HTMLSelectElement).value;
    apiStore.updateActiveRequest({ method });
  }

  function updateUrl(e: Event) {
    const url = (e.target as HTMLInputElement).value;
    apiStore.updateActiveRequest({ url });
  }

  async function sendRequest() {
    if (!activeRequest?.url) return;
    
    isLoading = true;
    
    try {
      const startTime = performance.now();
      
      const headers: Record<string, string> = {};
      activeRequest.headers
        .filter(h => h.enabled && h.key)
        .forEach(h => { headers[h.key] = h.value; });

      const params = activeRequest.params
        .filter(p => p.enabled && p.key)
        .map(p => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`)
        .join('&');

      let url = activeRequest.url;
      if (params) {
        url += (url.includes('?') ? '&' : '?') + params;
      }

      const fetchOptions: RequestInit = {
        method: activeRequest.method,
        headers,
        redirect: $settings.apiFollowRedirects ? 'follow' : 'manual',
      };

      if (['POST', 'PUT', 'PATCH'].includes(activeRequest.method) && activeRequest.body) {
        fetchOptions.body = activeRequest.body;
        if (!headers['Content-Type']) {
          headers['Content-Type'] = activeRequest.bodyType === 'json' 
            ? 'application/json' 
            : activeRequest.bodyType === 'form'
            ? 'application/x-www-form-urlencoded'
            : 'text/plain';
        }
      }

      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), $settings.apiDefaultTimeout);
      fetchOptions.signal = controller.signal;

      const response = await fetch(url, fetchOptions);
      clearTimeout(timeoutId);
      
      const endTime = performance.now();
      
      const responseHeaders: Record<string, string> = {};
      response.headers.forEach((value, key) => {
        responseHeaders[key] = value;
      });

      const contentType = response.headers.get('content-type') || '';
      let body: string;
      
      if (contentType.includes('application/json')) {
        const json = await response.json();
        body = $settings.apiAutoFormatJson 
          ? JSON.stringify(json, null, 2)
          : JSON.stringify(json);
      } else {
        body = await response.text();
      }

      apiStore.setResponse({
        status: response.status,
        statusText: response.statusText,
        headers: responseHeaders,
        body,
        time: Math.round(endTime - startTime),
        size: new Blob([body]).size,
      }, $settings.apiSaveToHistory);

    } catch (err) {
      let errorMessage = 'Request failed';
      if (err instanceof Error) {
        if (err.name === 'AbortError') {
          errorMessage = `Request timed out after ${$settings.apiDefaultTimeout}ms`;
        } else {
          errorMessage = err.message;
        }
      }
      
      apiStore.setResponse({
        status: 0,
        statusText: 'Error',
        headers: {},
        body: errorMessage,
        time: 0,
        size: 0,
        error: true,
      }, false);
    } finally {
      isLoading = false;
    }
  }

  function getMethodColor(method: string): string {
    const colors: Record<string, string> = {
      GET: '#22c55e',
      POST: '#eab308',
      PUT: '#3b82f6',
      PATCH: '#a855f7',
      DELETE: '#ef4444',
      HEAD: '#06b6d4',
      OPTIONS: '#6b7280',
    };
    return colors[method] || '#6b7280';
  }

  function handleParamsUpdate(items: any[]) {
    apiStore.updateActiveRequest({ params: items });
  }

  function handleHeadersUpdate(items: any[]) {
    apiStore.updateActiveRequest({ headers: items });
  }

  function handleBodyUpdate(data: { body: string; bodyType: any }) {
    apiStore.updateActiveRequest({ body: data.body, bodyType: data.bodyType });
  }
</script>

<div class="request-panel">
  <div class="url-bar">
    <select 
      class="method-select" 
      value={activeRequest?.method || 'GET'}
      onchange={updateMethod}
      style="color: {getMethodColor(activeRequest?.method || 'GET')}"
    >
      {#each HTTP_METHODS as method}
        <option value={method} style="color: {getMethodColor(method)}">{method}</option>
      {/each}
    </select>

    <input 
      type="text" 
      class="url-input"
      placeholder="Enter request URL"
      value={activeRequest?.url || ''}
      oninput={updateUrl}
      onkeydown={(e) => e.key === 'Enter' && sendRequest()}
    />

    <button 
      class="send-btn" 
      onclick={sendRequest}
      disabled={isLoading || !activeRequest?.url}
    >
      {#if isLoading}
        <span class="spinner"></span>
      {:else}
        Send
      {/if}
    </button>
  </div>

  <div class="tabs">
    <button 
      class="tab" 
      class:active={activeTab === 'params'}
      onclick={() => activeTab = 'params'}
    >
      Params
      {#if activeRequest?.params?.filter(p => p.enabled && p.key).length}
        <span class="badge">{activeRequest.params.filter(p => p.enabled && p.key).length}</span>
      {/if}
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'headers'}
      onclick={() => activeTab = 'headers'}
    >
      Headers
      {#if activeRequest?.headers?.filter(h => h.enabled && h.key).length}
        <span class="badge">{activeRequest.headers.filter(h => h.enabled && h.key).length}</span>
      {/if}
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'body'}
      onclick={() => activeTab = 'body'}
    >
      Body
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'auth'}
      onclick={() => activeTab = 'auth'}
    >
      Auth
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === 'params'}
      <KeyValueEditor 
        items={activeRequest?.params || []}
        onupdate={handleParamsUpdate}
        placeholder={{ key: 'Parameter name', value: 'Value' }}
      />
    {:else if activeTab === 'headers'}
      <KeyValueEditor 
        items={activeRequest?.headers || []}
        onupdate={handleHeadersUpdate}
        placeholder={{ key: 'Header name', value: 'Value' }}
      />
    {:else if activeTab === 'body'}
      <BodyEditor 
        body={activeRequest?.body || ''}
        bodyType={activeRequest?.bodyType || 'json'}
        onupdate={handleBodyUpdate}
      />
    {:else if activeTab === 'auth'}
      <div class="auth-panel">
        <p class="placeholder-text">Authentication options coming soon</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .request-panel {
    display: flex;
    flex-direction: column;
    background: var(--bg-main);
  }

  .url-bar {
    display: flex;
    gap: 8px;
    padding: 12px;
    background: var(--bg-panel);
  }

  .method-select {
    width: 100px;
    padding: 8px 12px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    outline: none;
  }

  .method-select:focus {
    border-color: var(--border-focus);
  }

  .method-select option {
    background: var(--bg-panel);
    font-weight: 600;
  }

  .url-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-main);
    font-size: 13px;
    font-family: 'Fira Code', monospace;
    outline: none;
  }

  .url-input:focus {
    border-color: var(--border-focus);
  }

  .url-input::placeholder {
    color: var(--text-muted);
  }

  .send-btn {
    padding: 8px 24px;
    background: var(--border-focus);
    border: none;
    border-radius: 4px;
    color: white;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
    min-width: 80px;
  }

  .send-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .send-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .tabs {
    display: flex;
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
    background: var(--border-focus);
    border-radius: 10px;
    font-size: 10px;
    color: white;
  }

  .tab-content {
    padding: 12px;
    overflow: auto;
  }

  .auth-panel {
    padding: 24px;
    text-align: center;
  }

  .placeholder-text {
    color: var(--text-muted);
    font-size: 13px;
  }
</style>