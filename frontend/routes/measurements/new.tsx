import { page } from "fresh";
import { define } from "../../utils.ts";
import Button from "../../components/Button.tsx";
import { getSensors, type Sensor } from "../../lib/sensor.ts";
import { type Device, getDevices } from "../../lib/device.ts";
import { createMeasurement } from "../../lib/measurements.ts";

interface Data {
  devices: Device[];
  sensors: Sensor[];
  error?: string;
}

export const handler = define.handlers({
  async GET(_ctx) {
    const [devices, sensors] = await Promise.all([getDevices(), getSensors()]);
    return page({ devices, sensors, error: undefined as string | undefined });
  },
  async POST(ctx) {
    const form = await ctx.req.formData();
    const device_id = form.get("device_id")?.toString() || "";
    const sensor_id = form.get("sensor_id")?.toString() || "";
    const value = form.get("value")?.toString() || "";

    if (device_id && sensor_id && value) {
      await createMeasurement(
        parseInt(device_id),
        parseInt(sensor_id),
        parseFloat(value),
      );
      console.log(
        "Measurement created:",
        `Device ID: ${device_id}, Sensor ID: ${sensor_id}, Value: ${value}`,
      );
      return new Response(null, {
        status: 303,
        headers: { Location: "/measurements" },
      });
    }

    const [devices, sensors] = await Promise.all([getDevices(), getSensors()]);
    return page({ devices, sensors, error: "Please fill in all fields." });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">New measurement</h1>
        {data.error && <p class="text-red-500 mb-4">{data.error}</p>}
        <form method="POST">
          <label class="block mb-2">
            Device:
            <select
              name="device_id"
              class="border rounded px-3 py-2 w-full"
              required
            >
              {data.devices.map((device) => (
                <option value={device.id} key={device.id}>
                  {device.name} ({device.location})
                </option>
              ))}
            </select>
          </label>
          <label class="block mb-2">
            Sensor:
            <select
              name="sensor_id"
              class="border rounded px-3 py-2 w-full"
              required
            >
              {data.sensors.map((sensor) => (
                <option value={sensor.id} key={sensor.id}>
                  {sensor.name} ({sensor.unit})
                </option>
              ))}
            </select>
          </label>
          <label class="block mb-2">
            Value:
            <input
              type="number"
              name="value"
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <Button
            type="submit"
            class="bg-blue-500 text-white px-4 py-2 rounded"
          >
            Create Measurement
          </Button>
        </form>
      </div>
    </div>
  );
});
