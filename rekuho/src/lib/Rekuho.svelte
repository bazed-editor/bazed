<script lang="ts">
  import { example as configExample } from "./config"
  import type { KeyInput, MouseWheel } from "./rpc"
  import { initSession, Session } from "./rpc"
  import type { CaretPosition } from "./core"
  import { state } from "./core"
  import Portion from "./Portion.svelte"

  let session: Promise<Session> = initSession()

  const onKeyInput = (session: Session, viewId: string, input: CustomEvent<KeyInput>) =>
    session.handleKeyPressed(viewId, input.detail)

  const onMouseDown = (session: Session, viewId: string, pos: CustomEvent<CaretPosition>) =>
    session.handleMouseClicked(viewId, pos.detail)

  const onMouseWheel = (session: Session, viewId: string, pos: CustomEvent<MouseWheel>) =>
    session.handleMouseWheel(viewId, pos.detail.delta)

  const onResize = (session: Session, viewId: string, pos: CustomEvent<[number, number]>) => {
    session.handleUpdateView(
      viewId,
      {
        width: pos.detail[0],
        height: pos.detail[1],
      }
    )
  }

  // FIXME: CURRENT SVELTE PREPROCESSING CONFIGURATION DOES NOT PREPROCESS ATTRIBUTE CODE
  const helperhelper = (session: Session, viewId: string) =>
    <T>(callback: (session: Session, viewId: string, event: CustomEvent<T>) => void) =>
    (event: CustomEvent<T>): void => callback(session, viewId, event)
</script>

<div class="rekuho">
  {#await session}
    <div>Establishing session...</div>
  {:then session}
    {#each Object.entries($state.views) as [id, viewState], _}
      {@const helper = helperhelper(session, id)}
      <Portion
        config={configExample}
        lines={viewState.lines}
        firstLine={viewState.firstLine}
        carets={viewState.carets}
        on:keyinput={helper(onKeyInput)}
        on:mousedown={helper(onMouseDown)}
        on:mousewheel={helper(onMouseWheel)}
        on:resize={helper(onResize)}
      />
    {/each}
  {/await}
</div>

<style>
  .rekuho {
    width: 100%;
    height: 100%;
  }
</style>
