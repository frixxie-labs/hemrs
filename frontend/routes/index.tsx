import { page } from "fresh";
import { define } from "../utils.ts";
import { getDevices } from "../lib/device.ts";
import {
  getAllLatestMeasurements,
  getMeasurementCount,
} from "../lib/measurements.ts";
import { getLatestAllPlot, getRecentMeasurementsPlot } from "../lib/plotter.ts";
import { getSensors } from "../lib/sensor.ts";
import DeviceGroups from "../islands/DeviceGroups.tsx";
import MeasurementsList from "../components/MeasurementsList.tsx";
import PlotCard from "../components/PlotCard.tsx";

export const handler = define.handlers({
  async GET() {
    const [
      devices,
      sensors,
      measurement_count,
      latest,
      historyPlot,
      latestPlot,
    ] = await Promise.all([
      getDevices(),
      getSensors(),
      getMeasurementCount(),
      getAllLatestMeasurements(),
      getRecentMeasurementsPlot(),
      getLatestAllPlot(),
    ]);
    return page({
      devices,
      sensors,
      measurement_count,
      latest,
      historyPlot,
      latestPlot,
    });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <>
      <div class="monitor-heading">
        <h1>Overview</h1>
        <span>Readings at page load</span>
      </div>
      <dl class="monitor-stats">
        {[["Devices", data.devices.length], [
          "Sensor types",
          data.sensors.length,
        ], ["Measurements", data.measurement_count]].map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{Number(value).toLocaleString("en-US")}</dd>
          </div>
        ))}
      </dl>
      <div class="monitor-plots">
        <PlotCard title="Measurement history (24h)" svg={data.historyPlot} />
        <PlotCard title="Latest measurements" svg={data.latestPlot} />
      </div>
      <MeasurementsList measurements={data.latest} showAllLink />
      <DeviceGroups devices={data.devices} />
      <section class="monitor-sensors">
        <h2>Sensor types</h2>
        <div>
          {data.sensors.map((sensor) => (
            <span key={sensor.id}>
              {sensor.name} <code>{sensor.unit}</code>
            </span>
          ))}
          {!data.sensors.length && <p>No sensor types registered.</p>}
        </div>
      </section>
    </>
  );
});
