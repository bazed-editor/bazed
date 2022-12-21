<script
  lang="ts"
  context="module"
>
  import type { Vector2 } from "./LinearAlgebra"
</script>

<script lang="ts">
  import type { CaretPosition } from "./Core"

  import type { Theme } from "./Theme"

  export let cursors: CaretPosition[]

  export let theme: Theme
  export let column_width: number
  export let line_height: number

  const transformToScreenPosition = ([x, y]: Vector2): Vector2 => [
    x * column_width,
    y * line_height,
  ]
</script>

<div class="cursors-layer">
  {#each cursors as { line, col }, i}
    {@const [x, y] = transformToScreenPosition([col, line])}
    <div
      class="cursor"
      id="cursor-{i}"
      style:visibility="inherit"
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
