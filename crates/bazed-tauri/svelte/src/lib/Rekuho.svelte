<script lang="ts">
  import { example as configExample } from "./config"
  import { Session, initSession } from "./rpc"
  import { state } from "./core"
  import Portion from "./Portion.svelte"
  import Welcome from "./Welcome.svelte"

  let session: Promise<Session> = initSession()
</script>

<div class="rekuho">
  {#await session}
    <Welcome>
      <div class="info">Establishing session...</div>
    </Welcome>
  {:then session}
    {#each Object.entries($state.views) as [id, viewState], _}
      <Portion
        config={configExample}
        lines={viewState.lines}
        firstLine={viewState.firstLine}
        carets={viewState.carets}
        on:keyinput={(event) => session.handleKeyPressed(id, event.detail)}
        on:mousedown={(event) => session.handleMouseClicked(id, event.detail)}
        on:mousewheel={(event) => session.handleMouseWheel(id, event.detail)}
        on:resize={(event) => session.handleUpdateView(id, event.detail)}
      />
    {/each}
  {:catch}
    <Welcome>
      <div class="info failure">Failed establishing session.</div>
    </Welcome>
  {/await}
</div>

<style lang="sass">
  .rekuho
    width: 100%
    height: 100%
    overflow: hidden

  .info
    font-size: 20px
    text-align: center

  .failure
    color: crimson
</style>
