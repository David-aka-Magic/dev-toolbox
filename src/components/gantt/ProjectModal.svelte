<script lang="ts">
  import Modal from '../ui/Modal.svelte';
  import {
    createProject, updateProject, deleteProject,
    type GanttProject, type GanttProjectInput,
  } from '$lib/stores/ganttStore';

  let {
    open = false,
    project = null,
    onclose = () => {},
    onsaved = (_p: GanttProject) => {},
    ondeleted = () => {},
  }: {
    open?: boolean;
    project?: GanttProject | null;
    onclose?: () => void;
    onsaved?: (p: GanttProject) => void;
    ondeleted?: () => void;
  } = $props();

  let name        = $state('');
  let description = $state('');
  let startDate   = $state('');
  let saving      = $state(false);
  let error       = $state('');

  // Sync fields when modal opens or project changes
  $effect(() => {
    if (open) {
      name        = project?.name        ?? '';
      description = project?.description ?? '';
      startDate   = project?.start_date  ?? new Date().toISOString().split('T')[0];
      error       = '';
    }
  });

  async function save() {
    if (!name.trim()) { error = 'Name is required.'; return; }
    if (!startDate)   { error = 'Start date is required.'; return; }
    saving = true;
    error  = '';
    try {
      const input: GanttProjectInput = {
        name: name.trim(),
        description: description.trim() || null,
        start_date: startDate,
      };
      const saved = project
        ? await updateProject(project.id, input)
        : await createProject(input);
      onsaved(saved);
      onclose();
    } catch (e: any) {
      error = e?.toString() ?? 'Save failed.';
    } finally {
      saving = false;
    }
  }

  async function remove() {
    if (!project) return;
    if (!confirm(`Delete project "${project.name}" and all its tasks?`)) return;
    saving = true;
    try {
      await deleteProject(project.id);
      ondeleted();
      onclose();
    } catch (e: any) {
      error = e?.toString() ?? 'Delete failed.';
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); save(); }
  }
</script>

<Modal
  open={open}
  title={project ? 'Edit Project' : 'New Project'}
  width="440px"
  on:close={onclose}
>
  <div class="form" onkeydown={handleKeydown} role="presentation">

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <label>
      <span>Name</span>
      <input
        type="text"
        bind:value={name}
        placeholder="Project name"
        autofocus
      />
    </label>

    <label>
      <span>Description</span>
      <textarea
        bind:value={description}
        placeholder="Optional description…"
        rows="3"
      ></textarea>
    </label>

    <label>
      <span>Start Date</span>
      <input type="date" bind:value={startDate} />
    </label>

  </div>

  <div slot="footer" class="footer">
    {#if project}
      <button class="btn-danger" onclick={remove} disabled={saving}>Delete</button>
      <div style="flex:1"></div>
    {/if}
    <button class="btn-cancel" onclick={onclose} disabled={saving}>Cancel</button>
    <button class="btn-save"   onclick={save}    disabled={saving}>
      {saving ? 'Saving…' : 'Save'}
    </button>
  </div>
</Modal>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  label span {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  input, textarea {
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.88rem;
    padding: 7px 10px;
    outline: none;
    resize: vertical;
  }
  input:focus, textarea:focus {
    border-color: var(--border-focus);
  }
  input[type="date"] { cursor: pointer; }

  .error {
    margin: 0;
    font-size: 0.82rem;
    color: #ef4444;
    background: color-mix(in srgb, #ef4444 10%, transparent);
    border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
    border-radius: 5px;
    padding: 6px 10px;
  }

  .footer {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  button {
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.84rem;
    font-weight: 500;
    padding: 6px 16px;
    transition: opacity 0.15s;
  }
  button:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-save   { background: var(--border-focus); color: #fff; }
  .btn-save:hover:not(:disabled) { opacity: 0.85; }
  .btn-cancel { background: var(--hover-bg); color: var(--text-muted); border: 1px solid var(--border); }
  .btn-cancel:hover:not(:disabled) { color: var(--text-main); }
  .btn-danger { background: color-mix(in srgb, #ef4444 15%, transparent); color: #ef4444; border: 1px solid color-mix(in srgb, #ef4444 30%, transparent); }
  .btn-danger:hover:not(:disabled) { background: color-mix(in srgb, #ef4444 25%, transparent); }
</style>
