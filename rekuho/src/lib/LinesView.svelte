<!--
    LinesView contains lines of text
-->
<script lang="ts" context="module">
  import type { Writable } from "svelte/store"
  import { writable } from "svelte/store"

  export const lines: Writable<string[]> = writable([]) // contains all cached lines
</script>

<script lang="ts">
  import type { Theme } from "./Theme"
  export let theme: Theme
  export let line_height: number
</script>

<div class="lines-container">
  {#each $lines as line, i}
    <div
      class="line-container"
      style:top="{i * line_height}px"
      style:position="absolute"
      style:cursor="text"
      style:height="{line_height}px">
      <span
        class="line-view"
        style:color={theme.font_color}
        style:font-family={theme.font.family}
        style:font-size={theme.font.size}
        style:line-height="{line_height}px"
        style:white-space="pre-wrap">{line}</span>
    </div>
  {/each}
</div>