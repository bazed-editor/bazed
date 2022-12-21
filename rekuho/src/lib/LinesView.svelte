<!--
    LinesView contains lines of text
-->
<script
  lang="ts"
  context="module"
>
  const longestLine = (text: string[]): string =>
    text.length === 0 ? "" : text.reduce((a, b) => (a.length < b.length ? b : a))
</script>

<script lang="ts">
  import type { Theme } from "./Theme"

  export let theme: Theme
  export let line_height: number

  export let height: number
  export let width: number

  export let lines: string[] // contains all cached lines

  $: height = lines.length * line_height
  $: width = lines ? longestLine(lines).length : 1
</script>

<div class="lines-container">
  {#each lines as line, i}
    <div
      class="line-container"
      style:top="{i * line_height}px"
      style:height="{line_height}px"
    >
      <span
        class="line-view"
        style:font-family={theme.font.family}
        style:font-size={theme.font.size}
        style:color={theme.text_color}
        style:height="{line_height}px"
        style:line-height="{line_height}px"
      >
        {line}
      </span>
    </div>
  {/each}
</div>

<style>
  .lines-container {
    position: absolute;
  }

  .line-container {
    position: absolute;
    width: 100%;
    cursor: text;
  }

  .line-view {
    white-space: pre;
  }
</style>
