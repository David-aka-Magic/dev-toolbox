<script lang="ts">
  import {
    projects, currentProject, tasks,
    loadProjects, loadProjectData,
    type GanttProject,
  } from '$lib/stores/ganttStore';
  import ProjectModal from './ProjectModal.svelte';

  let modalOpen    = $state(false);
  let editProject: GanttProject | null = $state(null);

  // Task counts per project cached after loading
  let taskCounts = $state<Record<string, number>>({});

  $effect(() => { loadProjects(); });

  function openCreate() { editProject = null; modalOpen = true; }
  function openEdit(e: MouseEvent, p: GanttProject) {
    e.stopPropagation();
    editProject = p;
    modalOpen = true;
  }

  async function selectProject(p: GanttProject) {
    await loadProjectData(p.id);
  }

  function onSaved(p: GanttProject) {
    // If this was a create, jump straight in
    if (!editProject) selectProject(p);
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function descriptionPreview(desc: string | null): string {
    if (!desc) return '';
    return desc.length > 80 ? desc.slice(0, 77) + '…' : desc;
  }
</script>

<div class="selector">

  <div class="header">
    <h1>Gantt Projects</h1>
    <p class="subtitle">Select a project to open the chart, or create a new one.</p>
  </div>

  <div class="grid">

    <!-- New project card -->
    <button class="card card-new" onclick={openCreate}>
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      <span>New Project</span>
    </button>

    {#each $projects as p (p.id)}
      <button class="card card-project" onclick={() => selectProject(p)}>

        <div class="card-top">
          <span class="card-name">{p.name}</span>
          <button class="edit-btn" onclick={(e) => openEdit(e, p)} title="Edit project">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
        </div>

        {#if p.description}
          <p class="card-desc">{descriptionPreview(p.description)}</p>
        {/if}

        <div class="card-meta">
          <span class="meta-item">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="18" rx="2"/>
              <line x1="16" y1="2" x2="16" y2="6"/>
              <line x1="8" y1="2" x2="8" y2="6"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
            {formatDate(p.start_date)}
          </span>
          <span class="meta-item zoom-badge">{p.zoom_level}</span>
        </div>

      </button>
    {/each}

  </div>

</div>

<ProjectModal
  open={modalOpen}
  project={editProject}
  onclose={() => (modalOpen = false)}
  onsaved={onSaved}
  ondeleted={() => {}}
/>

<style>
  .selector {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-main);
    overflow-y: auto;
    padding: 40px 48px;
    box-sizing: border-box;
  }

  .header {
    margin-bottom: 32px;
  }

  h1 {
    margin: 0 0 6px;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-main);
  }

  .subtitle {
    margin: 0;
    font-size: 0.88rem;
    color: var(--text-muted);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 16px;
  }

  .card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    transition: border-color 0.15s, background 0.15s;
    padding: 0;
  }
  .card:hover {
    border-color: var(--border-focus);
    background: var(--hover-bg);
  }

  /* ── New project card ── */
  .card-new {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    min-height: 130px;
    color: var(--text-muted);
    border-style: dashed;
  }
  .card-new:hover {
    color: var(--border-focus);
    border-color: var(--border-focus);
  }
  .card-new span {
    font-size: 0.88rem;
    font-weight: 600;
  }

  /* ── Project card ── */
  .card-project {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    min-height: 130px;
  }

  .card-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }

  .card-name {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-main);
    line-height: 1.3;
    flex: 1;
    word-break: break-word;
  }

  .edit-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 3px;
    border-radius: 4px;
    display: flex;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
  }
  .card-project:hover .edit-btn { opacity: 1; }
  .edit-btn:hover { color: var(--text-main); background: var(--hover-bg); }

  .card-desc {
    margin: 0;
    font-size: 0.8rem;
    color: var(--text-muted);
    line-height: 1.45;
    flex: 1;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: auto;
    flex-wrap: wrap;
  }

  .meta-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .zoom-badge {
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 1px 8px;
    text-transform: capitalize;
    font-size: 0.72rem;
  }
</style>
