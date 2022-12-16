<!--
    LinesView contains the lines of code
-->

<script lang="ts">
    export let lines: string[] = []
    export let starting_line_offset: number = 0
    export let line_height_px: number = 16
    export let gutter_width: number = 30
</script>

<div class="lines-container">
    {#each lines as line, i}
        <div class="line-container" style="top: {i * line_height_px}px; line-height: {line_height_px}px">
            <div class="line-gutter-cell" style="width: {gutter_width}px">{starting_line_offset + i}</div>
            <div class="line-view">{line}</div>
        </div>
    {/each}
</div>

<style>
    .line-gutter-cell {
        padding-left: 10px;
        background: darkgray;
    }

    .line-container {
        display: flex;
        position: absolute;
        background: dimgray;
    }

    .lines-container {
        font-family: monospace;
        position: relative;
    }
</style>