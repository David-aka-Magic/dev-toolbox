<script lang="ts">
  import { tabs, activeTabId } from '$lib/stores/terminalStore';
  import Terminal from './Terminal.svelte';

  // We don't need much logic here because the stores handle the state.
  // The HTML below handles the switching visibility.
</script>

<div class="terminal-view-container">
  {#each $tabs as tab (tab.id)}
    <div 
      class="terminal-instance-wrapper" 
      style:display={$activeTabId === tab.id ? 'block' : 'none'}
    >
      <Terminal 
        terminalId={tab.id} 
        profile={tab.profile} 
      />
    </div>
  {/each}
</div>

<style>
  .terminal-view-container {
    width: 100%;
    /* Fill remaining height (100vh - TitleBar height) */
    height: calc(100vh - 38px); 
    background-color: var(--bg);
    position: relative;
  }

  .terminal-instance-wrapper {
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
  }
</style>