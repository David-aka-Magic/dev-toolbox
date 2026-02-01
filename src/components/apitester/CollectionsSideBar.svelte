<script lang="ts">
  import { apiStore, type ApiRequest } from '$lib/stores/apiStore';

  let { 
    ontoggle = () => {} 
  }: { 
    ontoggle?: () => void 
  } = $props();

  let newCollectionName = $state('');
  let showNewCollection = $state(false);
  let editingId: string | null = $state(null);
  let editingName = $state('');

  let collections = $derived($apiStore.collections);
  let history = $derived($apiStore.history);
  let activeRequestId = $derived($apiStore.activeRequest?.id);

  function createCollection() {
    if (!newCollectionName.trim()) return;
    apiStore.createCollection(newCollectionName.trim());
    newCollectionName = '';
    showNewCollection = false;
  }

  function selectRequest(request: ApiRequest) {
    apiStore.setActiveRequest(request);
  }

  function saveToCollection(collectionId: string) {
    const activeRequest = $apiStore.activeRequest;
    if (!activeRequest) return;
    apiStore.saveRequestToCollection(collectionId, activeRequest);
  }

  function deleteRequest(e: Event, collectionId: string, requestId: string) {
    e.stopPropagation();
    apiStore.deleteRequestFromCollection(collectionId, requestId);
  }

  function deleteCollection(collectionId: string) {
    if (confirm('Delete this collection and all its requests?')) {
      apiStore.deleteCollection(collectionId);
    }
  }

  function toggleCollection(collectionId: string) {
    apiStore.toggleCollection(collectionId);
  }

  function startRename(e: Event, id: string, name: string) {
    e.stopPropagation();
    editingId = id;
    editingName = name;
  }

  function finishRename() {
    if (editingId && editingName.trim()) {
      apiStore.renameCollection(editingId, editingName.trim());
    }
    editingId = null;
    editingName = '';
  }

  function clearHistory() {
    if (confirm('Clear all history?')) {
      apiStore.clearHistory();
    }
  }

  function getMethodColor(method: string): string {
    const colors: Record<string, string> = {
      GET: '#22c55e',
      POST: '#eab308',
      PUT: '#3b82f6',
      PATCH: '#a855f7',
      DELETE: '#ef4444',
    };
    return colors[method] || '#6b7280';
  }

  function formatUrl(url: string): string {
    try {
      const parsed = new URL(url);
      return parsed.pathname + parsed.search;
    } catch {
      return url;
    }
  }
</script>

<div class="collections-sidebar">
  <div class="sidebar-header">
    <h3>Collections</h3>
    <button class="icon-btn" onclick={ontoggle} title="Hide Sidebar">
      ◀
    </button>
  </div>

  <div class="section">
    <div class="section-header">
      <span>Collections</span>
      <button class="add-btn" onclick={() => showNewCollection = true} title="New Collection">
        +
      </button>
    </div>

    {#if showNewCollection}
      <div class="new-collection">
        <input 
          type="text" 
          bind:value={newCollectionName}
          placeholder="Collection name"
          onkeydown={(e) => e.key === 'Enter' && createCollection()}
        />
        <button class="save-btn" onclick={createCollection}>Save</button>
        <button class="cancel-btn" onclick={() => showNewCollection = false}>×</button>
      </div>
    {/if}

    <div class="collections-list">
      {#each collections as collection}
        <div class="collection-item">
          <div class="collection-header">
            {#if editingId === collection.id}
              <input 
                type="text"
                bind:value={editingName}
                onblur={finishRename}
                onkeydown={(e) => e.key === 'Enter' && finishRename()}
                class="rename-input"
              />
            {:else}
              <button 
                class="collection-name" 
                onclick={() => toggleCollection(collection.id)}
                ondblclick={(e) => startRename(e, collection.id, collection.name)}
              >
                <span class="expand-icon">{collection.expanded ? '▼' : '▶'}</span>
                <span>{collection.name}</span>
                <span class="count">({collection.requests.length})</span>
              </button>
            {/if}
            <div class="collection-actions">
              <button 
                class="action-btn" 
                onclick={() => saveToCollection(collection.id)} 
                title="Save current request here"
              >
                +
              </button>
              <button 
                class="action-btn delete" 
                onclick={() => deleteCollection(collection.id)}
                title="Delete collection"
              >
                🗑
              </button>
            </div>
          </div>
          
          {#if collection.expanded}
            <div class="requests-list">
              {#each collection.requests as request}
                <div 
                  class="request-item"
                  class:active={request.id === activeRequestId}
                  onclick={() => selectRequest(request)}
                  onkeydown={(e) => e.key === 'Enter' && selectRequest(request)}
                  role="button"
                  tabindex="0"
                >
                  <span class="method" style="color: {getMethodColor(request.method)}">
                    {request.method}
                  </span>
                  <span class="url" title={request.url}>
                    {request.name || formatUrl(request.url)}
                  </span>
                  <span 
                    class="delete-btn"
                    onclick={(e) => deleteRequest(e, collection.id, request.id)}
                    onkeydown={(e) => e.key === 'Enter' && deleteRequest(e, collection.id, request.id)}
                    role="button"
                    tabindex="0"
                  >
                    ×
                  </span>
                </div>
              {/each}
              {#if collection.requests.length === 0}
                <div class="empty-text">No requests saved</div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}

      {#if collections.length === 0}
        <div class="empty-text">No collections yet</div>
      {/if}
    </div>
  </div>

  <div class="section">
    <div class="section-header">
      <span>History</span>
      {#if history.length > 0}
        <button class="clear-btn" onclick={clearHistory}>Clear</button>
      {/if}
    </div>

    <div class="history-list">
      {#each history as request}
        <div 
          class="request-item"
          class:active={request.id === activeRequestId}
          onclick={() => selectRequest(request)}
          onkeydown={(e) => e.key === 'Enter' && selectRequest(request)}
          role="button"
          tabindex="0"
        >
          <span class="method" style="color: {getMethodColor(request.method)}">
            {request.method}
          </span>
          <span class="url" title={request.url}>
            {formatUrl(request.url)}
          </span>
        </div>
      {/each}
      {#if history.length === 0}
        <div class="empty-text">No history yet</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .collections-sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    border-bottom: 1px solid var(--border);
  }

  .sidebar-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .icon-btn {
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 10px;
    transition: color 0.15s;
  }

  .icon-btn:hover {
    color: var(--text-main);
  }

  .section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-bottom: 1px solid var(--border);
  }

  .section:last-child {
    border-bottom: none;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    background: var(--bg-main);
  }

  .add-btn, .clear-btn {
    padding: 2px 8px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .add-btn:hover, .clear-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .new-collection {
    display: flex;
    gap: 4px;
    padding: 8px 12px;
    background: var(--bg-main);
  }

  .new-collection input {
    flex: 1;
    padding: 6px 8px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-main);
    font-size: 12px;
    outline: none;
  }

  .new-collection input:focus {
    border-color: var(--border-focus);
  }

  .save-btn {
    padding: 6px 10px;
    background: var(--border-focus);
    border: none;
    border-radius: 3px;
    color: white;
    font-size: 11px;
    cursor: pointer;
  }

  .cancel-btn {
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
  }

  .collections-list, .history-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .collection-item {
    margin-bottom: 4px;
  }

  .collection-header {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .collection-name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-main);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }

  .collection-name:hover {
    background: var(--hover-bg);
  }

  .expand-icon {
    font-size: 8px;
    color: var(--text-muted);
  }

  .count {
    color: var(--text-muted);
    font-size: 11px;
  }

  .collection-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .collection-header:hover .collection-actions {
    opacity: 1;
  }

  .action-btn {
    padding: 4px 8px;
    background: transparent;
    border: none;
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .action-btn.delete:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .rename-input {
    flex: 1;
    padding: 6px 8px;
    background: var(--bg-input);
    border: 1px solid var(--border-focus);
    border-radius: 3px;
    color: var(--text-main);
    font-size: 12px;
    outline: none;
  }

  .requests-list {
    padding-left: 20px;
  }

  .request-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-main);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }

  .request-item:hover {
    background: var(--hover-bg);
  }

  .request-item.active {
    background: var(--selection);
  }

  .method {
    font-size: 10px;
    font-weight: 600;
    width: 40px;
    flex-shrink: 0;
  }

  .url {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-muted);
    font-family: 'Fira Code', monospace;
    font-size: 11px;
  }

  .delete-btn {
    padding: 2px 6px;
    background: transparent;
    border: none;
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    opacity: 0;
    transition: all 0.15s;
  }

  .request-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .empty-text {
    padding: 16px 12px;
    color: var(--text-muted);
    font-size: 12px;
    text-align: center;
  }
</style>