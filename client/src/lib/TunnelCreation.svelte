<script lang="ts">
    import {addTunnel} from "../stores/tunnels";

    let from = "";
    let to = "";
</script>

<div>
    <h2>Create Tunnel</h2>

    <div>
        <input bind:value={from} type="text" name="from" id="from" placeholder="from">
        <input bind:value={to} type="text" name="from" id="to" placeholder="to">
        <button on:click={() => addTunnel(from,to)}>Add</button>
    </div>
</div>