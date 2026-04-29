<script lang="ts">
  import "../app.css";
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { theme } from "$lib/stores/theme";
  import { settings } from "$lib/stores/settingsStore";
  import { startNotificationScheduler, stopNotificationScheduler } from "$lib/services/notificationService";

  onMount(() => {
    theme.subscribe((value) => {
      document.documentElement.setAttribute("data-theme", value);
    });

    settings.subscribe((s) => {
      if (s.globalFontFamily) {
        const fontStack = `'${s.globalFontFamily}', 'Segoe UI', sans-serif`;
        document.documentElement.style.setProperty('--font-global', fontStack);
      }
      // Keep Rust-side close-to-tray flag in sync
      invoke('set_close_to_tray', { enabled: s.closeToTray }).catch(() => {});
    });

    startNotificationScheduler();
  });

  onDestroy(() => {
    stopNotificationScheduler();
  });
</script>

<slot />