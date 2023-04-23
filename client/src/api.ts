import type {Tunnel} from "./types/tunnel";

export async function apiGetAllTunnels() {
    const response = await fetch(import.meta.env.VITE_SERVER_URL + "/api/")
    let json: Tunnel[] = await response.json();
    return json;
}

export async function apiDeleteTunnel(tunnel: Tunnel) {
    await fetch(import.meta.env.VITE_SERVER_URL + "/api/delete/", {
        method: "POST",
        body: JSON.stringify({id: tunnel.id})
    })
}


export async function apiAddTunnel(from: string, to: string): Promise<number>{
    let res = await fetch( import.meta.env.VITE_SERVER_URL+ "/api/", {
        method: "POST",
        body: JSON.stringify({from, to})
    });

    return await res.json();
}