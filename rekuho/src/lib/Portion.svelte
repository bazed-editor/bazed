<!--
    Portion, may be simply the Editor
    This window contains the visible and editable text.
-->
<script lang="ts">
  import type { Theme } from "./Theme"
  import LinesView, { lines } from "./LinesView.svelte"
  import type { Cursor, Position } from "./Cursors.svelte"
  import CursorsLayer, { cursors } from "./Cursors.svelte"

  export let theme: Theme

  export let height: number
  export let width: number

  let view: Element
  let input: HTMLTextAreaElement
  let container: Element

  // debug shit
  cursors.update((_) => [{ pos: [2, 1] }])
  lines.update((_) => [
    ...new Array(10).fill(""),
    ..."funky\nbanana\nt0wn".split("\n"),
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ...new Array(20).fill("a"),
  ])

  // let portion_start_line: number = 0

  const getCharacterWidth = (font: string): number | null => {
    const canvas = new OffscreenCanvas(0, 0)
    const context = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D | null
    if (context) {
      context.font = font
    }
    // REMARK: Elkowar mentioned: "with a font of **ZERO WIDTH X-CHARACTERS**" this breaks.
    // It does. ture. (typo intended)
    return context?.measureText("X").width || null
  }

  const updateCursor = (id: number, pos: Position): void => {
    cursors.update((cursors) => {
      cursors[0] = { pos }
      return cursors
    })
  }

  // TODO:
  $: view_rect = container && container.getBoundingClientRect()
  const pxToPortionPosition = ([x, y]: Position): Position => {
    const div = (x: number, y: number): number => Math.floor(x / y)
    const column = div(x - view_rect.x, column_width)
    const line = div(y - view_rect.y, line_height)
    return [column, line]
  }

  const font = theme.font
  const line_height: number = 18
  const column_width: number = getCharacterWidth(`${font.weight} ${font.size} ${font.family}`) || 1

  const mousedown = (ev: MouseEvent) => {
    updateCursor(0, pxToPortionPosition([ev.pageX, ev.pageY]))
    input.focus()
    ev.preventDefault()
  }

  const mouseup = (ev: MouseEvent) => {}
  const mousemove = (ev: MouseEvent) => {}

  const keydown = (ev: KeyboardEvent) => {
    ev.preventDefault() // stops the textarea from filling up with input
    switch (ev.key) {
      case "k":
        cursors.update((cursors) => {
          cursors[0] = { pos: [cursors[0].pos[0], cursors[0].pos[1] - 1] }
          return cursors
        })
        break
      case "j":
        cursors.update((cursors) => {
          cursors[0] = { pos: [cursors[0].pos[0], cursors[0].pos[1] + 1] }
          return cursors
        })
        break
      case "h":
        cursors.update((cursors) => {
          cursors[0] = { pos: [cursors[0].pos[0] - 1, cursors[0].pos[1]] }
          return cursors
        })
        break
      case "l":
        cursors.update((cursors) => {
          cursors[0] = { pos: [cursors[0].pos[0] + 1, cursors[0].pos[1]] }
          return cursors
        })
        break
    }
  }
  const gutter_width = 50
</script>

<div class="view" bind:this={view} style:width="{width}px" style:height="{height}px">
  <!--<GutterColumn line_height={line_height} />-->
  <div
    class="gutter"
    style:background={theme.gutter_background}
    style:width="{gutter_width}px"
    style:height="{height}px">
    {#each $lines as _, i}
      <div class="gutter-cell" style:height="{line_height}px" style:top="{i * line_height}px">
        {i}
      </div>
    {/each}
  </div>
  <div
    bind:this={container}
    class="container"
    on:mousedown={mousedown}
    on:mousemove={mousemove}
    on:mouseup={mouseup}
    style:background={theme.editor_background}
    style:left="{gutter_width}px">
    <textarea
      bind:this={input}
      wrap="off"
      tabindex="-1"
      on:keydown={keydown}
      style:user-select="text"
      style:position="absolute"
      style:width="{column_width}px" />
    <LinesView bind:theme {line_height} />
    <CursorsLayer bind:theme {column_width} {line_height} />
  </div>
  <!-- <Scrollbar /> -->
  <!-- <Scrollbar /> -->
</div>

<style>
  .gutter-cell {
    width: 50px;
    font-family: monospace;
    position: absolute;
    text-align: right;
  }

  .view {
    position: relative;
    overflow: hidden;
  }

  .container {
    position: absolute;
    top: 0;
    width: 1000000px;
    height: 1000000px;
  }

  textarea {
    opacity: 0;
    padding: 0;
    border: 0;
    margin: 0;

    width: 0;
    height: 0;
  }
</style>