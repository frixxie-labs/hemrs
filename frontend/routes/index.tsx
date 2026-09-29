import { page } from "fresh";
import { define } from "../utils.ts";
import { getDevices } from "../lib/device.ts";
import {
  getAllLatestMeasurements,
  getMeasurementCount,
} from "../lib/measurements.ts";
import { getLatestAllPlot, getRecentMeasurementsPlot } from "../lib/plotter.ts";
import {
  getOverviewSelection,
  OVERVIEW_PAGE_SIZE,
  overviewPageUrl,
} from "../lib/overview.ts";
import { getSensors } from "../lib/sensor.ts";
import DeviceGroups from "../islands/DeviceGroups.tsx";
import MeasurementsList from "../components/MeasurementsList.tsx";
import PlotCard from "../components/PlotCard.tsx";

export const handler = define.handlers({
  async GET(ctx) {
    const [
      devices,
      sensors,
      measurement_count,
      latest,
    ] = await Promise.all([
      getDevices(),
      getSensors(),
      getMeasurementCount(),
      getAllLatestMeasurements(),
    ]);
    const selection = getOverviewSelection(
      latest,
      ctx.url.searchParams.get("sensor"),
      ctx.url.searchParams.get("plot_page"),
    );
    const comparison = selection.selected
      ? { sensor: selection.selected.name, page: selection.page }
      : null;
    const [historyPlot, latestPlot] = comparison
      ? await Promise.all([
        getRecentMeasurementsPlot(24, Date.now(), comparison),
        getLatestAllPlot(comparison),
      ])
      : [null, null];
    return page({
      devices,
      sensors,
      measurement_count,
      latest,
      historyPlot,
      latestPlot,
      selection,
    });
  },
});

export default define.page<typeof handler>(({ data }) => {
  const { selected, options, page, pages } = data.selection;
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
      <section id="overview-plots" aria-label="Sensor comparison">
        <div class="monitor-plot-controls">
          <form method="get" action="/#overview-plots">
            <label for="overview-sensor">Compare sensor</label>
            <select id="overview-sensor" name="sensor" value={selected?.name}>
              {options.map((option) => (
                <option value={option.name} key={option.name}>
                  {option.name} ({option.unit})
                </option>
              ))}
            </select>
            <button type="submit" disabled={!selected}>Show plots</button>
          </form>
          {selected && (
            <nav aria-label="Comparison devices" class="monitor-plot-paging">
              <span>
                Devices {(page - 1) * OVERVIEW_PAGE_SIZE + 1}–{Math.min(
                  page * OVERVIEW_PAGE_SIZE,
                  selected.count,
                )} of {selected.count}
              </span>
              {page > 1 && (
                <a href={overviewPageUrl(selected.name, page - 1)}>
                  ← Previous
                </a>
              )}
              {page < pages && (
                <a href={overviewPageUrl(selected.name, page + 1)}>Next →</a>
              )}
            </nav>
          )}
        </div>
        <p class="monitor-plot-note">
          {selected
            ? `Comparing ${selected.name} in ${selected.unit}. Colors identify the same devices in both plots.`
            : "Plots will appear when sensors report measurements."}
        </p>
        <div class="monitor-plots">
          <PlotCard title="Measurement history (24h)" svg={data.historyPlot} />
          <PlotCard title="Latest readings by device" svg={data.latestPlot} />
        </div>
      </section>
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
