<script
  lang="ts"
  context="module"
>
  import type { Writable } from "svelte/store"
  import { writable } from "svelte/store"

  import type { Vector2 } from "./LinearAlgebra"
  import { merge as vectorMerge } from "./LinearAlgebra"

  export type Position = Vector2
  export type Cursor = { pos: Vector2 }
  export let cursors: Writable<Cursor[]> = writable([{ id: 0, pos: [0, 0] }])

  export const cursorUpdate = (id: number, pos: [number | null, number | null]): void => {
    cursors.update((cursors) => {
      cursors[0] = { pos: vectorMerge(cursors[0].pos, pos) }
      return cursors
    })
  }

  export const cursorMove = (id: number, movement: Vector2): void => {
    cursors.update((cursors) => {
      cursors[0] = { pos: [cursors[id].pos[0] + movement[0], cursors[id].pos[1] + movement[1]] }
      return cursors
    })
  }
</script>

<script lang="ts">
  import type { Theme } from "./Theme"

  export let theme: Theme
  export let column_width: number
  export let line_height: number

  const visibility: string = "inherit"

  export const transformToScreenPosition = ([x, y]: Position): Position => [
    x * column_width,
    y * line_height,
  ]
</script>

<div class="cursors-layer">
  {#each $cursors as { pos }, i}
    {@const [x, y] = transformToScreenPosition(pos)}
    <div
      class="cursor"
      id="cursor-{i}"
      style:visibility
      style:width="{column_width}px"
      style:height="{line_height}px"
      style:background={theme.primary_cursor_color}
      style:left="{x}px"
      style:top="{y}px"
    />
  {/each}
</div>

<style>
  .cursors-layer {
    position: absolute;
    top: 0;
  }

  .cursor {
    position: absolute;
  }
</style>
