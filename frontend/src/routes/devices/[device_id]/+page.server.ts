import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"
import type { Sensor } from "$lib/sensor"


export async function load({ fetch, params }) {
    let sens: Promise<Sensor[]> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices/${params.device_id}/sensors`).then((r: Response) => r.json())

    return {
        device_id: params.device_id,
        sensors: sens,
    }
}
