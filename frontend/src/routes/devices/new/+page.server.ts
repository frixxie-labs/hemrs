import { PUBLIC_HEMRS_BASEURL } from "$env/static/public";

export const actions = {
    default: async ({ fetch, request }) => {
        const data = await request.formData();
        const device_name = data.get('name');
        const device_location = data.get('location');

        if (!device_name || !device_location) return { status: false }

        console.log(device_name, device_location);

        var res = await fetch(`${PUBLIC_HEMRS_BASEURL}/api/devices`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ name: device_name, location: device_location })
        })

        console.log(res)

        return { status: true }
    }
}
