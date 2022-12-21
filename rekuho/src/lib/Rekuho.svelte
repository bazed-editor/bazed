<script
  lang="ts"
  context="module"
>
</script>

<script lang="ts">
  import { example as exampleTheme } from "./Theme"
  import { initSession, Session, type KeyInput } from "./Rpc"

  import Portion from "./Portion.svelte"
  import { state, type CaretPosition } from "./Core"

  export let width: number = 500
  export let height: number = 500
  let theme = exampleTheme

  let session: Session | null = null
  initSession().then((x) => {
    session = x
  })

  function onKeyboardInput(input: KeyInput) {
    if (!session) {
      return
    }
    session.handleKeyPressed(input)
  }

  function onMouseClicked(pos: CaretPosition) {
    if (!session) {
      return
    }
    session.handleMouseClicked(pos)
  }
</script>

<div
  class="rekuho"
  style:width
>
  <Portion
    bind:theme
    onKeyInput={onKeyboardInput}
    {onMouseClicked}
    lines={$state.lines}
    {width}
    {height}
  />
</div>
