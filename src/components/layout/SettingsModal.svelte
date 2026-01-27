<script lang="ts">
  import { tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Stores
  import { isSettingsOpen, settings } from '$lib/stores/settingsStore';
  import { currentView } from '$lib/stores/viewStore';
  import { theme } from '$lib/stores/theme';
  import { viewMode, sortConfig } from '$lib/stores/viewModeStore';

  // Components
  import Modal from '../ui/Modal.svelte';
  import SettingsSidebar from './SettingsSidebar.svelte';
  import Input from '../ui/forms/Input.svelte';
  import Select from '../ui/forms/Select.svelte';
  import Checkbox from '../ui/forms/Checkbox.svelte';

  // State
  let activeSection = $state('general');
  let scrollContainer: HTMLElement | undefined = $state();
  let isManualScroll = $state(false);
  let cacheSize = $state('Calculating...');
  let isClearingCache = $state(false);

  // === OPTIONS ===
  
  const themeOptions = [
    { value: 'dark', label: 'Dark (Default)' },
    { value: 'light', label: 'Light' },
    { value: 'dracula', label: 'Dracula' },
    { value: 'monokai', label: 'Monokai' }
  ];
  
  const cursorOptions = [
    { value: 'block', label: 'Block █' },
    { value: 'bar', label: 'Line |' },
    { value: 'underline', label: 'Underline _' }
  ];

  const tabOptions = [
    { value: '2', label: '2 Spaces' },
    { value: '4', label: '4 Spaces' },
    { value: '8', label: '8 Spaces' }
  ];

  const wrapOptions = [
    { value: 'off', label: 'Off' },
    { value: 'on', label: 'On' },
    { value: 'wordWrapColumn', label: 'Word Wrap Column' }
  ];

  const viewModeOptions = [
    { value: 'grid', label: 'Grid' },
    { value: 'list', label: 'List' },
    { value: 'details', label: 'Details' }
  ];

  const sortFieldOptions = [
    { value: 'name', label: 'Name' },
    { value: 'size', label: 'Size' },
    { value: 'modified', label: 'Date Modified' },
    { value: 'type', label: 'Type' }
  ];

  const sortDirectionOptions = [
    { value: 'asc', label: 'Ascending' },
    { value: 'desc', label: 'Descending' }
  ];

  const doubleClickOptions = [
    { value: 'open', label: 'Open (navigate/default app)' },
    { value: 'preview', label: 'Preview in viewer' },
    { value: 'editor', label: 'Open in Editor' }
  ];

  const iconThemeOptions = [
    { value: 'material', label: 'Material Icons' },
    { value: 'minimal', label: 'Minimal' },
    { value: 'none', label: 'None' }
  ];

  // === EFFECTS ===

  $effect(() => {
    if ($isSettingsOpen) {
      let startSection = 'general';
      if ($currentView === 'editor') startSection = 'editor';
      else if ($currentView === 'terminal') startSection = 'terminal';
      else if ($currentView === 'files') startSection = 'files';
      
      activeSection = startSection;
      scrollToSection(startSection);
      loadCacheSize();
    }
  });

  // === FUNCTIONS ===

  async function loadCacheSize() {
    try {
      const sizeBytes = await invoke<number>('get_thumbnail_cache_size');
      cacheSize = formatBytes(sizeBytes);
    } catch (err) {
      cacheSize = 'Unknown';
    }
  }

  async function clearThumbnailCache() {
    isClearingCache = true;
    try {
      await invoke('clear_thumbnail_cache');
      cacheSize = '0 B';
    } catch (err) {
      console.error('Failed to clear cache:', err);
      alert('Failed to clear thumbnail cache: ' + err);
    } finally {
      isClearingCache = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function handleSidebarSelect(id: string) {
    isManualScroll = true;
    activeSection = id;
    scrollToSection(id);
    setTimeout(() => { isManualScroll = false; }, 500);
  }

  async function scrollToSection(id: string) {
    await tick();
    const element = document.getElementById(`section-${id}`);
    if (element && scrollContainer) {
      const top = element.offsetTop - scrollContainer.offsetTop; 
      scrollContainer.scrollTo({ top, behavior: 'smooth' });
    }
  }

  function handleScroll() {
    if (isManualScroll || !scrollContainer) return;
    const sections = ['general', 'terminal', 'files', 'editor'];
    const scrollPos = scrollContainer.scrollTop + 100; 

    for (const id of sections) {
      const el = document.getElementById(`section-${id}`);
      if (el && el.offsetTop <= scrollPos && (el.offsetTop + el.offsetHeight) > scrollPos) {
        if (activeSection !== id) activeSection = id;
      }
    }
  }

  function handleDefaultViewChange(e: Event) {
    const value = (e.target as HTMLSelectElement).value as 'grid' | 'list' | 'details';
    $settings.fileDefaultView = value;
    $viewMode = value;
  }

  function handleDefaultSortFieldChange(e: Event) {
    const value = (e.target as HTMLSelectElement).value as 'name' | 'size' | 'modified' | 'type';
    $settings.fileDefaultSortField = value;
    $sortConfig = { ...$sortConfig, field: value };
  }

  function handleDefaultSortDirectionChange(e: Event) {
    const value = (e.target as HTMLSelectElement).value as 'asc' | 'desc';
    $settings.fileDefaultSortDirection = value;
    $sortConfig = { ...$sortConfig, direction: value };
  }

  function close() { $isSettingsOpen = false; }
</script>

<Modal 
  open={$isSettingsOpen} 
  title="Settings" 
  width="70vw"
  height="70vh"
  on:close={close}
>
  <div class="settings-layout">
    
    <div class="sidebar-col">
      <SettingsSidebar 
        {activeSection}
        onSelect={handleSidebarSelect} 
      />
    </div>

    <div 
      class="content-col" 
      bind:this={scrollContainer}
      onscroll={handleScroll}
    >
      <!-- GENERAL -->
      <section id="section-general">
        <div class="section-header">
          <h3>Appearance & Behavior</h3>
          <p>Customize the look and feel of the toolkit.</p>
        </div>

        <Select 
          label="Color Theme" 
          options={themeOptions} 
          bind:value={$theme} 
        />
      </section>

      <hr class="divider" />

      <!-- TERMINAL -->
      <section id="section-terminal">
        <div class="section-header">
          <h3>Terminal Configuration</h3>
          <p>Manage your integrated shell environment.</p>
        </div>

        <Input 
          label="Default Shell Path" 
          bind:value={$settings.termShellPath} 
        />

        <div class="row">
          <div class="half">
            <Input type="number" label="Font Size" bind:value={$settings.termFontSize} />
          </div>
          <div class="half">
            <Select label="Cursor Style" options={cursorOptions} bind:value={$settings.termCursorStyle} />
          </div>
        </div>
      </section>
      
      <hr class="divider" />

      <!-- FILE MANAGER -->
      <section id="section-files">
        <div class="section-header">
          <h3>File Manager</h3>
          <p>Control how files and folders are displayed.</p>
        </div>

        <!-- Display Settings -->
        <h4 class="subsection-header">Display</h4>
        
        <div class="row">
          <div class="half">
            <div class="form-group">
              <label>Default View</label>
              <select 
                class="select-input"
                value={$settings.fileDefaultView}
                onchange={handleDefaultViewChange}
              >
                {#each viewModeOptions as opt}
                  <option value={opt.value}>{opt.label}</option>
                {/each}
              </select>
            </div>
          </div>
          <div class="half">
            <Select 
              label="Icon Theme" 
              options={iconThemeOptions} 
              bind:value={$settings.fileIconTheme}
            />
          </div>
        </div>

        <div class="row">
          <div class="half">
            <div class="form-group">
              <label>Sort By</label>
              <select 
                class="select-input"
                value={$settings.fileDefaultSortField}
                onchange={handleDefaultSortFieldChange}
              >
                {#each sortFieldOptions as opt}
                  <option value={opt.value}>{opt.label}</option>
                {/each}
              </select>
            </div>
          </div>
          <div class="half">
            <div class="form-group">
              <label>Sort Direction</label>
              <select 
                class="select-input"
                value={$settings.fileDefaultSortDirection}
                onchange={handleDefaultSortDirectionChange}
              >
                {#each sortDirectionOptions as opt}
                  <option value={opt.value}>{opt.label}</option>
                {/each}
              </select>
            </div>
          </div>
        </div>

        <!-- Grid Icon Size Slider -->
        <div class="slider-group">
          <label class="slider-label">
            Grid Icon Size
            <span class="slider-value">{$settings.fileGridIconSize}px</span>
          </label>
          <input 
            type="range" 
            min="32" 
            max="128" 
            step="8"
            bind:value={$settings.fileGridIconSize}
            class="slider"
          />
          <div class="slider-range">
            <span>32px</span>
            <span>128px</span>
          </div>
        </div>

        <Checkbox label="Show Hidden Files (.*)" bind:checked={$settings.fileShowHidden} />
        <Checkbox label="Show File Extensions" bind:checked={$settings.fileShowExtensions} />
        
        <div class="spacer-sm"></div>

        <!-- Behavior Settings -->
        <h4 class="subsection-header">Behavior</h4>

        <Select 
          label="Double-click Action" 
          options={doubleClickOptions} 
          bind:value={$settings.fileDoubleClickAction}
        />

        <Checkbox label="Confirm before deleting" bind:checked={$settings.fileConfirmDelete} />

        <div class="spacer-sm"></div>

        <!-- Start Path Settings -->
        <h4 class="subsection-header">Start Location</h4>
        
        <Checkbox label="Remember last visited path" bind:checked={$settings.fileRememberLastPath} />
        
        {#if !$settings.fileRememberLastPath}
          <div class="indent-group">
            <Input 
              label="Default Start Path" 
              bind:value={$settings.fileDefaultStartPath}
              placeholder="C:\Users\YourName"
            />
          </div>
        {/if}

        <div class="spacer-sm"></div>

        <!-- Folder Size Settings -->
        <h4 class="subsection-header">Folder Size Calculation</h4>
        
        <Checkbox label="Show folder size" bind:checked={$settings.fileShowFolderSize} />
        
        {#if $settings.fileShowFolderSize}
          <div class="indent-group">
            <div class="slider-group">
              <label class="slider-label">
                Skip calculation if folder has more than
                <span class="slider-value">{$settings.fileFolderSizeThreshold.toLocaleString()} files</span>
              </label>
              <input 
                type="range" 
                min="100" 
                max="10000" 
                step="100"
                bind:value={$settings.fileFolderSizeThreshold}
                class="slider"
              />
              <div class="slider-range">
                <span>100</span>
                <span>10,000</span>
              </div>
            </div>
            <p class="hint">Large folders will show "—" instead of calculating size to avoid slowdowns.</p>
          </div>
        {/if}

        <div class="spacer-sm"></div>

        <!-- Thumbnails & Previews -->
        <h4 class="subsection-header">Thumbnails & Previews</h4>

        <div class="slider-group">
          <label class="slider-label">
            Thumbnail Size
            <span class="slider-value">{$settings.fileThumbnailSize}px</span>
          </label>
          <input 
            type="range" 
            min="32" 
            max="128" 
            step="8"
            bind:value={$settings.fileThumbnailSize}
            class="slider"
          />
          <div class="slider-range">
            <span>32px</span>
            <span>128px</span>
          </div>
        </div>

        <div class="row">
          <div class="half">
            <div class="form-group">
              <label>Max Concurrent Thumbnails</label>
              <input 
                type="number" 
                bind:value={$settings.fileMaxConcurrentThumbnails}
                min="1"
                max="20"
                class="number-input"
              />
            </div>
          </div>
          <div class="half">
            <div class="form-group">
              <label>Video Preview Resolution</label>
              <select bind:value={$settings.fileVideoPreviewResolution} class="select-input">
                <option value={120}>120p (Fast)</option>
                <option value={160}>160p (Default)</option>
                <option value={240}>240p (Quality)</option>
                <option value={320}>320p (High Quality)</option>
              </select>
            </div>
          </div>
        </div>

        <Checkbox label="Enable video hover preview" bind:checked={$settings.fileEnableVideoPreview} />
        
        {#if $settings.fileEnableVideoPreview}
          <div class="indent-group">
            <div class="slider-group">
              <label class="slider-label">
                Preview Duration
                <span class="slider-value">{$settings.fileVideoPreviewDuration}s</span>
              </label>
              <input 
                type="range" 
                min="1" 
                max="10" 
                step="1"
                bind:value={$settings.fileVideoPreviewDuration}
                class="slider"
              />
              <div class="slider-range">
                <span>1s</span>
                <span>10s</span>
              </div>
            </div>

            <Checkbox label="Use hardware acceleration" bind:checked={$settings.fileUseHardwareAccel} />
            <p class="hint">Enable GPU acceleration for faster video preview generation. Disable if you experience issues.</p>
          </div>
        {/if}

        <div class="spacer-sm"></div>

        <!-- Cache Management -->
        <h4 class="subsection-header">Cache Management</h4>

        <div class="slider-group">
          <label class="slider-label">
            Thumbnail Cache Size Limit
            <span class="slider-value">{$settings.fileThumbnailCacheSize} MB</span>
          </label>
          <input 
            type="range" 
            min="100" 
            max="2000" 
            step="100"
            bind:value={$settings.fileThumbnailCacheSize}
            class="slider"
          />
          <div class="slider-range">
            <span>100 MB</span>
            <span>2 GB</span>
          </div>
        </div>

        <div class="cache-actions">
          <div class="cache-info">
            <span class="cache-label">Current cache usage:</span>
            <span class="cache-value">{cacheSize}</span>
          </div>
          <button 
            class="btn-danger" 
            onclick={clearThumbnailCache}
            disabled={isClearingCache}
          >
            {isClearingCache ? 'Clearing...' : 'Clear Thumbnail Cache'}
          </button>
        </div>
      </section>

      <hr class="divider" />

      <!-- EDITOR -->
      <!-- EDITOR -->
      <section id="section-editor">
        <div class="section-header">
          <h3>Text Editor</h3>
          <p>Settings for code editing and viewing.</p>
        </div>

        <Input 
          label="Font Family" 
          bind:value={$settings.editorFontFamily} 
        />
        
        <div class="row">
          <div class="half">
            <Input 
              type="number" 
              label="Font Size (px)" 
              bind:value={$settings.editorFontSize} 
            />
          </div>
          <div class="half">
            <Select 
              label="Tab Size" 
              options={tabOptions} 
              bind:value={$settings.editorTabSize}
            />
          </div>
        </div>
        
        <Select 
          label="Word Wrap" 
          options={wrapOptions} 
          bind:value={$settings.editorWordWrap}
        />
        
        <Checkbox 
          label="Show Line Numbers" 
          bind:checked={$settings.editorShowLineNumbers} 
        />
        
        <Checkbox 
          label="Show Minimap" 
          bind:checked={$settings.editorShowMinimap} 
        />
        
        <Checkbox 
          label="Auto-Save" 
          bind:checked={$settings.editorAutoSave} 
        />
        
        {#if $settings.editorAutoSave}
          <div class="indent-group">
            <Input 
              type="number" 
              label="Auto-Save Interval (seconds)" 
              bind:value={$settings.editorAutoSaveInterval} 
            />
          </div>
        {/if}
      </section>
    </div>
  </div>

  <div slot="footer">
    <button class="btn-cancel" onclick={close}>Cancel</button>
    <button class="btn-save" onclick={close}>Save Changes</button>
  </div>
</Modal>

<style>
  .settings-layout { display: flex; margin: -20px; width: calc(100% + 40px); height: calc(100% + 40px); }
  .sidebar-col { width: 220px; background: var(--bg-panel); border-right: 1px solid var(--border); flex-shrink: 0; padding-top: 10px; }
  .content-col { flex: 1; padding: 40px; overflow-y: auto; scroll-behavior: smooth; position: relative; display: flex; flex-direction: column; gap: 10px; }
  .content-col::-webkit-scrollbar { width: 10px; }
  .content-col::-webkit-scrollbar-track { background: transparent; }
  .content-col::-webkit-scrollbar-thumb { background-color: var(--border); border-radius: 5px; border: 2px solid var(--bg-header); }
  .content-col::-webkit-scrollbar-thumb:hover { background-color: var(--text-muted); }
  section { margin-bottom: 20px; display: flex; flex-direction: column; gap: 15px; }
  
  .divider { border: 0; border-top: 1px solid var(--border); margin: 30px 0; opacity: 0.5; }
  
  .spacer-sm { height: 10px; }
  .spacer-lg { height: 300px; }
  
  .section-header { margin-bottom: 10px; }
  .section-header h3 { margin: 0 0 5px 0; font-size: 1.2rem; font-weight: 600; }
  .section-header p { margin: 0; color: var(--text-muted); font-size: 0.9rem; }
  
  .subsection-header { 
    margin: 10px 0 5px 0; 
    font-size: 0.85rem; 
    font-weight: 600; 
    color: var(--text-muted); 
    text-transform: uppercase; 
    letter-spacing: 0.5px;
  }
  
  .row { display: flex; gap: 20px; width: 100%; }
  .half { flex: 1; }
  
  /* Slider Styles */
  .slider-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .slider-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9rem;
    color: var(--text-main);
  }

  .slider-value {
    font-weight: 600;
    color: var(--border-focus);
    font-family: monospace;
  }

  .slider {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--bg-main);
    outline: none;
    -webkit-appearance: none;
    appearance: none;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--border-focus);
    cursor: pointer;
    border: 2px solid var(--bg-header);
    transition: transform 0.1s;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }

  .slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--border-focus);
    cursor: pointer;
    border: 2px solid var(--bg-header);
  }

  .slider-range {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .indent-group {
    margin-left: 24px;
    padding-left: 12px;
    border-left: 2px solid var(--border);
  }

  .hint {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 4px 0 0 0;
    font-style: italic;
  }
  
  button { padding: 8px 16px; border-radius: 4px; cursor: pointer; font-weight: 500; border: none; font-size: 0.9rem; }
  .btn-cancel { background: transparent; color: var(--text-muted); }
  .btn-cancel:hover { background: var(--hover-bg); color: var(--text-main); }
  .btn-save { background: var(--border-focus); color: white; }

  /* Inline form elements */
  .form-group { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  .form-group label { font-size: 0.85rem; font-weight: 500; color: var(--text-main); }
  
  .number-input, .select-input {
    background: var(--bg-main);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 10px;
    border-radius: 4px;
    outline: none;
    font-family: inherit;
    font-size: 0.9rem;
    transition: border-color 0.2s;
    width: 100%;
    box-sizing: border-box;
  }
  
  .number-input:focus, .select-input:focus { border-color: var(--border-focus); }
  .select-input { cursor: pointer; }

  /* Cache management */
  .cache-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--bg-main);
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .cache-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .cache-label {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .cache-value {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-main);
    font-family: monospace;
  }

  .btn-danger {
    background: #dc2626;
    color: white;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-danger:hover:not(:disabled) {
    background: #b91c1c;
  }

  .btn-danger:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>