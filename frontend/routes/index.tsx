import { Context } from "fresh";
import MeasurementsInfo from "../components/MeasurementsInfo.tsx";
import { Device, getDevices } from "../lib/device.ts";
import { getMeasurementCount } from "../lib/measurements.ts";
import { getSensors, Sensor } from "../lib/sensor.ts";
import DeviceList from "../islands/DeviceList.tsx";

interface HomeProps {
  devices: Promise<Device[]>;
  sensors: Promise<Sensor[]>;
  measurement_count: Promise<number>;
}

export const handler = {
  async GET(ctx: Context<HomeProps>) {
    const devices = getDevices();
    const sensors = getSensors();
    const measurement_count = getMeasurementCount();

    ctx.state.devices = devices;
    ctx.state.sensors = sensors;
    ctx.state.measurement_count = measurement_count;
    return await Home(ctx);
  },
};

export default async function Home(ctx: Context<HomeProps>) {
  const devices = await ctx.state.devices;
  const sensors = await ctx.state.sensors;
  const measurement_count = await ctx.state.measurement_count;
  return (
    <div class="px-2 sm:px-4 py-4 sm:py-8 mx-auto">
      <div class="max-w-screen-lg mx-auto flex flex-col items-center justify-center space-y-4">
        <h1 class="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">
          HEMRS Dashboard
        </h1>
        <MeasurementsInfo
          device_count={devices.length}
          sensor_count={sensors.length}
          measurement_count={measurement_count}
        />
        <DeviceList
          devices={devices}
        />
      </div>
    </div>
  );
}
