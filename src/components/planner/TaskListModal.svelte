<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from '../ui/Modal.svelte';
  import { createTaskList, updateTaskList, deleteTaskList } from '$lib/stores/plannerStore';
  import type { TaskList } from '$lib/stores/plannerStore';

  let {
    open = false,
    list = null,
  }: {
    open?: boolean;
    list?: TaskList | null;
  } = $props();

  const dispatch = createEventDispatcher<{ close: void }>();

  const PRESET_COLORS = [
    '#3b82f6', '#ef4444', '#10b981', '#f59e0b',
    '#8b5cf6', '#ec4899', '#06b6d4', '#f97316',
    '#84cc16', '#6b7280',
  ];

  let name  = $state('');
  let color = $state('#3b82f6');
  let saving = $state(false);
  let error  = $state('');

  $effect(() => {
    if (!open) return;
    if (list) {
      name  = list.name;
      color = list.color;
    } else {
      name  = '';
      color = '#3b82f6';
    }
    error = '';
  });

  async function handleSave() {
    if (!name.trim()) { error = 'Name is required.'; return; }
    saving = true; error = '';
    try {
      if (list) await updateTaskList(list.id, name.trim(), color);
      else      await createTaskList(name.trim(), color);
      dispatch('close');
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!list) return;
    saving = true;
    try {
      await deleteTaskList(list.id);
      dispatch('close');
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function handleClose() { dispatch('close'); }
</script>

<Modal {open} title={list ? 'Edit List' : 'New List'} width="360px" on:close={handleClose}>
  <div class="form">
    <div class="field">
      <label>Name</label>
      <input class="text-in" type="text" bind:value={name} placeholder="List name" autofocus />
    </div>

    <div class="field">
      <label>Color</label>
      <div class="swatches">
        {#each PRESET_COLORS as c}
          <button
            class="swatch"
            class:active={color === c}
            style:background={c}
            onclick={() => (color = c)}
            title={c}
          ></button>
        {/each}
      </div>
    </div>

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </div>

  <div slot="footer" class="footer-btns">
    {#if list}
      <button class="btn-delete" onclick={handleDelete} disabled={saving}>Delete</button>
    {/if}
    <div class="spacer"></div>
    <button class="btn-cancel" onclick={handleClose} disabled={saving}>Cancel</button>
    <button class="btn-save" onclick={handleSave} disabled={saving || !name.trim()}>
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

  .field { display: flex; flex-direction: column; gap: 6px; }
  .field label { font-size: 0.82rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }

  .text-in {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px 10px;
    border-radius: 5px;
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
    box-sizing: border-box;
  }
  .text-in:focus { border-color: var(--border-focus); }

  .swatches { display: flex; gap: 8px; flex-wrap: wrap; }
  .swatch {
    width: 26px; height: 26px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; transition: transform 0.1s, border-color 0.1s;
  }
  .swatch:hover { transform: scale(1.15); }
  .swatch.active { border-color: var(--text-main); box-shadow: 0 0 0 2px var(--bg-header); }

  .error { color: #ef4444; font-size: 0.85rem; margin: 0; }

  .footer-btns { display: flex; align-items: center; gap: 8px; width: 100%; }
  .spacer { flex: 1; }
  button { padding: 7px 16px; border-radius: 5px; border: none; cursor: pointer; font-size: 0.88rem; font-weight: 500; font-family: inherit; transition: background 0.15s; }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-cancel { background: transparent; color: var(--text-muted); }
  .btn-cancel:hover:not(:disabled) { background: var(--hover-bg); color: var(--text-main); }
  .btn-save { background: var(--border-focus); color: #fff; }
  .btn-save:hover:not(:disabled) { opacity: 0.85; }
  .btn-delete { background: transparent; color: #ef4444; border: 1px solid #ef4444; }
  .btn-delete:hover:not(:disabled) { background: #ef4444; color: #fff; }
</style>
