<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { fade, scale } from "svelte/transition";

  export let open: boolean = false;
  export let filePath: string = '';
  export let fileName: string = '';
  export let isDir: boolean = false;
  export let onclose: (() => void) | undefined = undefined;

  interface FileInfo {
    name: string;
    path: string;
    is_dir: boolean;
    size: number | null;
    created: number | null;
    modified: number | null;
    accessed: number | null;
    readonly: boolean;
    hidden: boolean;
    item_count: number | null;
  }

  let fileInfo: FileInfo | null = null;
  let isLoading = true;
  let error: string | null = null;

  $: if (open && filePath) {
    loadFileInfo();
  }

  async function loadFileInfo() {
    isLoading = true;
    error = null;
    try {
      fileInfo = await invoke<FileInfo>('get_file_info', { path: filePath });
    } catch (err) {
      error = String(err);
    } finally {
      isLoading = false;
    }
  }

  function close() {
    onclose?.();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (open && event.key === 'Escape') close();
  }

  function formatSize(bytes: number | null | undefined): string {
    if (bytes === null || bytes === undefined) return '—';
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const value = parseFloat((bytes / Math.pow(k, i)).toFixed(2));
    return `${value} ${sizes[i]} (${bytes.toLocaleString()} bytes)`;
  }

  function formatDate(timestamp: number | null | undefined): string {
    if (!timestamp) return '—';
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(undefined, {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  function getIcon(name: string, is_dir: boolean): string {
    if (is_dir) return '📁';
    const ext = name.split('.').pop()?.toLowerCase() || '';
    const iconMap: Record<string, string> = {
      'txt': '📄', 'md': '📝', 'pdf': '📕',
      'doc': '📘', 'docx': '📘', 'xls': '📗', 'xlsx': '📗',
      'jpg': '🖼️', 'jpeg': '🖼️', 'png': '🖼️', 'gif': '🖼️', 'webp': '🖼️', 'svg': '🖼️',
      'mp4': '🎬', 'mkv': '🎬', 'avi': '🎬', 'mov': '🎬', 'webm': '🎬',
      'mp3': '🎵', 'wav': '🎵', 'flac': '🎵', 'ogg': '🎵',
      'zip': '📦', 'rar': '📦', '7z': '📦',
      'js': '📜', 'ts': '📜', 'py': '🐍', 'rs': '🦀', 'go': '🔵',
      'html': '🌐', 'css': '🎨', 'json': '📋', 'xml': '📋', 'yaml': '📋',
      'exe': '⚙️', 'msi': '⚙️', 'dll': '⚙️',
    };
    return iconMap[ext] || '📄';
  }

  function getFileType(name: string, is_dir: boolean): string {
    if (is_dir) return 'Folder';
    const ext = name.split('.').pop()?.toLowerCase() || '';
    const typeMap: Record<string, string> = {
      'txt': 'Text Document', 'md': 'Markdown', 'pdf': 'PDF Document',
      'doc': 'Word Document', 'docx': 'Word Document',
      'xls': 'Excel Spreadsheet', 'xlsx': 'Excel Spreadsheet',
      'jpg': 'JPEG Image', 'jpeg': 'JPEG Image', 'png': 'PNG Image',
      'gif': 'GIF Image', 'webp': 'WebP Image', 'svg': 'SVG Image',
      'mp4': 'MP4 Video', 'mkv': 'MKV Video', 'avi': 'AVI Video',
      'mov': 'MOV Video', 'webm': 'WebM Video',
      'mp3': 'MP3 Audio', 'wav': 'WAV Audio', 'flac': 'FLAC Audio', 'ogg': 'OGG Audio',
      'zip': 'ZIP Archive', 'rar': 'RAR Archive', '7z': '7z Archive',
      'js': 'JavaScript', 'ts': 'TypeScript', 'py': 'Python',
      'rs': 'Rust', 'go': 'Go',
      'html': 'HTML', 'css': 'CSS', 'json': 'JSON', 'xml': 'XML', 'yaml': 'YAML',
      'exe': 'Executable', 'msi': 'Installer', 'dll': 'DLL',
    };
    return typeMap[ext] || `${ext.toUpperCase()} File`;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <div
    class="backdrop"
    on:click={close}
    transition:fade={{ duration: 150 }}
  >
    <div
      class="modal"
      on:click|stopPropagation
      transition:scale={{ duration: 150, start: 0.95 }}
    >
      <header>
        <h2>Properties</h2>
        <button class="close-btn" on:click={close}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </header>

      <div class="content">
        {#if isLoading}
          <div class="loading">Loading details...</div>
        {:else if error}
          <div class="error">Failed to load: {error}</div>
        {:else if fileInfo}
          <div class="file-header">
            <span class="file-icon">{getIcon(fileInfo.name, fileInfo.is_dir)}</span>
            <div class="file-title">
              <span class="file-name">{fileInfo.name}</span>
              <span class="file-type">{getFileType(fileInfo.name, fileInfo.is_dir)}</span>
            </div>
          </div>

          <div class="separator"></div>

          <div class="details-grid">
            <div class="detail-row">
              <span class="detail-label">Location</span>
              <span class="detail-value path" title={fileInfo.path}>{fileInfo.path}</span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Size</span>
              <span class="detail-value">{formatSize(fileInfo.size)}</span>
            </div>

            {#if fileInfo.is_dir && fileInfo.item_count !== null}
              <div class="detail-row">
                <span class="detail-label">Contains</span>
                <span class="detail-value">{fileInfo.item_count.toLocaleString()} item{fileInfo.item_count !== 1 ? 's' : ''}</span>
              </div>
            {/if}

            <div class="separator"></div>

            <div class="detail-row">
              <span class="detail-label">Created</span>
              <span class="detail-value">{formatDate(fileInfo.created)}</span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Modified</span>
              <span class="detail-value">{formatDate(fileInfo.modified)}</span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Accessed</span>
              <span class="detail-value">{formatDate(fileInfo.accessed)}</span>
            </div>

            <div class="separator"></div>

            <div class="detail-row">
              <span class="detail-label">Attributes</span>
              <span class="detail-value attributes">
                {#if fileInfo.readonly}
                  <span class="badge">Read-only</span>
                {/if}
                {#if fileInfo.hidden}
                  <span class="badge">Hidden</span>
                {/if}
                {#if !fileInfo.readonly && !fileInfo.hidden}
                  <span class="badge neutral">Normal</span>
                {/if}
              </span>
            </div>
          </div>
        {/if}
      </div>

      <footer>
        <button class="btn-close" on:click={close}>Close</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 9999;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .modal {
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    width: 420px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: var(--text-main);
  }

  header {
    padding: 14px 18px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
  }

  .close-btn:hover {
    background: var(--hover-bg);
    color: var(--text-main);
  }

  .content {
    padding: 18px;
    overflow-y: auto;
    flex: 1;
  }

  .loading {
    text-align: center;
    color: var(--text-muted);
    padding: 30px 0;
    font-size: 13px;
  }

  .error {
    text-align: center;
    color: #ef4444;
    padding: 30px 0;
    font-size: 13px;
  }

  .file-header {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 4px;
  }

  .file-icon {
    font-size: 36px;
    line-height: 1;
  }

  .file-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .file-name {
    font-size: 15px;
    font-weight: 600;
    word-break: break-all;
  }

  .file-type {
    font-size: 12px;
    color: var(--text-muted);
  }

  .separator {
    height: 1px;
    background: var(--border);
    margin: 14px 0;
  }

  .details-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .detail-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .detail-label {
    width: 80px;
    flex-shrink: 0;
    font-size: 12px;
    color: var(--text-muted);
    font-weight: 500;
    padding-top: 1px;
  }

  .detail-value {
    flex: 1;
    font-size: 13px;
    color: var(--text-main);
    word-break: break-all;
    min-width: 0;
  }

  .detail-value.path {
    font-size: 12px;
    color: var(--text-muted);
    font-family: monospace;
  }

  .attributes {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }

  .badge.neutral {
    background: rgba(100, 100, 100, 0.15);
    color: var(--text-muted);
  }

  footer {
    padding: 12px 18px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    flex-shrink: 0;
  }

  .btn-close {
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    border: none;
    font-size: 13px;
    background: var(--border-focus);
    color: white;
  }

  .btn-close:hover {
    opacity: 0.9;
  }
</style>