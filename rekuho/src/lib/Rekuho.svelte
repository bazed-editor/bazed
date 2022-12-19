<script
  lang="ts"
  context="module"
>
  const websocket = async () => {
    try {
      const ws = new WebSocket("ws://localhost:6969")
      const cmd = {
        method: "key_pressed",
        params: {
          key: "X",
          modifiers: [],
        },
      }

      await new Promise((resolve) => ws.addEventListener("open", () => resolve(null)))
      ws.send(JSON.stringify(cmd))
    } catch (e) {
      console.log(e)
    }
  }
</script>

<script lang="ts">
  import { example as exampleTheme } from "./Theme"
  import Portion from "./Portion.svelte"

  export let width: number = 500
  export let height: number = 500

  let theme = exampleTheme

  websocket()
</script>

<div
  class="rekuho"
  style:width
>
  <Portion
    bind:theme
    {width}
    {height}
  />
</div>
