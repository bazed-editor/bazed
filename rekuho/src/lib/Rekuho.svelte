<script lang="ts">
    import Portion from "./Portion.svelte"
    import kanagawa from "./kanagawa.js"
    import type { Theme } from "./Theme"

    export let width: number = 500;
    export let height: number = 500;

    export let theme: Theme = {
        font: {
            family: "monospace",
            weight: "normal",
            size: "20px",
        },

        font_color: kanagawa.fujiWhite,
        primary_cursor_color: kanagawa.fujiWhite,
        editor_background: kanagawa.sumiInk1,
        line_height: 18,
    }
</script>

<div class="rekuho" style:width={width}>
  <Portion theme={theme} width={width} height={height} />
</div>