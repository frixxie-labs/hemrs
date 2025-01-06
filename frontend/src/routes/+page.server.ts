import type { Sensor } from "$lib/sensor"
import type { Device } from "$lib/device"
import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"

export async function load({ fetch }) {
    console.log(PUBLIC_HEMRS_BASEURL)
    let devs: Promise<Device[]> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices`).then((r: Response) => r.json())
    let sens: Promise<Sensor[]> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/sensors`).then((r: Response) => r.json())
    let meas: Promise<Number> = fetch(`${PUBLIC_HEMRS_BASEURL}/api/measurements/count`).then((r: Response) => r.json());

    return {
        devices: devs,
        sensors: sens,
        measurements: meas
    }
}
