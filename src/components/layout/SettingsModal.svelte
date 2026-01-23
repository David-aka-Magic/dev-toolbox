<script lang="ts">
  import { tick } from 'svelte';
  
  // Stores
  import { isSettingsOpen, settings } from '$lib/stores/settingsStore';
  import { currentView } from '$lib/stores/viewStore';
  import { theme } from '$lib/stores/theme'; 

  // Components
  import Modal from '../ui/Modal.svelte';
  import SettingsSidebar from './SettingsSidebar.svelte';
  import Input from '../ui/forms/Input.svelte';
  import Select from '../ui/forms/Select.svelte';
  import Checkbox from '../ui/forms/Checkbox.svelte';

  let activeSection = 'general';
  let scrollContainer: HTMLElement;
  let isManualScroll = false; 

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

  const iconOptions = [
    { value: 'material', label: 'Material Icons' },
    { value: 'minimal', label: 'Minimal' },
    { value: 'none', label: 'None' }
  ];

  // --- LOGIC ---

  // 1. Context-Aware Open
  $: if ($isSettingsOpen) {
    let startSection = 'general';
    if ($currentView === 'editor') startSection = 'editor';
    else if ($currentView === 'terminal') startSection = 'terminal';
    else if ($currentView === 'files') startSection = 'files';
    
    activeSection = startSection;
    scrollToSection(startSection);
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
        bind:activeSection 
        onSelect={handleSidebarSelect} 
      />
    </div>

    <div 
      class="content-col" 
      bind:this={scrollContainer}
      on:scroll={handleScroll}
    >
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

      <section id="section-files">
        <div class="section-header">
          <h3>File Manager</h3>
          <p>Control how files and folders are displayed.</p>
        </div>
        
        <Checkbox label="Show Hidden Files (.*)" checked={true} />
        <Checkbox label="Confirm before deleting" checked={true} />
        
        <div class="spacer-sm"></div>

        <Select 
          label="File Icon Theme" 
          options={iconOptions} 
          value="material"
        />
      </section>

      <hr class="divider" />

      <section id="section-editor">
        <div class="section-header">
          <h3>Text Editor</h3>
          <p>Settings for code editing and viewing.</p>
        </div>

        <Input label="Font Family" value="'Fira Code', Consolas, monospace" />
        <div class="row">
            <div class="half">
                <Select label="Tab Size" options={tabOptions} value="2"/>
            </div>
            <div class="half">
                 <Select label="Word Wrap" options={wrapOptions} value="off"/>
            </div>
        </div>
        <Checkbox label="Show Line Numbers" checked={true} />
        <Checkbox label="Show Minimap" checked={true} />
      </section>

      <div class="spacer-lg"></div>
    </div>
  </div>

  <div slot="footer">
    <button class="btn-cancel" on:click={close}>Cancel</button>
    <button class="btn-save" on:click={close}>Save Changes</button>
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
  
  /* Divider Style */
  .divider { border: 0; border-top: 1px solid var(--border); margin: 30px 0; opacity: 0.5; }
  
  .spacer-sm { height: 10px; }
  .spacer-lg { height: 300px; }
  .section-header { margin-bottom: 10px; }
  .section-header h3 { margin: 0 0 5px 0; font-size: 1.2rem; font-weight: 600; }
  .section-header p { margin: 0; color: var(--text-muted); font-size: 0.9rem; }
  .row { display: flex; gap: 20px; width: 100%; }
  .half { flex: 1; }
  button { padding: 8px 16px; border-radius: 4px; cursor: pointer; font-weight: 500; border: none; font-size: 0.9rem; }
  .btn-cancel { background: transparent; color: var(--text-muted); }
  .btn-cancel:hover { background: var(--hover-bg); color: var(--text-main); }
  .btn-save { background: var(--border-focus); color: white; }
</style>