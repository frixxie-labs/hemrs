import { page } from "fresh";
import { define } from "../../utils.ts";
import Button from "../../components/Button.tsx";
import { createSensor } from "../../lib/sensor.ts";

export const handler = define.handlers({
  GET(_ctx) {
    return page({ error: undefined as string | undefined });
  },
  async POST(ctx) {
    const form = await ctx.req.formData();
    const sensor_name = form.get("sensor_name")?.toString() || "";
    const sensor_unit = form.get("sensor_unit")?.toString() || "";
    if (sensor_name && sensor_unit) {
      await createSensor(sensor_name, sensor_unit);
      console.log(
        "Sensor created:",
        `Name: ${sensor_name}, Unit: ${sensor_unit}`,
      );
      return new Response(null, {
        status: 303,
        headers: { Location: "/sensors" },
      });
    }
    return page({ error: "Please fill in all fields." });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">New Sensor</h1>
        {data.error && <p class="text-red-500 mb-4">{data.error}</p>}
        <form method="POST">
          <label class="block mb-2">
            Sensor Name:
            <input
              type="text"
              name="sensor_name"
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <label class="block mb-2">
            Sensor Unit:
            <input
              type="text"
              name="sensor_unit"
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
});
