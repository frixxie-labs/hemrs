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
      console.log(
        "Device created:",
        `Name: ${device_name}, Location: ${device_location}`,
      );
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
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">New Device</h1>
        {data.error && <p class="text-red-500 mb-4">{data.error}</p>}
        <form method="POST">
          <label class="block mb-2">
            Device Name:
            <input
              type="text"
              name="device_name"
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <label class="block mb-2">
            Device Location:
            <input
              type="text"
              name="device_location"
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <Button
            type="submit"
            class="bg-blue-500 text-white px-4 py-2 rounded"
          >
            Create Device
          </Button>
        </form>
      </div>
    </div>
  );
});
