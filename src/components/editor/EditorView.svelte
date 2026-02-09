<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight';
  import { common, createLowlight } from 'lowlight';
  import javascript from 'highlight.js/lib/languages/javascript';
  import typescript from 'highlight.js/lib/languages/typescript';
  import python from 'highlight.js/lib/languages/python';
  import rust from 'highlight.js/lib/languages/rust';
  import css from 'highlight.js/lib/languages/css';
  import html from 'highlight.js/lib/languages/xml';
  import json from 'highlight.js/lib/languages/json';
  import 'highlight.js/styles/vs2015.css';
  import { invoke } from '@tauri-apps/api/core';
  import { editorTabs, activeEditorTabId } from '$lib/stores/editorStore';
  import { settings } from '$lib/stores/settingsStore';
  import { recentFiles } from '$lib/stores/recentFileStore';
  import FilePickerModal from './FilePickerModal.svelte';
  import EditorSidebar from './EditorSideBar.svelte';
  import EditorStatusBar from './EditorStatusBar.svelte';
  import CommandPalette from './CommandPalette.svelte';

  const lowlight = createLowlight(common);
  
  lowlight.register('javascript', javascript);
  lowlight.register('typescript', typescript);
  lowlight.register('python', python);
  lowlight.register('rust', rust);
  lowlight.register('css', css);
  lowlight.register('html', html);
  lowlight.register('json', json);

  let element: HTMLDivElement;
  let editor: Editor;
  let showOpenDialog = false;
  let showSaveDialog = false;
  let showCommandPalette = false;
  let showSidebar = true;
  let saveStatus: 'saved' | 'saving' | 'unsaved' = 'saved';
  let autoSaveInterval: number;
  let currentLine = 1;
  
  $: activeTab = $editorTabs.find(t => t.id === $activeEditorTabId);

  onMount(() => {
    editor = new Editor({
      element: element,
      extensions: [
        StarterKit.configure({
          codeBlock: false,
        }),
        CodeBlockLowlight.configure({
          lowlight,
          defaultLanguage: 'plaintext',
        }),
      ],
      content: activeTab?.content || '',
      editorProps: {
        attributes: {
          class: 'editor-content',
        },
        handleDOMEvents: {
          dblclick: (view, event) => {
            selectWord(event);
            return true;
          },
          keydown: (view, event) => {
            if (event.key === 'Tab') {
              event.preventDefault();
              const tabSize = Number($settings.editorTabSize) || 4;
              const tabString = ' '.repeat(tabSize);
              editor.commands.insertContent(tabString);
              return true;
            }
            if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
              return false;
            }
            return false;
          }
        }
      },
      onUpdate: ({ editor }) => {
        if (activeTab) {
          const content = editor.getHTML();
          editorTabs.update(tabs => 
            tabs.map(tab => 
              tab.id === $activeEditorTabId
                ? { ...tab, content, isDirty: true }
                : tab
            )
          );
          saveStatus = 'unsaved';
        }
        updateCurrentLine();
      },
      onSelectionUpdate: () => {
        updateCurrentLine();
      }
    });
    
    autoSaveInterval = window.setInterval(() => {
      if (activeTab?.isDirty && $settings.editorAutoSave) {
        autoSave();
      }
    }, $settings.editorAutoSaveInterval * 1000);
    
    document.addEventListener('keydown', handleGlobalKeydown);
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
    if (autoSaveInterval) {
      clearInterval(autoSaveInterval);
    }
    document.removeEventListener('keydown', handleGlobalKeydown);
  });

  $: if (editor && activeTab) {
    const currentContent = editor.getHTML();
    if (currentContent !== activeTab.content) {
      editor.commands.setContent(activeTab.content);
      saveStatus = activeTab.isDirty ? 'unsaved' : 'saved';
    }
  }
  
  function updateCurrentLine() {
    if (!editor) return;
    const { from } = editor.state.selection;
    const text = editor.getText();
    const beforeCursor = text.substring(0, from);
    currentLine = (beforeCursor.match(/\n/g) || []).length + 1;
  }
  
  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'k') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
      return;
    }
    
    if (e.ctrlKey && e.key === 'b') {
      e.preventDefault();
      showSidebar = !showSidebar;
      return;
    }
    
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      saveFile();
      return;
    }
    
    if (e.ctrlKey && e.key === 'o') {
      e.preventDefault();
      openFile();
      return;
    }
  }
  
  function handleCommand(event: CustomEvent) {
    const { action } = event.detail;
    
    switch(action) {
      case 'save':
        saveFile();
        break;
      case 'open':
        openFile();
        break;
      case 'toggleSidebar':
        showSidebar = !showSidebar;
        break;
    }
  }
  
  function selectWord(event: MouseEvent) {
    if (!editor) return;
    const pos = editor.view.posAtCoords({ left: event.clientX, top: event.clientY });
    if (!pos) return;
    
    const { doc } = editor.state;
    const text = doc.textBetween(0, doc.content.size);
    
    let start = pos.pos;
    let end = pos.pos;
    
    while (start > 0 && /\w/.test(text[start - 1])) start--;
    while (end < text.length && /\w/.test(text[end])) end++;
    
    if (start < end) {
      editor.commands.setTextSelection({ from: start, to: end });
    }
  }

  function toggleBold() { editor?.chain().focus().toggleBold().run(); }
  function toggleItalic() { editor?.chain().focus().toggleItalic().run(); }
  function toggleCodeBlock() { editor?.chain().focus().toggleCodeBlock().run(); }
  function toggleHeading(level: 1 | 2 | 3) { 
    editor?.chain().focus().toggleHeading({ level }).run(); 
  }
  function toggleBulletList() { editor?.chain().focus().toggleBulletList().run(); }
  function toggleOrderedList() { editor?.chain().focus().toggleOrderedList().run(); }

  function openFile() {
    showOpenDialog = true;
  }

  async function handleFileSelect(event: CustomEvent<string>) {
    showOpenDialog = false;
    const filePath = event.detail;

    try {
      const content = await invoke<string>('read_file', { path: filePath });
      
      const fileName = filePath.split(/[\\/]/).pop() || 'Untitled';
      const newTab = {
        id: crypto.randomUUID(),
        name: fileName,
        path: filePath,
        content: `<p>${content.replace(/\n/g, '</p><p>')}</p>`,
        isDirty: false
      };
      
      editorTabs.update(tabs => [...tabs, newTab]);
      activeEditorTabId.set(newTab.id);
      
      recentFiles.addFile(filePath, fileName);
      saveStatus = 'saved';
    } catch (err) {
      alert(`Failed to open file: ${err}`);
    }
  }
  
  async function handleRecentFileOpen(event: CustomEvent) {
    const { path, name } = event.detail;
    
    try {
      const content = await invoke<string>('read_file', { path });
      
      const newTab = {
        id: crypto.randomUUID(),
        name,
        path,
        content: `<p>${content.replace(/\n/g, '</p><p>')}</p>`,
        isDirty: false
      };
      
      editorTabs.update(tabs => [...tabs, newTab]);
      activeEditorTabId.set(newTab.id);
      recentFiles.addFile(path, name);
      saveStatus = 'saved';
    } catch (err) {
      alert(`Failed to open file: ${err}`);
    }
  }

  function saveFile() {
    if (!activeTab) return;

    if (activeTab.path) {
      saveToPath(activeTab.path);
    } else {
      showSaveDialog = true;
    }
  }
  
  async function autoSave() {
    if (!activeTab || !activeTab.path) return;
    saveStatus = 'saving';
    
    try {
      const plainText = editor?.getText() || '';
      await invoke('write_file', { path: activeTab.path, content: plainText });
      
      editorTabs.update(tabs => 
        tabs.map(tab => 
          tab.id === $activeEditorTabId
            ? { ...tab, isDirty: false }
            : tab
        )
      );
      saveStatus = 'saved';
    } catch (err) {
      console.error('Auto-save failed:', err);
      saveStatus = 'unsaved';
    }
  }

  async function handleSaveSelect(event: CustomEvent<string>) {
    showSaveDialog = false;
    await saveToPath(event.detail);
  }

  async function saveToPath(filePath: string) {
    if (!activeTab) return;
    
    saveStatus = 'saving';

    try {
      const plainText = editor?.getText() || '';
      await invoke('write_file', { path: filePath, content: plainText });
      
      const fileName = filePath.split(/[\\/]/).pop() || 'Untitled';
      editorTabs.update(tabs => 
        tabs.map(tab => 
          tab.id === $activeEditorTabId
            ? { ...tab, name: fileName, path: filePath, isDirty: false }
            : tab
        )
      );
      
      recentFiles.addFile(filePath, fileName);
      saveStatus = 'saved';
    } catch (err) {
      alert(`Failed to save file: ${err}`);
      saveStatus = 'unsaved';
    }
  }
</script>

<div class="editor-layout">
  <div class="editor-container">
    <div class="toolbar">
      <div class="toolbar-section">
        <button onclick={openFile} title="Open File (Ctrl+O)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
          </svg>
        </button>
        <button onclick={saveFile} title="Save File (Ctrl+S)" class:active={activeTab?.isDirty}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z"/>
            <polyline points="17 21 17 13 7 13 7 21"/>
            <polyline points="7 3 7 8 15 8"/>
          </svg>
        </button>
        <button onclick={() => showSidebar = !showSidebar} title="Toggle Sidebar (Ctrl+B)" class:active={showSidebar}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
            <line x1="9" y1="3" x2="9" y2="21"/>
          </svg>
        </button>
      </div>

      <div class="divider"></div>

      <div class="toolbar-section">
        <button onclick={toggleBold} class:active={editor?.isActive('bold')} title="Bold (Ctrl+B)">
          <strong>B</strong>
        </button>
        <button onclick={toggleItalic} class:active={editor?.isActive('italic')} title="Italic (Ctrl+I)">
          <em>I</em>
        </button>
        <button onclick={() => toggleHeading(1)} class:active={editor?.isActive('heading', { level: 1 })} title="Heading 1">
          H1
        </button>
        <button onclick={() => toggleHeading(2)} class:active={editor?.isActive('heading', { level: 2 })} title="Heading 2">
          H2
        </button>
        <button onclick={toggleCodeBlock} class:active={editor?.isActive('codeBlock')} title="Code Block">
          {`</>`}
        </button>
      </div>

      <div class="divider"></div>

      <div class="toolbar-section">
        <button onclick={toggleBulletList} class:active={editor?.isActive('bulletList')} title="Bullet List">•</button>
        <button onclick={toggleOrderedList} class:active={editor?.isActive('orderedList')} title="Numbered List">1.</button>
      </div>
      
      <div class="divider"></div>
      
      <div class="toolbar-section">
        <button onclick={() => showCommandPalette = true} title="Command Palette (Ctrl+K)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="editor-wrapper">
      <div 
        class="editor" 
        bind:this={element}
        style="
          font-family: {$settings.editorFontFamily};
          font-size: {$settings.editorFontSize}px;
          white-space: {$settings.editorWordWrap === 'on' ? 'pre-wrap' : 'pre'};
        "
        class:hide-line-numbers={!$settings.editorShowLineNumbers}
      ></div>
    </div>
    
    <EditorStatusBar 
      {editor} 
      fileName={activeTab?.name || 'Untitled'}
      filePath={activeTab?.path || ''}
      {saveStatus}
    />
  </div>
  
  {#if showSidebar}
    <EditorSidebar on:openfile={handleRecentFileOpen} />
  {/if}
</div>

<CommandPalette bind:open={showCommandPalette} on:command={handleCommand} />

{#if showOpenDialog}
  <FilePickerModal 
    mode="open"
    on:select={handleFileSelect}
    on:cancel={() => showOpenDialog = false}
  />
{/if}

{#if showSaveDialog}
  <FilePickerModal 
    mode="save"
    on:select={handleSaveSelect}
    on:cancel={() => showSaveDialog = false}
  />
{/if}

<style>
  .editor-layout {
    display: flex;
    height: 100%;
    background: var(--bg-main);
  }
  
  .editor-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    height: 100%;
    background: var(--bg-main);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .toolbar-section {
    display: flex;
    gap: 2px;
  }

  .divider {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 4px;
  }

  .toolbar button {
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    transition: all 0.2s;
  }

  .toolbar button:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .toolbar button.active {
    background: var(--selection);
    color: var(--border-focus);
  }

  .editor-wrapper {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-main);
    position: relative;
    display: flex;
  }

  :global(.editor) {
    height: 100%;
    width: 100%;
    color: var(--text-main);
    counter-reset: line;
  }

  :global(.editor .ProseMirror) {
    min-height: 100%;
    outline: none;
    padding: 16px 16px 16px 60px;
    position: relative;
    line-height: 1.4;
  }

  :global(.editor.hide-line-numbers .ProseMirror) {
    padding-left: 16px;
  }

  :global(.editor .ProseMirror > *) {
    position: relative;
    counter-increment: line;
    margin: 0;
    padding: 0;
  }

  :global(.editor:not(.hide-line-numbers) .ProseMirror > *::before) {
    content: counter(line);
    position: absolute;
    left: -50px;
    width: 40px;
    text-align: right;
    color: var(--text-muted);
    font-size: 0.9em;
    user-select: none;
    opacity: 0.5;
  }

  :global(.editor .ProseMirror > *) {
    margin-bottom: 0;
    line-height: 1.4;
  }

  :global(.editor p) {
    margin: 0;
    padding: 2px 0;
  }

  :global(.editor h1) {
    font-size: 2em;
    font-weight: bold;
    margin-top: 0.5em;
  }

  :global(.editor h2) {
    font-size: 1.5em;
    font-weight: bold;
    margin-top: 0.5em;
  }

  :global(.editor h3) {
    font-size: 1.25em;
    font-weight: bold;
  }

  :global(.editor code) {
    background: var(--bg-panel);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: 'Fira Code', monospace;
    font-size: 0.9em;
  }

  :global(.editor pre) {
    background: #1e1e1e !important;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 16px;
    overflow-x: auto;
    font-family: 'Fira Code', monospace;
    font-size: 0.9em;
    line-height: 1.5;
    counter-reset: none;
    margin: 8px 0;
  }

  :global(.editor pre code) {
    background: none !important;
    padding: 0;
  }

  :global(.editor ul, .editor ol) {
    padding-left: 2.5em;
    margin: 0;
    margin-left: 0.5em;
  }

  :global(.editor li) {
    margin-bottom: 0.25em;
  }

  :global(.editor li p) {
    margin: 0;
    display: inline;
  }

  :global(.editor ul ul, .editor ol ol, .editor ul ol, .editor ol ul) {
    margin-top: 0.25em;
  }

  :global(.editor .ProseMirror:focus) {
    outline: none;
  }

  :global(.editor .ProseMirror) {
    cursor: text;
  }
</style>