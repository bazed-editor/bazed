<script lang="ts">
    import type { Theme } from "./Theme"
    import LinesView, { lines } from "./LinesView.svelte"
    import type { Cursor } from "./Cursors.svelte"
    import Cursors, { cursors } from "./Cursors.svelte"

    export let theme: Theme;

    export let height: number;
    export let width: number;

    // debug shit
    cursors.update(_ => [{pos: [2, 1]}])
    lines.update(_ => [ ...(new Array(10)).fill("")
                      , ..."funky\nbanana\nt0wn".split("\n")
                      , ...(new Array(20)).fill("a")
                      ])

    // let portion_start_line: number = 0

    const getCharacterWidth = (font: string): number | null => {
        const canvas = new OffscreenCanvas(0, 0)
        const context = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D | null
        if (context) {
          context.font = font
        }
        // REMARK: Elkowar mentioned: "with a font of **ZERO WIDTH X-CHARACTERS**" this breaks.
        // It does. ture. (typo intended)
        return context?.measureText("X").width || null
    }

    const font = theme.font
    const columnwidth = getCharacterWidth(`${font.weight} ${font.size} ${font.family}`) || 1;

    let view: Element;
    $: view_rect = view && view.getBoundingClientRect()

    const mousedown = (ev: MouseEvent) => {
        const column = Math.floor((ev.pageX - view_rect.x) / columnwidth)
        const line = Math.floor((ev.pageY - view_rect.y) / theme.line_height)

        cursors.update(cursors => {
            cursors[0] = { pos: [column, line] }
            return cursors
        })
    }

    const keydown = (ev: KeyboardEvent) => {
        alert!("funny")
        switch (ev.key) {
            case "j": cursors.update(cursors => {
                cursors[0] = { pos: [cursors[0].pos[0], cursors[0].pos[1] + 1] }
                return cursors
            })
            case "l": cursors.update(cursors => {
                cursors[0] = { pos: [cursors[0].pos[0] + 1, cursors[0].pos[1]] }
                return cursors
            })
        }
    }
</script>

<div class="portion-view" bind:this={view} on:mousedown={mousedown}
                          style:width={width}px style:height={height}px style:overflow=hidden>
    <!-- <GutterColumn /> -->
    <div class="container" style:position=relative
                           style:width=1000000px style:height=1000000px
                           style:background={theme.editor_background}>
        <div class="lines-container">
          <LinesView theme={theme} width={width} />
        </div>
        <Cursors theme={theme} />
    </div>
    <textarea on:keydown={keydown}
              style:width=0px style:height=0px style:padding=0 style:border=0 style:margin=0/>
    <!-- <Scrollbar /> -->
    <!-- <Scrollbar /> -->
</div>

<!--
    PortionView, may be simply the EditorView
    This window contains the visible and editable text.
-->