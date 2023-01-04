<script lang="ts">
  import { example as configExample } from "./config"
  import type { KeyInput, MouseWheel } from "./rpc"
  import { initSession, Session } from "./rpc"
  import type { CaretPosition } from "./core"
  import Portion from "./Portion.svelte"

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

  const onMouseWheel = (pos: CustomEvent<MouseWheel>) => {
    if (!session) {
      return
    }
    session.handleMouseWheel(pos.detail.delta)
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
    config={configExample}
    on:keyinput={onKeyInput}
    on:mousedown={onMouseDown}
    on:mousewheel={onMouseWheel}
    on:resize={onResize}
  />
</div>

<style>
  .rekuho {
    width: 100%;
    height: 100%;
  }
</style>
