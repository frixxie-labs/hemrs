import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"
import type { Device } from "$lib/device"


export async function load({ fetch }) {
    let devs: Promise<Device[]> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices`).then(r => r.json())

    return {
        devices: devs,
    }
}
