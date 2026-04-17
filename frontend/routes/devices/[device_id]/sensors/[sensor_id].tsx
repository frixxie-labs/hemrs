import { HttpError, page } from "fresh";
import { define } from "../../../../utils.ts";
import { getMeasurementStats } from "../../../../lib/measurement_stats.ts";
import MeasurementStatCard from "../../../../components/MeasurementStatCard.tsx";
import PlotCard from "../../../../components/PlotCard.tsx";
import { getDeviceById } from "../../../../lib/device.ts";
import { getSensorById } from "../../../../lib/sensor.ts";
import {
  getLatestMeasurementByDeviceAndSensorId,
} from "../../../../lib/measurements.ts";
import { getTodayDeviceSensorMeasurementsPlot } from "../../../../lib/plotter.ts";

export const handler = define.handlers({
  async GET(ctx) {
    const device_id = parseInt(ctx.params.device_id);
    const sensor_id = parseInt(ctx.params.sensor_id);

    const [stats, latest, plot, device, sensor] = await Promise.all([
      getMeasurementStats(device_id, sensor_id),
      getLatestMeasurementByDeviceAndSensorId(device_id, sensor_id),
      getTodayDeviceSensorMeasurementsPlot(device_id, sensor_id),
      getDeviceById(device_id),
      getSensorById(sensor_id),
    ]);

    if (!device) {
      throw new HttpError(404, `Device with ID ${device_id} not found`);
    }
    if (!sensor) {
      throw new HttpError(404, `Sensor with ID ${sensor_id} not found`);
    }

    return page({ stats, latest, plot, device, sensor });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">
          Measurement Statistics for {data.sensor.name}
        </h1>
        <p class="text-lg mb-4">
          Device: {data.device.name} ({data.device.id})
        </p>
        <MeasurementStatCard
          measurement_stats={data.stats}
          latest={data.latest}
        />
        <PlotCard
          title={`${data.sensor.name} Measurements Over Time`}
          svg={data.plot}
        />
      </div>
    </div>
  );
});
