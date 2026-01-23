<script lang="ts">
  import { currentView } from '$lib/stores/viewStore';

  // Layout Components
  import TitleBar from '../components/layout/TitleBar.svelte';
  import NavBar from '../components/layout/NavBar.svelte';
  import SettingsModal from '../components/layout/SettingsModal.svelte'; // NEW IMPORT
  
  // Views
  import TerminalView from '../components/terminal/TerminalView.svelte';
  import FileTree from '../components/filesystem/FileTree.svelte';
  import FileManager from '../components/filesystem/FileManager.svelte';
  import EditorView from '../components/editor/EditorView.svelte';
  import ApiTester from '../components/apitester/ApiTester.svelte';
</script>

<SettingsModal />

<div class="app-container">
  
  <div class="header">
    <TitleBar />
  </div>

  <div class="body-row">
    
    <div class="nav-panel">
      <NavBar />
    </div>

    <div class="main-viewport">

      <div class="view-layer terminal-layout" style:display={$currentView === 'terminal' ? 'flex' : 'none'}>
        <div class="terminal-area">
          <TerminalView />
        </div>

        <div class="sidebar-panel">
           <FileTree />
        </div>
      </div>

      <div class="view-layer" style:display={$currentView === 'files' ? 'block' : 'none'}>
         <FileManager />
      </div>

      <div class="view-layer" style:display={$currentView === 'editor' ? 'block' : 'none'}>
         <EditorView />
      </div>

      <div class="view-layer" style:display={$currentView === 'api' ? 'block' : 'none'}>
         <ApiTester />
      </div>

    </div>

  </div>
</div>

<style>
  /* Keep your existing styles exactly as they were in */
  :global(body) { 
    margin: 0;
    padding: 0; 
    background: var(--bg-main);
    color: var(--text-main);
    overflow: hidden; 
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .header { flex: 0 0 auto; z-index: 20; }

  .body-row {
    flex: 1;
    display: flex;
    flex-direction: row;
    min-height: 0;
  }

  .nav-panel { flex: 0 0 auto; }

  .main-viewport {
    flex: 1;
    position: relative;
    background: var(--bg-main);
    overflow: hidden;
  }

  .view-layer {
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
  }

  .terminal-layout { flex-direction: row; }
  .terminal-area {
    flex: 1; min-width: 0; height: 100%; position: relative;
  }

  .sidebar-panel {
    flex: 0 0 auto;
    height: 100%;
    border-left: 1px solid var(--border);
    background: var(--bg-panel);
    overflow: hidden;
  }
</style>