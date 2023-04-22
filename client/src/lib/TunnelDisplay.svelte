<script lang="ts">
    import type {Tunnel} from "../types/tunnel";
    import {deleteTunnel} from "../stores/tunnels";

    export let tunnel: Tunnel;

</script>

<div>
    {tunnel.domain_from} -> {tunnel.domain_to}
    <button on:click={() => deleteTunnel(tunnel)}>X</button>
</div>