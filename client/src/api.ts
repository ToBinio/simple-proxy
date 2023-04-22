import type {Tunnel} from "./types/tunnel";

export async function apiGetAllTunnels() {
    const response = await fetch("http://main.localhost:8080/api/")
    let json: Tunnel[] = await response.json();
    return json;
}

export async function apiDeleteTunnel(tunnel: Tunnel) {
    await fetch("http://main.localhost:8080/api/delete/", {
        method: "POST",
        body: JSON.stringify({id: tunnel.id})
    })
}