import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"
import type { Measurement } from "$lib/measurement"


export async function load({ fetch, params }) {
    let meas: Promise<Measurement[]> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices/${params.device_id}/sensors/${params.sensor_id}/measurements`).then((r: Response) => r.json())

    return {
        measurements: meas
    }
}
