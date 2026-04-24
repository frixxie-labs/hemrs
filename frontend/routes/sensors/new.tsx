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
      <div class="max-w-screen-md mx-auto">
        <h1 class="text-2xl font-bold mb-4 text-text-primary">New Sensor</h1>
        {data.error && <p class="text-red-400 mb-4">{data.error}</p>}
        <form
          method="POST"
          class="bg-dark-card border border-dark-border rounded-xl p-6 space-y-4"
        >
          <label class="block">
            <span class="text-text-secondary text-sm">Sensor Name:</span>
            <input
              type="text"
              name="sensor_name"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            />
          </label>
          <label class="block">
            <span class="text-text-secondary text-sm">Sensor Unit:</span>
            <input
              type="text"
              name="sensor_unit"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            />
          </label>
          <Button
            type="submit"
            class="bg-accent-green-dim text-dark-bg hover:bg-accent-green"
          >
            Create Sensor
          </Button>
        </form>
      </div>
    </div>
  );
});
