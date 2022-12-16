<script lang="ts" context="module">
  import type { Writable } from "svelte/store"
  import { writable } from "svelte/store"

  export type Position = [number, number]
  export type Cursor = { pos: [number, number] }

  export let cursors: Writable<Cursor[]> = writable([])
</script>

<script lang="ts">
  import type { Theme } from "./Theme"

  export let theme: Theme
  export let column_width: number
  export let line_height: number

  const visibility: string = "inherit"
</script>

<div class="cursors-layer" style:position="absolute" style:top="0">
  {#each $cursors as { pos: [x, y] }, i}
    <div style:visibility style:position="absolute" style:width="5px" style:height="{line_height}px"
         style:background={theme.primary_cursor_color}
         style:left="{x * column_width}px" style:top="{y * line_height}px" />
  {/each}
</div>