<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "xterm";
  import { FitAddon } from "xterm-addon-fit";
  import "xterm/css/xterm.css";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { theme } from "$lib/stores/theme";
  import { currentPath } from "$lib/stores/path"; 
  import { activeTabId } from '$lib/stores/terminalStore';

  export let terminalId: string;
  export let profile: string = "pwsh";

  let terminalContainer: HTMLDivElement;
  let term: Terminal;
  let fitAddon: FitAddon;
  let unlisten: UnlistenFn;
  let unsubscribeTheme: () => void;
  let resizeObserver: ResizeObserver;

  const pathRegex = /PS\s+([A-Z]:\\[^>]+)>/;
   
  const themeColors = {
    dark: { background: '#1e1e1e', foreground: '#cccccc', cursor: '#ffffff', cursorAccent: '#000000', selectionBackground: 'rgba(255, 255, 255, 0.3)' },
    light: { background: '#ffffff', foreground: '#333333', cursor: '#000000', cursorAccent: '#ffffff', selectionBackground: 'rgba(0, 0, 0, 0.3)' }
  };

  async function fitAndResize() {
    if (!fitAddon || !term || !terminalContainer || terminalContainer.clientHeight === 0) return;

    try {
      fitAddon.fit();
      const { rows, cols } = term;
      await invoke("resize_terminal", { id: terminalId, rows, cols });
    } catch (err) {
      console.error("Failed to resize PTY:", err);
    }
  }

  $: if ($activeTabId === terminalId && term && fitAddon) {
    setTimeout(() => {
      fitAndResize();
      term.focus();
    }, 50);
  }

  function detectPathChange(text: string) {
    const match = text.match(pathRegex);
    if (match && match[1]) {
      currentPath.set(match[1].trim());
    }
  }

  onMount(async () => {
    term = new Terminal({
      cursorBlink: true,
      fontFamily: '"Fira Code", monospace',
      fontSize: 14,
      theme: themeColors[$theme as 'dark' | 'light'], 
      allowTransparency: true,
      macOptionIsMeta: true,
      scrollback: 1000,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.attachCustomKeyEventHandler((event) => {
          if (event.ctrlKey && event.shiftKey && event.code === 'KeyC') {
            if (event.type === 'keydown') {
              const selection = term.getSelection();
              if (selection) {
                navigator.clipboard.writeText(selection);
              }
              event.preventDefault(); 
            }
            return false;
          }
          if (event.ctrlKey && event.shiftKey && event.code === 'KeyV') {
            if (event.type === 'keydown') {
              navigator.clipboard.readText().then(text => {
                term.paste(text);
              });
              event.preventDefault();
            }
            return false;
          }
          if (event.code === 'Tab') {
            event.preventDefault(); 
            return true;
          }
          return true;
        });

    if (terminalContainer) {
      term.open(terminalContainer);
      setTimeout(() => fitAndResize(), 100);
    }

    try {
      await invoke("spawn_terminal", { id: terminalId, profile: profile });
    } catch (err) {
      console.error("Failed to spawn terminal:", err);
      term.write(`\r\n\x1b[31mError: ${err}\x1b[0m\r\n`);
    }

    resizeObserver = new ResizeObserver(() => requestAnimationFrame(() => fitAndResize()));
    if (terminalContainer?.parentElement) resizeObserver.observe(terminalContainer.parentElement);

    unlisten = await listen<string>(`terminal-output-${terminalId}`, (event) => {
      term.write(event.payload);
      detectPathChange(event.payload);
    });

    term.onData((data) => {
      invoke("write_to_terminal", { id: terminalId, data });
    });

    unsubscribeTheme = theme.subscribe((newTheme) => {
      if (term) term.options.theme = themeColors[newTheme as 'dark' | 'light'];
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (unsubscribeTheme) unsubscribeTheme();
    if (resizeObserver) resizeObserver.disconnect();
    if (term) term.dispose();
  });
</script>

<div class="terminal-wrapper">
  <div class="terminal-container" bind:this={terminalContainer}></div>
</div>

<style>
  .terminal-wrapper { height: 100%; width: 100%; padding: 6px 0 60px 8px; box-sizing: border-box; overflow: hidden; background-color: var(--bg-primary); }
  .terminal-container { height: 100%; width: 100%; overflow: hidden; }

  :global(.xterm-screen) { padding: 0 !important; margin: 0 !important; }
  :global(.xterm-viewport) { overflow-y: auto !important; }
  :global(.xterm-viewport::-webkit-scrollbar) { width: 8px; }
  :global(.xterm-viewport::-webkit-scrollbar-track) { background: transparent; }
  :global(.xterm-viewport::-webkit-scrollbar-thumb) {
    background-color: var(--border);
    border-radius: 4px;
    border: 2px solid var(--bg-primary);
  }
  :global(.xterm-viewport::-webkit-scrollbar-thumb:hover) { background-color: #888; }
</style>