<script lang="ts">
  import RequestPanel from './RequestPanel.svelte';
  import ResponsePanel from './ResponsePanel.svelte';
  import CollectionsSidebar from './CollectionsSidebar.svelte';
  import { apiStore } from '$lib/stores/apiStore';

  let showSidebar = $state(true);

  function toggleSidebar() {
    showSidebar = !showSidebar;
  }

  function handleNewRequest() {
    apiStore.createNewRequest();
  }
</script>

<div class="api-tester">
  {#if showSidebar}
    <div class="sidebar">
      <CollectionsSidebar ontoggle={toggleSidebar} />
    </div>
  {/if}

  <div class="main-content">
    <div class="toolbar">
      {#if !showSidebar}
        <button class="toolbar-btn" onclick={toggleSidebar} title="Show Collections">
          <span class="icon">☰</span>
        </button>
      {/if}
      <button class="toolbar-btn new-btn" onclick={handleNewRequest}>
        <span class="icon">+</span>
        <span>New Request</span>
      </button>
    </div>

    <div class="panels-container">
      <div class="request-section">
        <RequestPanel />
      </div>
      <div class="response-section">
        <ResponsePanel />
      </div>
    </div>
  </div>
</div>

<style>
  .api-tester {
    display: flex;
    height: 100%;
    width: 100%;
    background: var(--bg-main);
    color: var(--text-main);
  }

  .sidebar {
    width: 280px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    background: var(--bg-panel);
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-panel);
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-main);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s;
  }

  .toolbar-btn:hover {
    background: var(--hover-bg);
    border-color: var(--border-focus);
  }

  .new-btn {
    background: var(--border-focus);
    border-color: var(--border-focus);
    color: white;
  }

  .new-btn:hover {
    background: #2563eb;
  }

  .icon {
    font-size: 14px;
  }

  .panels-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .request-section {
    flex: 0 0 auto;
    max-height: 50%;
    overflow: auto;
    border-bottom: 1px solid var(--border);
  }

  .response-section {
    flex: 1;
    min-height: 200px;
    overflow: hidden;
  }
</style>