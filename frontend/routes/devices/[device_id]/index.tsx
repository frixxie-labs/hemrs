import { Context } from "fresh";
import { getSensorsByDeviceId, Sensor } from "../../../lib/sensor.ts";
import { Device, getDevices } from "../../../lib/device.ts";
import SensorList from "../../../islands/SensorList.tsx";

interface DeviceProps {
  sensors: Promise<Sensor[]>;
  device: Device;
}

export const handler = {
  async GET(ctx: Context<DeviceProps>) {
    const device_id = ctx.params.device_id;
    const sensors = getSensorsByDeviceId(parseInt(device_id));
    const device = await getDevices().then((devices) =>
      devices.find((d) => d.id === parseInt(device_id))
    );
    if (!device) {
      throw new Error(`Device with ID ${device_id} not found`);
    }

    ctx.state.sensors = sensors;
    ctx.state.device = device;

    return await Home(ctx);
  },
};

export default async function Home(ctx: Context<DeviceProps>) {
  const sensors = await ctx.state.sensors;

  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-3xl font-bold mb-4">
          Device: {ctx.state.device.name}
        </h1>
        <p class="text-lg mb-6">
          Device Location: {ctx.state.device.location}
        </p>
        <SensorList
          device_id={ctx.state.device.id}
          sensors={sensors}
        />
      </div>
    </div>
  );
}
