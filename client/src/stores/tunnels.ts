import {writable} from "svelte/store";
import type {Tunnel} from "../types/tunnel";
import {apiAddTunnel, apiDeleteTunnel, apiGetAllTunnels} from "../api";

export const tunnels = writable<Tunnel[]>([])

export async function loadTunnels() {

    let newTunnels = await apiGetAllTunnels();

    tunnels.update(() => {
        return newTunnels;
    })
}

export async function deleteTunnel(tunnel: Tunnel) {

    await apiDeleteTunnel(tunnel)

    tunnels.update((tunnels) => {
        for (let i = 0; i < tunnels.length; i++) {
            if (tunnels[i].id == tunnel.id) {
                tunnels.splice(i, 1);
            }
        }

        return tunnels
    })
}

export async function addTunnel(from: string, to: string) {
    let id = await apiAddTunnel(from, to);

    console.log("asss " + id);

    tunnels.update((tunnels) => {
        tunnels.push({domain_from: from, domain_to: to, id: id})

        return tunnels
    })
}

//init
loadTunnels().then()