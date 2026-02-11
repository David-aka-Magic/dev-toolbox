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
        document.body.style.fontFamily = `'${s.globalFontFamily}', 'Segoe UI', sans-serif`;
      }
    });
  });
</script>

<slot />