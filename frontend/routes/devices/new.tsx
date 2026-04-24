import { page } from "fresh";
import { define } from "../../utils.ts";
import Button from "../../components/Button.tsx";
import { createDevice } from "../../lib/device.ts";

export const handler = define.handlers({
  GET(_ctx) {
    return page({ error: undefined as string | undefined });
  },
  async POST(ctx) {
    const form = await ctx.req.formData();
    const device_name = form.get("device_name")?.toString() || "";
    const device_location = form.get("device_location")?.toString() || "";
    if (device_name && device_location) {
      await createDevice(device_name, device_location);
      return new Response(null, {
        status: 303,
        headers: { Location: "/devices" },
      });
    }
    return page({ error: "Please fill in all fields." });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto">
        <h1 class="text-2xl font-bold mb-4 text-text-primary">New Device</h1>
        {data.error && <p class="text-red-400 mb-4">{data.error}</p>}
        <form
          method="POST"
          class="bg-dark-card border border-dark-border rounded-xl p-6 space-y-4"
        >
          <label class="block">
            <span class="text-text-secondary text-sm">Device Name:</span>
            <input
              type="text"
              name="device_name"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            />
          </label>
          <label class="block">
            <span class="text-text-secondary text-sm">Device Location:</span>
            <input
              type="text"
              name="device_location"
              class="bg-dark-card-inner border border-dark-border rounded-lg px-3 py-2 w-full text-text-primary mt-1 focus:outline-none focus:border-accent-green"
              required
            />
          </label>
          <Button
            type="submit"
            class="bg-accent-green-dim text-dark-bg hover:bg-accent-green"
          >
            Create Device
          </Button>
        </form>
      </div>
    </div>
  );
});
