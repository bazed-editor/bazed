<!--
  Portion, may be simply the Editor
  This window contains the visible and editable text.
-->
<script lang="ts">
  import { createEventDispatcher } from "svelte"

  import type { CaretPosition } from "./core"
  import { state } from "./core"

  import type { Theme } from "./theme"
  import { measureOnChild as fontMeasure, fontToString } from "./font"
  import type { Vector2 } from "./linearAlgebra"
  import type { KeyInput } from "./rpc"
  import { keyboardToKeyInput } from "./event"

  export let theme: Theme

  let width: number
  let height: number

  const gutterWidth = 50 // TODO: maybe should be part of theme, minimum value?
  let input: HTMLTextAreaElement
  let container: HTMLElement

  const dispatch = createEventDispatcher<{
    resize: [number, number]
    keyinput: KeyInput
    mousedown: CaretPosition
  }>()

  const emitKeyboardInput = (input: KeyInput) => dispatch("keyinput", input)
  const onMouseClicked = (pos: CaretPosition) => dispatch("mousedown", pos)

  ////////////////////////////////////////////////////////////////////////////////

  $: viewRect = container?.getBoundingClientRect()

  const pxToPortionPosition = ([x, y]: Vector2): Vector2 => {
    const div = (a: number, b: number): number => Math.floor(a / b)
    const column = div(x - viewRect.x, columnWidth)
    const line = div(y - viewRect.y, lineHeight)
    return [column, line]
  }

  type Pixels = number

  let lineHeight: Pixels
  let columnWidth: Pixels

  $: if (container) {
    const fontMetrics = fontMeasure(container, fontToString(theme.font))
    lineHeight = fontMetrics.actualHeight ?? 0
    columnWidth = fontMetrics.width ?? 0
  }

  $: lineCount = Math.ceil(height / lineHeight)
  $: columnCount = Math.ceil(width / columnWidth)

  const transformToScreenPosition = ([x, y]: Vector2): Vector2 => [x * columnWidth, y * lineHeight]

  const longestLine = (text: string[]): string =>
    text.reduce((a, b) => (a.length < b.length ? b : a), "")

  $: linesViewWidth = $state.lines ? longestLine($state.lines).length : 1
  $: textViewWidth = width - gutterWidth
  $: textToVisibleRatio = (linesViewWidth * columnWidth - theme.textOffset) / width
  $: verticalScrollerWidth = textViewWidth / textToVisibleRatio

  ////////////////////////////////////////////////////////////////////////////////

  const onMouseDown = (ev: MouseEvent) => {
    const [x, y] = pxToPortionPosition([ev.pageX, ev.pageY])
    onMouseClicked({ line: y, col: x })
    input.focus()
  }

  const onKeyDown = (domEvent: KeyboardEvent) => {
    console.log(domEvent)
    const event = keyboardToKeyInput(domEvent)
    if (event) {
      emitKeyboardInput(event)
    }
  }

  $: dispatch("resize", [columnCount, lineCount])
</script>

<div
  bind:clientWidth={width}
  bind:clientHeight={height}
  class="view"
>
  <!-- TODO: refactor into `Gutter.svelte` -->
  <div
    class="gutter"
    style:background={theme.gutter.background}
    style:width="{gutterWidth}px"
    style:height="{height}px"
  >
    {#each $state.lines as _, i}
      <div
        class="gutter-cell"
        on:mousedown|preventDefault={(_) => onMouseClicked({ col: 0, line: i })}
        style:font-size={theme.font.size}
        style:height="{lineHeight}px"
        style:top="{i * lineHeight}px"
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
    style:background={theme.editorBackground}
    style:width="{theme.textOffset}px"
    style:left="{gutterWidth}px"
  />

  <div
    bind:this={container}
    class="container"
    on:mousedown|preventDefault={onMouseDown}
    on:mousemove={() => {}}
    on:mouseup={() => {}}
    style:background={theme.editorBackground}
    style:left="{gutterWidth + theme.textOffset}px"
  >
    <textarea
      bind:this={input}
      tabindex="-1"
      wrap="off"
      on:keydown|preventDefault={onKeyDown}
      style:user-select="text"
      style:position="absolute"
      style:width="{columnWidth}px"
    />
    <div class="lines-container">
      {#each $state.lines as line, i}
        <div
          class="line-container"
          style:top="{i * lineHeight}px"
          style:height="{lineHeight}px"
        >
          <span
            class="line-view"
            style:font-family={theme.font.family}
            style:font-size={theme.font.size}
            style:color={theme.textColor}
            style:height="{lineHeight}px"
            style:line-height="{lineHeight}px"
          >
            {line}
          </span>
        </div>
      {/each}
    </div>

    <div class="cursors-layer">
      {#each $state.carets as { line, col }, i}
        {@const [x, y] = transformToScreenPosition([col, line])}
        <div
          class="cursor"
          id="cursor-{i}"
          style:visibility="inherit"
          style:width="{columnWidth}px"
          style:height="{lineHeight}px"
          style:background={theme.primaryCursorColor}
          style:left="{x}px"
          style:top="{y}px"
        />
      {/each}
    </div>
  </div>

  <!-- TODO: Implement vertical scrollbar and scrolling -->
  <!-- <VerticalScrollbar /> -->
  {#if textToVisibleRatio > 1}
    <div
      class="scrollbar vertical"
      style:height="{theme.scrollbarWidth}px"
      style:width="{width - gutterWidth}px"
      style:left="{gutterWidth}px"
      style:top="{height - theme.scrollbarWidth}px"
    >
      <div
        class="scroller"
        on:mousedown|preventDefault={(_) => {}}
        style:height="{theme.scrollbarWidth}px"
        style:width="{verticalScrollerWidth}px"
        style:background="#FFFFFF"
      />
    </div>
  {/if}
</div>

<style>
  .view {
    position: relative;
    overflow: hidden;
    width: 100%;
    height: 100%;
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

  .cursors-layer {
    position: absolute;
    top: 0;
  }

  .cursor {
    position: absolute;
  }
</style>
