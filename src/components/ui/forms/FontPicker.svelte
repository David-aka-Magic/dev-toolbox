<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let value: string = '';
  export let label: string = '';

  let query = value;
  let fonts: string[] = [];
  let filtered: string[] = [];
  let isOpen = false;
  let highlightedIndex = -1;
  let inputEl: HTMLInputElement;
  let listEl: HTMLUListElement;
  let wrapperEl: HTMLDivElement;
  let loading = true;

  onMount(async () => {
    try {
      fonts = await invoke<string[]>('get_system_fonts');
    } catch (e) {
      console.error('Failed to load system fonts:', e);
      fonts = [];
    }
    loading = false;
    query = value;
    updateFiltered();
  });

  $: inputFontStyle = value ? `font-family: '${value}', sans-serif` : '';

  function updateFiltered() {
    const q = query.toLowerCase().trim();
    if (!q) {
      filtered = fonts.slice(0, 100);
    } else {
      filtered = fonts.filter(f => f.toLowerCase().includes(q)).slice(0, 100);
    }
    highlightedIndex = -1;
  }

  function handleInput() {
    value = query;
    isOpen = true;
    updateFiltered();
  }

  function handleFocus() {
    isOpen = true;
    updateFiltered();
  }

  function handleClickOutside(e: MouseEvent) {
    if (wrapperEl && !wrapperEl.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  function selectFont(font: string) {
    query = font;
    value = font;
    isOpen = false;
    inputEl?.focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        isOpen = true;
        updateFiltered();
        e.preventDefault();
        return;
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        highlightedIndex = Math.min(highlightedIndex + 1, filtered.length - 1);
        scrollToHighlighted();
        break;
      case 'ArrowUp':
        e.preventDefault();
        highlightedIndex = Math.max(highlightedIndex - 1, 0);
        scrollToHighlighted();
        break;
      case 'Enter':
        e.preventDefault();
        if (highlightedIndex >= 0 && highlightedIndex < filtered.length) {
          selectFont(filtered[highlightedIndex]);
        } else {
          isOpen = false;
        }
        break;
      case 'Escape':
        isOpen = false;
        break;
    }
  }

  function scrollToHighlighted() {
    if (!listEl) return;
    const item = listEl.children[highlightedIndex] as HTMLElement;
    if (item) {
      item.scrollIntoView({ block: 'nearest' });
    }
  }
</script>

<svelte:window on:mousedown={handleClickOutside} />

<div class="form-group font-picker" bind:this={wrapperEl}>
  {#if label}
    <label>{label}</label>
  {/if}

  <div class="input-wrapper">
    <input
      bind:this={inputEl}
      type="text"
      bind:value={query}
      on:input={handleInput}
      on:focus={handleFocus}
      on:keydown={handleKeydown}
      placeholder={loading ? 'Loading fonts...' : 'Search fonts...'}
      autocomplete="off"
      spellcheck="false"
      style={inputFontStyle}
    />
    <button
      class="toggle-btn"
      tabindex="-1"
      on:click={() => { isOpen = !isOpen; if (isOpen) { updateFiltered(); inputEl?.focus(); } }}
      type="button"
    >
      <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
        <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
  </div>

  {#if isOpen && filtered.length > 0}
    <ul class="dropdown" bind:this={listEl}>
      {#each filtered as font, i}
        <li
          class:highlighted={i === highlightedIndex}
          on:mousedown|preventDefault={() => selectFont(font)}
          on:mouseenter={() => highlightedIndex = i}
        >
          <span class="font-preview" style="font-family: '{font}', sans-serif;">{font}</span>
        </li>
      {/each}
    </ul>
  {:else if isOpen && !loading && query}
    <ul class="dropdown">
      <li class="no-results">No fonts match "{query}"</li>
    </ul>
  {/if}
</div>

<style>
  .form-group { display: flex; flex-direction: column; gap: 8px; width: 100%; position: relative; }
  label { font-size: 0.85rem; font-weight: 500; color: var(--text-main); }

  .input-wrapper {
    display: flex;
    position: relative;
  }

  input {
    flex: 1;
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 10px;
    padding-right: 32px;
    border-radius: 4px;
    outline: none;
    font-size: 0.9rem;
    transition: border-color 0.2s;
    width: 100%;
    box-sizing: border-box;
  }

  input:focus { border-color: var(--border-focus); }

  .toggle-btn {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
  }

  .toggle-btn:hover { color: var(--text-main); background: var(--hover-bg); }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 100;
    background: var(--bg-panel, #1e1e2e);
    border: 1px solid var(--border);
    border-radius: 4px;
    margin-top: 4px;
    max-height: 250px;
    overflow-y: auto;
    list-style: none;
    padding: 4px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.3);
  }

  .dropdown::-webkit-scrollbar { width: 8px; }
  .dropdown::-webkit-scrollbar-track { background: transparent; }
  .dropdown::-webkit-scrollbar-thumb { background: var(--border); border-radius: 4px; }
  .dropdown::-webkit-scrollbar-thumb:hover { background: var(--text-muted); }

  li {
    padding: 8px 10px;
    cursor: pointer;
    border-radius: 3px;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  li.highlighted { background: var(--hover-bg); }
  li.no-results { color: var(--text-muted); cursor: default; font-style: italic; font-size: 0.88rem; }

  .font-preview {
    font-size: 0.95rem;
    display: block;
    line-height: 1.4;
  }
</style>