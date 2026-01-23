<script lang="ts">
  // We receive a callback function directly from the parent
  export let onSelect: (id: string) => void = () => {}; 
  export let activeSection: string = 'general';

  const sections = [
    { id: 'general', label: 'General', icon: 'settings' },
    { id: 'terminal', label: 'Terminal', icon: 'terminal' },
    { id: 'files', label: 'Filesystem', icon: 'folder' },
    { id: 'editor', label: 'Editor', icon: 'code' }
  ];

  function handleClick(id: string) {
    activeSection = id;
    // Call the parent's function to trigger the scroll
    onSelect(id);
  }
</script>

<div class="sidebar">
  {#each sections as section}
    <button 
      class:active={activeSection === section.id} 
      on:click={() => handleClick(section.id)}
    >
      <span class="label">{section.label}</span>
    </button>
  {/each}
</div>

<style>
  .sidebar {
    width: 100%;
    height: 100%;
    padding: 10px 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  button {
    background: transparent;
    border: none;
    text-align: left;
    padding: 10px 20px;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 0.95rem;
    border-left: 3px solid transparent;
    transition: all 0.2s;
  }

  button:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  button.active {
    background: var(--hover-bg);
    color: var(--text-main);
    border-left-color: var(--border-focus);
  }
</style>