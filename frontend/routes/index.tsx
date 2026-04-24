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
    <div class="space-y-6">
      <MeasurementsInfo
        device_count={data.devices.length}
        sensor_count={data.sensors.length}
        measurement_count={data.measurement_count}
      />
      <DeviceList devices={data.devices} clickable searchable />
    </div>
  );
});
