import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"
import type { Sensor } from "$lib/sensor"


export async function load({ fetch }) {
    let sens: Promise<Sensor[]> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/sensors`).then(r => r.json())

    return {
        sensors: sens,
    }
}
