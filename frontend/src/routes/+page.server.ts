import type { Measurement } from "$lib/measurement"
import type { Sensor } from "$lib/sensor"
import type { Device } from "$lib/device"
import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"

export async function load({ fetch }) {
    console.log(PUBLIC_HEMRS_BASEURL)
    let devs: Device[] = fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices`).then(r => r.json())
    let sens: Sensor[] = fetch(`${PUBLIC_HEMRS_BASEURL}/api/sensors`).then(r => r.json())
    let meas: Number = fetch(`${PUBLIC_HEMRS_BASEURL}/api/measurements/count`).then(r => r.json());

    return {
        devices: devs,
        sensors: sens,
        measurements: meas
    }
}
