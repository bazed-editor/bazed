<!--
    LinesView contains lines of text
-->

<script lang=ts context=module>
    import type { Writable } from "svelte/store"
    import { writable } from "svelte/store"

    export const lines: Writable<string[]> = writable([]) // contains all cached lines
</script>

<script lang=ts>
    import type { Theme } from "./Theme"
    export let theme: Theme

    export let width: number

    const line_height_px: number = theme.line_height;
</script>

{#each $lines as line, i}
                           <!-- style:background=dimgray -->
    <div class="line-container" style:top={i * line_height_px}px
                                style:position=absolute
                                style:cursor=text
                                style:width={width}px
                                style:height={line_height_px}px>
        <span class="line-view" style:color={theme.font_color}
                                style:font-family={theme.font.family}
                                style:line-height={line_height_px}px
                                style:white-space=pre-wrap>{line}</span>
    </div>
{/each}