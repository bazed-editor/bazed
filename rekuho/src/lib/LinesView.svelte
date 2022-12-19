<!--
    LinesView contains lines of text
-->

<script lang="ts" context="module">
  import type { Writable } from "svelte/store"
  import { writable } from "svelte/store"

  import type { Vector } from "./LinearAlgebra"

  export const lines: Writable<string[]> = writable([]) // contains all cached lines

  export const isAlpha = (code: number): boolean =>
    (code > 47 && code < 58) || (code > 64 && code < 91) || (code > 96 && code < 123)

  const splice = (self: string, offset: number, text: string, removeCount: number = 0): string => {
    let calculatedOffset = offset < 0 ? self.length + offset : offset
    return (
      self.substring(0, calculatedOffset) + text + self.substring(calculatedOffset + removeCount)
    )
  }

  const modifyNth = <T>(list: T[], i: number, f: (_: T) => T): T[] => {
    list[i] = f(list[i])
    return list
  }

  export const insertAt = (text: string, [x, y]: Vector) =>
    lines.update((lines: string[]) => modifyNth(lines, y, (line) => splice(line, x, text)))
</script>

<script lang="ts">
  import type { Theme } from "./Theme"
  export let theme: Theme
  export let line_height: number
</script>

<div class="lines-container">
  {#each $lines as line, i}
    <div class="line-container" style:top="{i * line_height}px" style:height="{line_height}px">
      <span
        class="line-view"
        style:color={theme.font_color}
        style:height="{line_height}px"
        style:font-family={theme.font.family}
        style:font-size={theme.font.size}
        style:line-height="{line_height}px">
        {line}
      </span>
    </div>
  {/each}
</div>

<style>
    .line-container {
        width: 100%;
        position: absolute;
        cursor: text;
    }

  .line-view {
    white-space: pre;
  }
</style>
