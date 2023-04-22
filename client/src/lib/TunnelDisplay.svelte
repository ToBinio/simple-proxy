<script lang="ts">
    import type {Tunnel} from "../types/tunnel";
    import {apiDeleteTunnel} from "../api";
    import {deleteTunnel} from "../stores/tunnels";

    export let tunnel: Tunnel;

</script>

<div>
    <h2>Tunnel</h2>
    <div>
        {tunnel.domain_from} -> {tunnel.domain_to}
    </div>
    <button on:click={() => deleteTunnel(tunnel)}>X</button>
</div>