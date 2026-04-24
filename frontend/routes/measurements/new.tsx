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
      <div class="max-w-screen-md mx-auto">
        <h1 class="text-2xl font-bold mb-4 text-text-primary">
          New measurement
        </h1>
        {data.error && <p class="text-red-400 mb-4">{data.error}</p>}
        <form
          method="POST"
          class="bg-dark-card border border-dark-border rounded-xl p-6 space-y-4"
        >
          <label class="block">
            <span class="text-text-secondary text-sm">Device:</span>
            <select
              name="device_id"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            >
              {data.devices.map((device) => (
                <option value={device.id} key={device.id}>
                  {device.name} ({device.location})
                </option>
              ))}
            </select>
          </label>
          <label class="block">
            <span class="text-text-secondary text-sm">Sensor:</span>
            <select
              name="sensor_id"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            >
              {data.sensors.map((sensor) => (
                <option value={sensor.id} key={sensor.id}>
                  {sensor.name} ({sensor.unit})
                </option>
              ))}
            </select>
          </label>
          <label class="block">
            <span class="text-text-secondary text-sm">Value:</span>
            <input
              type="number"
              name="value"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            />
          </label>
          <Button
            type="submit"
            class="bg-accent-green-dim text-dark-bg hover:bg-accent-green"
          >
            Create Measurement
          </Button>
        </form>
      </div>
    </div>
  );
});
