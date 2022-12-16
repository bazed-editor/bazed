<script lang="ts" context="module">
    export type Cursor = { pos: [number, number] };
</script>

<script lang="ts">
    export let cursors: Cursor[] = []
    let visibility: string = "inherit";

    // const verticalOffsetPxFromLine = (): number => {
    //     
    // }
</script>


<div class="cursors-layer">
    {#each cursors as {pos: [x, y]}, i}
        <div style="visibility: {visibility}"></div>
    {/each}
</div>

<style>
    .cursors-layer {
        position: absolute;
        top: 0px;
    }
</style>