import { Context } from "fresh";
import {
  getMeasurementStats,
  MeasurementStats as Stats,
} from "../../../../lib/measurement_stats.ts";
import MeasurementStatCard from "../../../../components/MeasurementStatCard.tsx";
import PlotCard from "../../../../components/PlotCard.tsx";
import { Device, getDeviceById } from "../../../../lib/device.ts";
import { getSensorById, Sensor } from "../../../../lib/sensor.ts";
import {
  getLatestMeasurementByDeviceAndSensorId,
  Measurement,
} from "../../../../lib/measurements.ts";
import { getDeviceSensorMeasurementsPlot } from "../../../../lib/plotter.ts";

interface MeasurementStatsProps {
  stats: Promise<Stats>;
  latest: Promise<Measurement>;
  device: Device;
  sensor: Sensor;
  plot: Promise<string | null>;
}

export const handler = {
  async GET(ctx: Context<MeasurementStatsProps>) {
    const device_id = ctx.params.device_id;
    const sensor_id = ctx.params.sensor_id;

    const stats = getMeasurementStats(
      parseInt(device_id),
      parseInt(sensor_id),
    );

    const latest = getLatestMeasurementByDeviceAndSensorId(
      parseInt(device_id),
      parseInt(sensor_id),
    );

    const plot = getDeviceSensorMeasurementsPlot(
      parseInt(device_id),
      parseInt(sensor_id),
    );

    const device = await getDeviceById(parseInt(device_id));
    if (!device) {
      throw new Error(`Device with ID ${device_id} not found`);
    }
    const sensor = await getSensorById(parseInt(sensor_id));
    if (!sensor) {
      throw new Error(`Sensor with ID ${sensor_id} not found`);
    }

    ctx.state.stats = stats;
    ctx.state.device = device;
    ctx.state.sensor = sensor;
    ctx.state.latest = latest;
    ctx.state.plot = plot;

    return MeasurementStats(ctx);
  },
};

export default async function MeasurementStats(
  ctx: Context<MeasurementStatsProps>,
) {
  const stats = await ctx.state.stats;
  const latest = await ctx.state.latest;
  const plot = await ctx.state.plot;
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">
          Measurement Statistics for {ctx.state.sensor.name}
        </h1>
        <p class="text-lg mb-4">
          Device: {ctx.state.device.name} ({ctx.state.device.id})
        </p>
        <MeasurementStatCard measurement_stats={stats} latest={latest} />
        <PlotCard title={`${ctx.state.sensor.name} Measurements Over Time`} svg={plot} />
      </div>
    </div>
  );
}
