import { PUBLIC_HEMRS_BASEURL } from "$env/static/public"
import type { Device } from "$lib/device"
import type { Sensor } from "$lib/sensor"


export async function load({ fetch }) {
    let sens: Sensor[] = await fetch(`${PUBLIC_HEMRS_BASEURL}/api/sensors`).then(r => r.json())
    let devs: Device[] = await fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices`).then(r => r.json())

    return {
        devices: devs,
        sensors: sens
    }
}

type Measurement = {
    device: number,
    sensor: number,
    measurement: number
}

export const actions = {
    default: async ({ fetch, request }) => {
        const data = await request.formData();
        let device_id = data.get('device');
        let sensor_id = data.get('sensor');
        let measurement = data.get('measurement');

        console.log(device_id, sensor_id, measurement);

        let measurementData: Measurement = {
            device: Number(device_id),
            sensor: Number(sensor_id),
            measurement: Number(measurement)
        }

        var res = await fetch(`${PUBLIC_HEMRS_BASEURL}/api/measurements`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(measurementData)
        })

        console.log(res)

        if (res.status == 200) {
            return { status: true }
        } else {
            return { status: false }
        }
    }
}
