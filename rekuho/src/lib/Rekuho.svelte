<script lang="ts">
  import { example as exampleTheme } from "./theme"
  import type { KeyInput } from "./rpc"
  import { initSession, Session } from "./rpc"
  import type { CaretPosition } from "./core"
  import Portion from "./Portion.svelte"

  export let width: number = 50
  export let height: number = 500

  let theme = exampleTheme

  let session: Session | null = null
  initSession().then((x) => {
    session = x
  })

  const onKeyInput = (input: CustomEvent<KeyInput>) => {
    if (!session) {
      return
    }
    session.handleKeyPressed(input.detail)
  }

  const onMouseDown = (pos: CustomEvent<CaretPosition>) => {
    if (!session) {
      return
    }
    session.handleMouseClicked(pos.detail)
  }
</script>

<div class="rekuho">
  <Portion
    {theme}
    {width}
    {height}
    on:keyinput={onKeyInput}
    on:mousedown={onMouseDown}
  />
</div>
