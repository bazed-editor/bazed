<script lang="ts">
  import Portion from "./Portion.svelte"
  import kanagawa from "./kanagawa.js"
  import type { Theme } from "./Theme"

  export let width: number = 500
  export let height: number = 500

  export let theme: Theme = {
    font: {
      family: "monospace",
      weight: "normal",
      size: "20px",
    },

    gutter_background: kanagawa.sumiInk0,
    font_color: kanagawa.fujiWhite,
    primary_cursor_color: kanagawa.fujiWhite,
    editor_background: kanagawa.sumiInk1,
  }
</script>

<div class="rekuho" style:width>
  <Portion {theme} {width} {height} />
</div>