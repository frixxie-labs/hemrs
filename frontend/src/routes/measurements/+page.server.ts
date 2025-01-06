import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"
import type { Measurement } from "$lib/measurement"


export async function load({ fetch }) {
    let meas: Promise<Measurement> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/measurements/latest`).then((r: Response) => r.json())

    return {
        measurement: meas
    }
}
