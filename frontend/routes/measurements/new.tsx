import { Context } from "fresh";
import Button from "../../components/Button.tsx";
import { getSensors, Sensor } from "../../lib/sensor.ts";
import { Device, getDevices } from "../../lib/device.ts";
import { createMeasurement } from "../../lib/measurements.ts";

interface NewMeasurementProps {
  device_id: number;
  sensor_id: number;
  value: number;
  devices: Device[];
  sensors: Sensor[];
}

export const handler = {
  async GET(ctx: Context<NewMeasurementProps>) {
    const device_id = ctx.url.searchParams.get("device_id") || "";
    const sensor_id = ctx.url.searchParams.get("sensor_id") || "";
    const value = ctx.url.searchParams.get("value") || "";

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
      return Response.redirect(`${ctx.url.origin}/measurements`);
    }

    const devices = await getDevices();
    const sensors = await getSensors();

    ctx.state.devices = devices;
    ctx.state.sensors = sensors;

    return new_measurement(ctx);
  },
};

export default function new_measurement(
  ctx: Context<NewMeasurementProps>,
) {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">New measurement</h1>
        <form>
          <label class="block mb-2">
            Device:
            <select
              name="device_id"
              value={ctx.state.device_id}
              class="border rounded px-3 py-2 w-full"
              required
            >
              {ctx.state.devices.map((device) => (
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
              value={ctx.state.sensor_id}
              class="border rounded px-3 py-2 w-full"
              required
            >
              {ctx.state.sensors.map((sensor) => (
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
              value={ctx.state.value}
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
}
