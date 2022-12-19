<script
  lang="ts"
  context="module">
  import kanagawa from "./kanagawa.js"
  import type { Theme } from "./Theme"

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

  export let width: number = 500
  export let height: number = 500

  export let theme: Theme = {
    font: {
      family: "monospace",
      weight: "normal",
      size: "20px",
    },

    gutter_background: kanagawa.sumiInk0,
    font_color: kanagawa.fujiWhite,
    primary_cursor_color: kanagawa.fujiWhite,
    editor_background: kanagawa.sumiInk1,
  }
</script>

<div
  class="rekuho"
  style:width>
  <Portion
    bind:theme
    {width}
    {height} />
</div>
