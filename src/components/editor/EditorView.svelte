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
  import FilePickerModal from './FilePickerModal.svelte';

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
        }
      },
    });
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
  });

  $: if (editor && activeTab) {
    const currentContent = editor.getHTML();
    if (currentContent !== activeTab.content) {
      editor.commands.setContent(activeTab.content);
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

  async function handleSaveSelect(event: CustomEvent<string>) {
    showSaveDialog = false;
    await saveToPath(event.detail);
  }

  async function saveToPath(filePath: string) {
    if (!activeTab) return;

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
      alert('File saved!');
    } catch (err) {
      alert(`Failed to save file: ${err}`);
    }
  }
</script>

<div class="editor-container">
  <div class="toolbar">
    <div class="toolbar-section">
      <button on:click={openFile} title="Open File">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
        </svg>
      </button>
      <button on:click={saveFile} title="Save File" class:active={activeTab?.isDirty}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z"/>
          <polyline points="17 21 17 13 7 13 7 21"/>
          <polyline points="7 3 7 8 15 8"/>
        </svg>
      </button>
    </div>

    <div class="divider"></div>

    <div class="toolbar-section">
      <button on:click={toggleBold} class:active={editor?.isActive('bold')} title="Bold">
        <strong>B</strong>
      </button>
      <button on:click={toggleItalic} class:active={editor?.isActive('italic')} title="Italic">
        <em>I</em>
      </button>
      <button on:click={() => toggleHeading(1)} class:active={editor?.isActive('heading', { level: 1 })} title="Heading 1">
        H1
      </button>
      <button on:click={() => toggleHeading(2)} class:active={editor?.isActive('heading', { level: 2 })} title="Heading 2">
        H2
      </button>
      <button on:click={toggleCodeBlock} class:active={editor?.isActive('codeBlock')} title="Code Block">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="16 18 22 12 16 6"/>
          <polyline points="8 6 2 12 8 18"/>
        </svg>
      </button>
    </div>

    <div class="divider"></div>

    <div class="toolbar-section">
      <button on:click={toggleBulletList} class:active={editor?.isActive('bulletList')} title="Bullet List">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"/>
          <line x1="8" y1="12" x2="21" y2="12"/>
          <line x1="8" y1="18" x2="21" y2="18"/>
          <line x1="3" y1="6" x2="3.01" y2="6"/>
          <line x1="3" y1="12" x2="3.01" y2="12"/>
          <line x1="3" y1="18" x2="3.01" y2="18"/>
        </svg>
      </button>
      <button on:click={toggleOrderedList} class:active={editor?.isActive('orderedList')} title="Numbered List">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="10" y1="6" x2="21" y2="6"/>
          <line x1="10" y1="12" x2="21" y2="12"/>
          <line x1="10" y1="18" x2="21" y2="18"/>
          <path d="M4 6h1v4"/>
          <path d="M4 10h2"/>
          <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="editor-wrapper">
    <div bind:this={element} class="editor"></div>
  </div>
</div>

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
  .editor-container {
    display: flex;
    flex-direction: column;
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

  :global(.editor .ProseMirror > *) {
    position: relative;
    counter-increment: line;
    margin: 0;
    padding: 0;
  }

  :global(.editor .ProseMirror > *::before) {
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
    padding-left: 1.5em;
  }

  :global(.editor li) {
    margin-bottom: 0.25em;
  }
</style>