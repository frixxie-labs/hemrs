import { Context } from "fresh";
import Button from "../../components/Button.tsx";
import { createSensor } from "../../lib/sensor.ts";

interface NewSensorProps {
  sensor_name: string;
  sensor_unit: string;
}

export const handler = {
  async GET(ctx: Context<NewSensorProps>) {
    const sensor_name = ctx.url.searchParams.get("sensor_name") || "";
    const sensor_unit = ctx.url.searchParams.get("sensor_unit") || "";
    if (sensor_name !== "" && sensor_unit !== "") {
      await createSensor(sensor_name, sensor_unit);
      console.log(
        "Sensor created:",
        `Name: ${sensor_name}, Location: ${sensor_unit}`,
      );
      return Response.redirect(`${ctx.url.origin}/devices`);
    }
    return new_sensor(ctx);
  },
};

export default function new_sensor(ctx: Context<NewSensorProps>) {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">New Sensor</h1>
        <form>
          <label class="block mb-2">
            Sensor Name:
            <input
              type="text"
              name="sensor_name"
              value={ctx.state.sensor_name}
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <label class="block mb-2">
            Sensor Location:
            <input
              type="text"
              name="sensor_unit"
              value={ctx.state.sensor_unit}
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <Button
            type="submit"
            class="bg-blue-500 text-white px-4 py-2 rounded"
          >
            Create Sensor
          </Button>
        </form>
      </div>
    </div>
  );
}
