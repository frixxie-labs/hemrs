import { HttpError, page } from "fresh";
import { define } from "../../../../utils.ts";
import { getMeasurementStats } from "../../../../lib/measurement_stats.ts";
import LiveMeasurementStatCard from "../../../../islands/LiveMeasurementStatCard.tsx";
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
      <div class="monitor-heading">
        <div>
          <h1>{data.sensor.name}</h1>
          <p class="text-text-secondary text-sm mt-2">
            {data.device.name} · #{data.device.id}
          </p>
        </div>
        <a
          class="text-sm text-text-secondary"
          href={`/devices/${data.device.id}`}
        >
          Device details ↗
        </a>
      </div>
      <LiveMeasurementStatCard
        deviceId={data.device.id}
        sensorId={data.sensor.id}
        measurementStats={data.stats}
        initialLatest={data.latest}
      />
      <PlotCard
        title={`${data.sensor.name} · last 24 hours`}
        svg={data.plot}
      />
    </div>
  );
});
