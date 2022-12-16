<script lang="ts">
    import PortionView from "./PortionView.svelte"
    export let width: number = 1000;
    export let height: number = 1000;
</script>

<div class="rekuho" style="width: {width}px; height: {height}px">
    <PortionView width={width} height={height} />
</div>

<style>
    * {
        font-family: monospace;
    }
</style>