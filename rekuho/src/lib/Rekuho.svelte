<script
  lang="ts"
  context="module"
>
</script>

<script lang="ts">
  import { example as exampleTheme } from "./theme"
  import { initSession, Session, type KeyInput } from "./rpc"
  import { state, type CaretPosition } from "./core"
  import Portion from "./Portion.svelte"

  export let width: number = 500
  export let height: number = 500

  let theme = exampleTheme

  let session: Session | null = null
  initSession().then((x) => {
    session = x
  })

  function onKeyInput(input: CustomEvent<KeyInput>) {
    if (!session) {
      return
    }
    session.handleKeyPressed(input.detail)
  }

  function onMouseDown(pos: CustomEvent<CaretPosition>) {
    if (!session) {
      return
    }
    session.handleMouseClicked(pos.detail)
  }
</script>

<div
  class="rekuho"
  style:width
>
  <Portion
    {theme}
    on:keyinput={onKeyInput}
    on:mousedown={onMouseDown}
    lines={$state.lines}
    {width}
    {height}
  />
</div>
