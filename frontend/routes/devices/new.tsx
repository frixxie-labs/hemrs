import { Context } from "fresh";
import Button from "../../components/Button.tsx";
import { createDevice } from "../../lib/device.ts";

interface NewDeviceProps {
  device_name: string;
  device_location: string;
}

export const handler = {
  async GET(ctx: Context<NewDeviceProps>) {
    const device_name = ctx.url.searchParams.get("device_name") || "";
    const device_location = ctx.url.searchParams.get("device_location") || "";
    if (device_name !== "" && device_location !== "") {
      await createDevice(device_name, device_location);
      return Response.redirect(`${ctx.url.origin}/devices`);
    }
    console.log(
      "Device created:",
      `Name: ${device_name}, Location: ${device_location}`,
    );
    return new_device(ctx);
  },
};

export default function new_device(ctx: Context<NewDeviceProps>) {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">New Device</h1>
        <form>
          <label class="block mb-2">
            Device Name:
            <input
              type="text"
              name="device_name"
              value={ctx.state.device_name}
              class="border rounded px-3 py-2 w-full"
              required
            />
          </label>
          <label class="block mb-2">
            Device Location:
            <input
              type="text"
              name="device_location"
              value={ctx.state.device_location}
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
}
