import { Context } from "fresh";
import Button from "../../components/Button.tsx";
import { Device, getDevices } from "../../lib/device.ts";
import DeviceList from "../../components/DeviceList.tsx";

interface DeviceProps {
  devices: Promise<Device[]>;
}

export const handler = {
  async GET(ctx: Context<DeviceProps>) {
    const devices = getDevices();

    ctx.state.devices = devices;
    return await Devices(ctx);
  },
};

export default async function Devices(ctx: Context<DeviceProps>) {
  const devices = await ctx.state.devices;
  return (
    <div class="px-2 sm:px-4 py-4 sm:py-8 mx-auto">
      <div class="max-w-screen-lg mx-auto flex flex-col items-center justify-center space-y-4">
        <h1 class="text-2xl sm:text-3xl font-bold mb-4 text-gray-800">
          Devices
        </h1>
        <a href="/devices/new" class="mb-4">
          <Button type="button">New Device</Button>
        </a>
        <DeviceList
          devices={devices}
        />
      </div>
    </div>
  );
}
