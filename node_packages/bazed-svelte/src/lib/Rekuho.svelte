<!--
  @component
  # Rekuho

  Antipode of monaco.

  Component displaying an editor.

    @param {Config} config - front-end configuration
    @param {string[]} lines - lines to display of the buffer, starting from firstLine
    @param {number} firstLine - offset at which to render the current buffer
    @param {CaretPosition[]} carets - cursor positions

    @fires Resize#resize - the editor has been resized by more than a line, fill the view with lines
    @fires KeyInput#keyinput - a key was pressed
    @fires CaretPosition#mousedown - click position, by line and column
    @fires MouseWheel#mousewheel - scrolled up or down
-->

<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte"

  import type { CaretPosition } from "./core"
  import type { Config } from "./config"
  import { measureOnChild as fontMeasure, fontToString } from "./font"
  import type { Vector2 } from "./linearAlgebra"
  import type { KeyInput, MouseWheel } from "./rpc"
  import { getModifiers, keyboardToKeyInput, wheelDelta } from "./event"

  type pixels = number
  type line = number

  export let config: Config
  export let lines: string[]
  export let firstLine: number
  export let carets: CaretPosition[]

  let width: pixels
  let height: pixels

  const gutterWidth = 50 // TODO: maybe should be part of theme, minimum value?
  let input: HTMLTextAreaElement
  let container: HTMLElement

  type Resize = { width: number; height: number }

  type Events = {
    resize: Resize
    keyinput: KeyInput
    mousedown: CaretPosition
    mousewheel: MouseWheel
  }

  const dispatch = createEventDispatcher<Events>()

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

  $: linesViewWidth = lines ? longestLine(lines).length : 1
  $: textViewWidth = width - gutterWidth
  $: textWidthToVisibleRatio = (linesViewWidth * columnWidth - config.textOffset) / width
  $: horizontalScrollerWidth = textViewWidth / textWidthToVisibleRatio

  let scrollOffset: number = 0
  $: scrollOffset = firstLine * lineHeight

  ////////////////////////////////////////////////////////////////////////////////

  const onMouseDown = (event: MouseEvent): boolean => {
    const [x, y] = pxToPortionPosition([event.pageX, event.pageY + scrollOffset])
    const handled = onMouseClicked({ line: y, col: x })
    input.focus()
    return handled
  }

  const onWheel = (event: WheelEvent): boolean => {
    const modifiers = getModifiers(event)
    const delta = wheelDelta(event)
    dispatch("mousewheel", { modifiers, delta })
    return true
  }

  const onKeyDown = (domEvent: KeyboardEvent) => {
    const event = keyboardToKeyInput(domEvent)
    if (event) {
      emitKeyboardInput(event)
    }
  }

  $: dispatch("resize", { width: columnCount, height: lineCount })
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
    {#each lines as _, _i}
      {@const i = firstLine + _i}
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
      {#each lines as line, i}
        <div
          class="line-container"
          style:top="{(firstLine + i) * lineHeight}px"
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
      {#each carets as { line, col }, i}
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

<style lang="sass">
  .view
    position: relative
    overflow: hidden
    width: 100%
    height: 100%

  // .scrollbar
  //   position: absolute

  // .scroller
  //   position: absolute

  .gutter-cell
    width: 50px
    font-family: monospace
    position: absolute
    text-align: right

  .container
    position: absolute
    width: 1000000px
    height: 1000000px

  textarea
    opacity: 0
    padding: 0
    border: 0
    margin: 0

    width: 0
    height: 0

  .lines-container
    position: absolute

  .line-container
    position: absolute
    width: 100%
    cursor: text

  .line-view
    white-space: pre

  .cursors-layer
    position: absolute
    top: 0

  .cursor
    position: absolute
</style>
