<script lang="ts">
  export let fileName: string = 'Untitled';
  export let saveStatus: 'saved' | 'saving' | 'unsaved' = 'saved';
  export let line: number = 1;
  export let column: number = 1;
  export let wordCount: number = 0;
  export let charCount: number = 0;
  
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
  
  $: language = detectLanguage(fileName);
  
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