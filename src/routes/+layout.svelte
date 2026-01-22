<script lang="ts">
  import "../app.css"; 
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme";

  onMount(() => {
    theme.subscribe((value) => {
      document.documentElement.setAttribute("data-theme", value);
    });
  });
</script>

<slot />
