<script lang="ts">
  import "../app.css"; 
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme";
  import { settings } from "$lib/stores/settingsStore";

  onMount(() => {
    theme.subscribe((value) => {
      document.documentElement.setAttribute("data-theme", value);
    });

    settings.subscribe((s) => {
      if (s.globalFontFamily) {
        const fontStack = `'${s.globalFontFamily}', 'Segoe UI', sans-serif`;
        document.documentElement.style.setProperty('--font-global', fontStack);
      }
    });
  });
</script>

<slot />