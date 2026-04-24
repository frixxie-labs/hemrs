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
    <div class="space-y-4">
      <div class="bg-dark-card border border-dark-border rounded-xl p-6">
        <h1 class="text-2xl font-bold text-text-primary">
          {data.sensor.name}
        </h1>
        <p class="text-text-secondary mt-1">
          Device: {data.device.name} (#{data.device.id})
        </p>
      </div>
      <MeasurementStatCard
        measurement_stats={data.stats}
        latest={data.latest}
      />
      <PlotCard
        title={`${data.sensor.name} Measurements Over Time`}
        svg={data.plot}
      />
    </div>
  );
});
