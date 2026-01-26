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
      <button onclick={openFile} title="Open File">Open</button>
      <button onclick={saveFile} title="Save File">Save</button>
    </div>

    <div class="divider"></div>

    <div class="toolbar-section">
      <button onclick={toggleBold} title="Bold">B</button>
      <button onclick={toggleItalic} title="Italic">I</button>
      <button onclick={toggleCodeBlock} title="Code Block">{`</>`}</button>
    </div>

    <div class="divider"></div>

    <div class="toolbar-section">
      <button onclick={() => toggleHeading(1)} title="Heading 1">H1</button>
      <button onclick={() => toggleHeading(2)} title="Heading 2">H2</button>
      <button onclick={() => toggleHeading(3)} title="Heading 3">H3</button>
    </div>

    <div class="divider"></div>

    <div class="toolbar-section">
      <button onclick={toggleBulletList} title="Bullet List">•</button>
      <button onclick={toggleOrderedList} title="Ordered List">1.</button>
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
    padding-left: 1.5em;
  }

  :global(.editor li) {
    margin-bottom: 0.25em;
  }
</style>