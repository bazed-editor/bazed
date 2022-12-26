<!--
  Portion, may be simply the Editor
  This window contains the visible and editable text.
-->
<script lang="ts">
  import type { Theme } from "./theme"
  import { measure as fontMeasure } from "./font"
  import { state, type CaretPosition } from "./core"
  import type { Vector2 } from "./linearAlgebra"
  import type { Key, KeyInput, Modifier } from "./rpc"

  export let theme: Theme
  export let height: number
  export let width: number
  export let lines: string[]
  export let onKeyInput: (k: KeyInput) => void
  export let onMouseClicked: (pos: CaretPosition) => void

  const gutter_width = 50 // maybe should be part of theme, minimum value?
  let input: HTMLTextAreaElement
  let container: Element

  const emitKeyboardInput = (key: Key) => onKeyInput({ modifiers: [], key })

  $: view_rect = container && container.getBoundingClientRect()

  const pxToPortionPosition = ([x, y]: Vector2): Vector2 => {
    const div = (x: number, y: number): number => Math.floor(x / y)
    const column = div(x - view_rect.x, column_width)
    const line = div(y - view_rect.y, line_height)
    return [column, line]
  }

  const font = theme.font
  const font_metrics = fontMeasure(`${font.weight} ${font.size} ${font.family}`)
  const line_height: number = font_metrics.actualHeight || 0
  const column_width: number = font_metrics.width || 0

  ////////////////////////////////////////////////////////////////////////////////

  const mouseDown = (ev: MouseEvent) => {
    const [x, y] = pxToPortionPosition([ev.pageX, ev.pageY])
    onMouseClicked({ line: y, col: x })
    input.focus()
  }

  // We don't have mouse-based selection in the backend yet, :ree:

  /*
  const mousedown = (ev: MouseEvent) => {
    const current = pxToPortionPosition([ev.pageX, ev.pageY])
    selection = current
    cursorUpdate(0, (_) => current)
    input.focus()
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
      cursorUpdate(0, (_) => pxToPortionPosition([ev.pageX, ev.pageY]))
    }
  }
  */

  const keydown = (ev: KeyboardEvent) => {
    console.log(ev)
    const modifiers: Modifier[] = []
    if (ev.ctrlKey) modifiers.push("ctrl")
    if (ev.shiftKey) modifiers.push("shift")
    if (ev.altKey) modifiers.push("alt")
    if (ev.metaKey) modifiers.push("win")

    let key: Key | null = null
    if (ev.key.length === 1) {
      key = { char: ev.key }
    }

    switch (ev.key) {
      case "Enter":
        key = "return"
        break
      case "Backspace":
        key = "backspace"
        break
      case "ArrowLeft":
        key = "left"
        break
      case "ArrowRight":
        key = "right"
        break
      case "ArrowUp":
        key = "up"
        break
      case "ArrowDown":
        key = "down"
        break
    }

    if (key) {
      onKeyInput({ modifiers, key })
    }
  }

  const gutter_mousedown = (line: number, _ev: MouseEvent) => {
    onMouseClicked({ col: 0, line })
  }

  ////////////////////////////////////////////////////////////////////////////////

  const transformToScreenPosition = ([x, y]: Vector2): Vector2 => [
    x * column_width,
    y * line_height,
  ]

  const longestLine = (text: string[]): string =>
    text.length === 0 ? "" : text.reduce((a, b) => (a.length < b.length ? b : a))

  let linesHeight: number
  let linesWidth: number

  $: linesHeight = lines.length * line_height
  $: linesWidth = lines ? longestLine(lines).length : 1

  let line_view_height: number
  let line_view_width: number

  $: text_view_width = width - gutter_width
  $: text_to_visible_ratio = (line_view_width * column_width - theme.text_offset) / width
  $: vertical_scroller_width = text_view_width / text_to_visible_ratio

  let cursors: CaretPosition[] = []
  cursors = $state.carets
</script>

<div
  class="view"
  style:width="{width}px"
  style:height="{height}px"
>
  <!--<GutterColumn line_height={line_height} />-->
  <!-- TODO: refactor into `Gutter.svelte` -->
  <div
    class="gutter"
    style:background={theme.gutter.background}
    style:width="{gutter_width}px"
    style:height="{height}px"
  >
    {#each lines as _, i}
      <div
        class="gutter-cell"
        on:mousedown|preventDefault={(e) => {
          gutter_mousedown(i, e)
        }}
        style:font-size={theme.font.size}
        style:height="{line_height}px"
        style:top="{i * line_height}px"
      >
        {i + 1}
      </div>
    {/each}
  </div>

  <!-- don't know if this thing ought to exist at all -->
  <div
    class="text-offset-background"
    style:position="absolute"
    style:height="{height}px"
    style:top="0"
    style:background={theme.editor_background}
    style:width="{theme.text_offset}px"
    style:left="{gutter_width}px"
  />

  <div
    bind:this={container}
    class="container"
    on:mousedown|preventDefault={mouseDown}
    on:mousemove={() => {}}
    on:mouseup={() => {}}
    style:background={theme.editor_background}
    style:left="{gutter_width + theme.text_offset}px"
  >
    <textarea
      bind:this={input}
      tabindex="-1"
      wrap="off"
      on:keydown|preventDefault={keydown}
      style:user-select="text"
      style:position="absolute"
      style:width="{column_width}px"
    />
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
  </div>

  <!-- TODO: Implement scrollbars -->
  <!-- <Scrollbar /> -->
  <!-- <Scrollbar /> -->
  <div
    class="scrollbar vertical"
    style:height="{theme.scrollbar_width}px"
    style:width="{width - gutter_width}px"
    style:left="{gutter_width}px"
    style:top="{height - theme.scrollbar_width}px"
  >
    <div
      class="scroller"
      on:mousedown|preventDefault={(_) => {}}
      style:height="{theme.scrollbar_width}px"
      style:width="{vertical_scroller_width}px"
      style:background="#FFFFFF"
    />
  </div>
</div>

<style>
  .view {
    position: relative;
    overflow: hidden;
  }

  .scrollbar {
    position: absolute;
  }

  .scroller {
    position: absolute;
  }

  .gutter-cell {
    width: 50px;
    font-family: monospace;
    position: absolute;
    text-align: right;
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
