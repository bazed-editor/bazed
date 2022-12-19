<!--
    Portion, may be simply the Editor
    This window contains the visible and editable text.
-->

<script lang="ts">
  import type { Theme } from "./Theme"
  import LinesView, { lines } from "./LinesView.svelte"
  import type { Cursor, Position } from "./Cursors.svelte"
  import CursorsLayer, { cursors, cursorUpdate, cursorMove } from "./Cursors.svelte"
  import Gutter from "./Gutter.svelte"

  export let theme: Theme
  const gutter_width = 50 // maybe should be part of theme, minimum value?

  export let height: number
  export let width: number

  let view: Element
  let input: HTMLTextAreaElement
  let container: Element

  // TODO: Get proper input from backend
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
    // FIXME: Elkowar mentioned: "with a font of **ZERO WIDTH X-CHARACTERS**" this breaks.
    // It does. ture. (typo intended)
    return context?.measureText("X").width || null
  }

  // TODO: Separate into linear_algebra.ts
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

  // TODO: Implement proper selections
  let selection: Position | null = null

  const mousedown = (ev: MouseEvent) => {
    selection = pxToPortionPosition([ev.pageX, ev.pageY])
    cursorUpdate(0, selection)
    input.focus()
    ev.preventDefault()
  }

  const mouseup = (ev: MouseEvent) => {
    if (selection) {
      const begin = selection
      selection = null
      const end = pxToPortionPosition([ev.pageX, ev.pageY])
    }
  }

  const mousemove = (ev: MouseEvent) => {
    if (selection) {
      cursorUpdate(0, pxToPortionPosition([ev.pageX, ev.pageY]))
    }
  }
  // const dragstart = (ev: DragEvent) => {}
  // const drag = (ev : DragEvent) => {}

  const keydown = (ev: KeyboardEvent) => {
    ev.preventDefault() // stops the textarea from filling up with input
    switch (ev.key) {
      case "h":
        cursorMove(0, [-1, 0])
        break
      case "j":
        cursorMove(0, [0, 1])
        break
      case "k":
        cursorMove(0, [0, -1])
        break
      case "l":
        cursorMove(0, [1, 0])
        break
    }
    // TODO: Handle input from keydown events
  }

  const gutter_mousedown = (line: number, ev: MouseEvent) => {
    cursorUpdate(0, [null, line])
  }
</script>

<div
  class="view"
  bind:this={view}
  style:width="{width}px"
  style:height="{height}px">
  <!--<GutterColumn line_height={line_height} />-->
  <!-- TODO: Maybe place into ex. GutterColumn.svelte -->
  <div
    class="gutter"
    style:background={theme.gutter.background}
    style:width="{gutter_width}px"
    style:height="{height}px">
    {#each $lines as _, i}
      <div
        class="gutter-cell"
        on:mousedown={(e) => {
          gutter_mousedown(i, e)
        }}
        style:font-size={theme.font.size}
        style:height="{line_height}px"
        style:top="{i * line_height}px">
        {i + 1}
      </div>
    {/each}
  </div>
  <div
    style:position="absolute"
    style:height="{height}px"
    style:top="0"
    style:background={theme.editor_background}
    style:width="{theme.text_offset}px"
    style:left="{gutter_width}px" />
  <div
    bind:this={container}
    class="container"
    on:mousedown={mousedown}
    on:mousemove={mousemove}
    on:mouseup={mouseup}
    style:background={theme.editor_background}
    style:left="{gutter_width + theme.text_offset}px">
    <textarea
      bind:this={input}
      wrap="off"
      tabindex="-1"
      on:keydown={keydown}
      style:user-select="text"
      style:position="absolute"
      style:width="{column_width}px" />
    <LinesView
      bind:theme
      {line_height} />
    <CursorsLayer
      bind:theme
      {column_width}
      {line_height} />
  </div>

  <!-- TODO: Implement scrollbars -->
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
