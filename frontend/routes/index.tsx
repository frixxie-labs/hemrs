import { page } from "fresh";
import { define } from "../utils.ts";
import MeasurementsInfo from "../components/MeasurementsInfo.tsx";
import { getDevices } from "../lib/device.ts";
import { getMeasurementCount } from "../lib/measurements.ts";
import { getSensors } from "../lib/sensor.ts";
import DeviceList from "../components/DeviceList.tsx";

export const handler = define.handlers({
  async GET(_ctx) {
    const [devices, sensors, measurement_count] = await Promise.all([
      getDevices(),
      getSensors(),
      getMeasurementCount(),
    ]);
    return page({ devices, sensors, measurement_count });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="px-2 sm:px-4 py-4 sm:py-8 mx-auto">
      <div class="max-w-screen-lg mx-auto flex flex-col items-center justify-center space-y-4">
        <h1 class="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">
          HEMRS Dashboard
        </h1>
        <MeasurementsInfo
          device_count={data.devices.length}
          sensor_count={data.sensors.length}
          measurement_count={data.measurement_count}
        />
        <DeviceList devices={data.devices} clickable />
      </div>
    </div>
  );
});
