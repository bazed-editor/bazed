<script lang="ts">
  import { example as exampleTheme } from "./theme"
  import type { KeyInput } from "./rpc"
  import { initSession, Session } from "./rpc"
  import type { CaretPosition } from "./core"
  import Portion from "./Portion.svelte"

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

  const onResize = (pos: CustomEvent<[number, number]>) => {
    if (!session) {
      return
    }
    session.handleUpdateView({
      first_line: 0,
      first_col: 0,
      width: pos.detail[0],
      height: pos.detail[1],
    })
  }
</script>

<div class="rekuho">
  <Portion
    {theme}
    on:keyinput={onKeyInput}
    on:mousedown={onMouseDown}
    on:resize={onResize}
  />
</div>

<style>
  .rekuho {
    width: 100%;
    height: 100%;
  }
</style>
