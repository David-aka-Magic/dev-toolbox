<script lang="ts">
  import { currentProject } from '$lib/stores/ganttStore';
  import ProjectSelector from './ProjectSelector.svelte';
  import GanttChart from './GanttChart.svelte';
</script>

<div class="gantt-view">
  {#if $currentProject}
    <GanttChart />
  {:else}
    <ProjectSelector />
  {/if}
</div>

<style>
  .gantt-view {
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--bg-main);
  }
</style>
