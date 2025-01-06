import { PUBLIC_HEMRS_BASEURL } from "$env/static/public";

export const actions = {
    default: async ({ fetch, request }) => {
        const data = await request.formData();
        const sensor_name = data.get('name');
        const sensor_unit = data.get('unit');

        if (!sensor_name || !sensor_unit) return { status: false }

        console.log(sensor_name, sensor_unit);

        var res = await fetch(`${PUBLIC_HEMRS_BASEURL}/api/sensors`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ name: sensor_name, unit: sensor_unit })
        })

        console.log(res)

        return { status: true }
    }
}
