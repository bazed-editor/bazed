<!--
    PortionView, may be simply the EditorView
    This window contains the visible and editable text.
-->

<script lang="ts">
    import LinesView from "./LinesView.svelte"
    import type { Cursor } from "./Cursors.svelte"
    import Cursors from "./Cursors.svelte"

    export let height: number;
    export let width: number;

    const lines = "funky\nbanana\ntown".split("\n")
    export let portion_start_line: number = 0
    export let cursors: Cursor[] = [{pos: [2, 1]}]
</script>

<div class="portion-view">
    <LinesView height={height}
               width={width}
               starting_line_offset={portion_start_line} 
               lines={lines} />
    <Cursors cursors={cursors} />
</div>