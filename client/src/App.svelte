<script lang="ts">

    import TunnelDisplay from "./lib/TunnelDisplay.svelte";
    import {tunnels} from "./stores/tunnels";
    import TunnelCreation from "./lib/TunnelCreation.svelte";
</script>

<div>
    <h2>Tunnels</h2>

    {#each $tunnels as tunnel}
        <TunnelDisplay bind:tunnel={tunnel}/>
    {/each}
</div>

<TunnelCreation/>