<!--
  Portion, may be simply the Editor
  This window contains the visible and editable text.
-->
<script lang="ts">
  import { createEventDispatcher } from "svelte"

  import type { CaretPosition } from "./core"
  import { state } from "./core"

  import type { Config } from "./config"
  import { measureOnChild as fontMeasure, fontToString } from "./font"
  import type { Vector2 } from "./linearAlgebra"
  import type { KeyInput, MouseWheel } from "./rpc"
  import { keyboardToKeyInput, getModifiers } from "./event"

  type pixels = number
  type line = number

  export let config: Config

  let width: pixels
  let height: pixels

  const gutterWidth = 50 // TODO: maybe should be part of theme, minimum value?
  let input: HTMLTextAreaElement
  let container: HTMLElement

  const dispatch = createEventDispatcher<{
    resize: [pixels, pixels]
    keyinput: KeyInput
    mousedown: CaretPosition
    mousewheel: MouseWheel
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

  let lineHeight: pixels
  let columnWidth: pixels

  $: if (container) {
    const fontMetrics = fontMeasure(container, fontToString(config.font))
    lineHeight = fontMetrics.actualHeight ?? 0
    columnWidth = fontMetrics.width ?? 0
  }

  $: lineCount = Math.ceil(height / lineHeight)
  $: columnCount = Math.ceil(width / columnWidth)

  const transformToScreenPosition = ([x, y]: Vector2): Vector2 => [x * columnWidth, y * lineHeight]

  ////////////////////////////////////////////////////////////////////////////////

  const longestLine = (text: string[]): string =>
    text.reduce((a, b) => (a.length < b.length ? b : a), "")

  $: linesViewWidth = $state.lines ? longestLine($state.lines).length : 1
  $: textViewWidth = width - gutterWidth
  $: textWidthToVisibleRatio = (linesViewWidth * columnWidth - config.textOffset) / width
  $: horizontalScrollerWidth = textViewWidth / textWidthToVisibleRatio

  ////////////////////////////////////////////////////////////////////////////////

  let scrollOffset: pixels = 0

  // const scrollShowLine = (lineNumber: line, height: pixels): void => {
  //   const lineOffset = lineNumber * lineHeight
  //   if (lineOffset + lineHeight > scrollOffset + height) {
  //     scrollOffset = lineOffset - height + lineHeight
  //   } else if (scrollOffset > lineOffset) {
  //     scrollOffset = lineOffset
  //   }
  // }

  // $: scrollShowLine($state.carets[0]?.line ?? 0, height)

  $: scrollOffset = $state.first_line * lineHeight

  ////////////////////////////////////////////////////////////////////////////////

  const onMouseDown = (event: MouseEvent): boolean => {
    const [x, y] = pxToPortionPosition([event.pageX, event.pageY + scrollOffset])
    const handled = onMouseClicked({ line: y, col: x })
    input.focus()
    return handled
  }

  const onWheel = (event: WheelEvent): boolean => {
    const modifiers = getModifiers(event)
    const delta = event.deltaY
    switch (event.deltaMode) {
      case WheelEvent.DOM_DELTA_PIXEL:
        console.log(`{ delta: ${delta} }`)
        break
      case WheelEvent.DOM_DELTA_LINE:
        console.error("unhandled page wheel line mode")
        break
      case WheelEvent.DOM_DELTA_PAGE:
        console.error("unhandled page wheel delta mode")
        break
    }
    dispatch("mousewheel", { modifiers, delta: Math.round(delta / lineHeight) > 0 ? 1 : -1 })
    return true
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
    style:background={config.theme.gutterBg}
    style:top="-{scrollOffset}px"
    style:width="{gutterWidth}px"
    style:height="{height}px"
  >
    {#each $state.lines as _, _i}
      {@const i = $state.first_line + _i}
      <div
        class="gutter-cell"
        on:mousedown|preventDefault={(_) => onMouseClicked({ col: 0, line: i })}
        style:font-size={config.font.size}
        style:height="{lineHeight}px"
        style:top="{_i * lineHeight}px"
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
    style:background={config.theme.editorBg}
    style:width="{config.textOffset}px"
    style:left="{gutterWidth}px"
  />

  <div
    bind:this={container}
    class="container"
    on:mousedown|preventDefault={onMouseDown}
    on:wheel={onWheel}
    style:top="-{scrollOffset}px"
    style:left="{gutterWidth + config.textOffset}px"
    style:background={config.theme.editorBg}
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
          style:top="{($state.first_line + i) * lineHeight}px"
          style:height="{lineHeight}px"
        >
          <span
            class="line-view"
            style:font-family={config.font.family}
            style:font-size={config.font.size}
            style:color={config.theme.editorFg}
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
          style:background={config.theme.cursorColorPrimary}
          style:left="{x}px"
          style:top="{y}px"
        />
      {/each}
    </div>
  </div>

  <!-- TODO: Implement vertical scrollbar and scrolling -->
  <!--
  {#if textWidthToVisibleRatio > 1}
    <div
      class="scrollbar horizontal"
      style:height="{config.scrollbarWidth}px"
      style:width="{width - gutterWidth}px"
      style:left="{gutterWidth}px"
      style:top="{height - config.scrollbarWidth}px"
    >
      <div
        class="scroller"
        on:mousedown|preventDefault={(_) => {}}
        style:height="{config.scrollbarWidth}px"
        style:width="{horizontalScrollerWidth}px"
        style:background="#FFFFFF"
      />
    </div>
  {/if}
  -->
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
