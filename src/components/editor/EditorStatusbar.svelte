<script lang="ts">
  import { Editor } from '@tiptap/core';
  
  export let editor: Editor | null = null;
  export let fileName: string = 'Untitled';
  export let saveStatus: 'saved' | 'saving' | 'unsaved' = 'saved';
  export let filePath: string = '';
  
  let line = 1;
  let column = 1;
  let wordCount = 0;
  let charCount = 0;
  let language = 'plaintext';
  
  $: if (editor) {
    updateStats();
  }
  
  function updateStats() {
    if (!editor) return;
    
    const { from } = editor.state.selection;
    const text = editor.getText();
    const beforeCursor = text.substring(0, from);
    
    line = (beforeCursor.match(/\n/g) || []).length + 1;
    column = beforeCursor.length - beforeCursor.lastIndexOf('\n');
    
    wordCount = text.split(/\s+/).filter(w => w.length > 0).length;
    charCount = text.length;
    
    language = detectLanguage(fileName);
  }
  
  function detectLanguage(name: string): string {
    const ext = name.split('.').pop()?.toLowerCase();
    const langMap: Record<string, string> = {
      'js': 'JavaScript',
      'ts': 'TypeScript',
      'py': 'Python',
      'rs': 'Rust',
      'html': 'HTML',
      'css': 'CSS',
      'json': 'JSON',
      'md': 'Markdown',
      'txt': 'Text',
      'svelte': 'Svelte'
    };
    return langMap[ext || ''] || 'Text';
  }
  
  $: statusColor = saveStatus === 'saved' ? 'var(--text-muted)' : 
                   saveStatus === 'saving' ? '#f59e0b' : 
                   '#ef4444';
  
  $: statusText = saveStatus === 'saved' ? 'Saved' :
                  saveStatus === 'saving' ? 'Saving...' :
                  'Unsaved';
</script>

<div class="status-bar">
  <div class="status-section">
    <span class="status-item" title="Line and Column">
      Ln {line}, Col {column}
    </span>
    <span class="separator">|</span>
    <span class="status-item" title="Language">
      {language}
    </span>
  </div>
  
  <div class="status-section">
    <span class="status-item" title="Word and Character Count">
      {wordCount} words, {charCount} chars
    </span>
    <span class="separator">|</span>
    <span class="status-item save-status" style="color: {statusColor}" title="Save Status">
      {statusText}
    </span>
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 24px;
    background: var(--bg-panel);
    border-top: 1px solid var(--border);
    padding: 0 12px;
    font-size: 11px;
    color: var(--text-muted);
    user-select: none;
  }
  
  .status-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .status-item {
    cursor: default;
  }
  
  .separator {
    opacity: 0.3;
  }
  
  .save-status {
    font-weight: 500;
  }
</style>