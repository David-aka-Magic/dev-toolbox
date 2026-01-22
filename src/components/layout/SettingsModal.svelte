<script lang="ts">
  export let onClose: () => void;
</script>

<div class="modal-backdrop" on:click={onClose}>
  <div class="modal-content" on:click|stopPropagation>
    <div class="modal-header">
      <h3>Settings</h3>
      <button class="close-btn" on:click={onClose}>✕</button>
    </div>
    <div class="modal-body">
      <p>Application settings will appear here.</p>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-content {
    width: 500px;
    height: 300px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    height: 40px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    color: var(--text-primary);
  }

  .modal-body {
    padding: 16px;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 16px;
  }
</style>