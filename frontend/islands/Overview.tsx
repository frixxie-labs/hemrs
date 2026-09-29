import type { OverviewResources } from "../lib/overview_data.ts";
import { useOverviewResource } from "../lib/use_overview_resource.ts";
import {
  getOverviewSelection,
  OVERVIEW_PAGE_SIZE,
  overviewPageUrl,
} from "../lib/overview.ts";
import DeviceGroups from "../islands/DeviceGroups.tsx";
import MeasurementsList from "../components/MeasurementsList.tsx";
import PlotCard from "../components/PlotCard.tsx";

function LoadingState({ label, error, retry }: {
  label: string;
  error: boolean;
  retry: () => void;
}) {
  return (
    <p class="monitor-empty" role="status">
      {error ? `${label} unavailable. ` : `Loading ${label.toLowerCase()}…`}
      {error && <button type="button" onClick={retry}>Retry</button>}
    </p>
  );
}

function AsyncPlot({ title, url }: { title: string; url: string }) {
  const plot = useOverviewResource<string>(url);
  return plot.data
    ? <PlotCard title={title} svg={plot.data} />
    : (
      <section class="monitor-plot" aria-busy={!plot.error}>
        <div class="monitor-section-heading">
          <h2>{title}</h2>
        </div>
        <LoadingState label="Plot" {...plot} />
      </section>
    );
}

export default function Overview({ sensor, plotPage }: {
  sensor: string | null;
  plotPage: string | null;
}) {
  const devices = useOverviewResource<OverviewResources["devices"]>(
    "/api/overview/devices",
  );
  const sensors = useOverviewResource<OverviewResources["sensors"]>(
    "/api/overview/sensors",
  );
  const count = useOverviewResource<number>("/api/overview/count");
  const latest = useOverviewResource<OverviewResources["latest"]>(
    "/api/overview/latest",
  );
  const { selected, options, page, pages } = getOverviewSelection(
    latest.data ?? [],
    sensor,
    plotPage,
  );
  const comparison = new URLSearchParams({
    sensor: selected?.name ?? "",
    page: String(page),
  });
  return (
    <>
      <div class="monitor-heading">
        <h1>Overview</h1>
        <span>Readings at page load</span>
      </div>
      <dl class="monitor-stats">
        {[
          { label: "Devices", value: devices.data?.length, resource: devices },
          {
            label: "Sensor types",
            value: sensors.data?.length,
            resource: sensors,
          },
          { label: "Measurements", value: count.data, resource: count },
        ].map(({ label, value, resource }) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>
              {value === undefined
                ? <LoadingState label={label} {...resource} />
                : value.toLocaleString("en-US")}
            </dd>
          </div>
        ))}
      </dl>
      <section id="overview-plots" aria-label="Sensor comparison">
        {!latest.data && <LoadingState label="Latest readings" {...latest} />}
        <div class="monitor-plot-controls">
          <form method="get" action="/#overview-plots">
            <label for="overview-sensor">Compare sensor</label>
            <select
              id="overview-sensor"
              name="sensor"
              value={selected?.name}
              disabled={!selected}
            >
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
            : latest.data
            ? "Plots will appear when sensors report measurements."
            : "Waiting for latest readings to select comparison plots."}
        </p>
        {selected && (
          <div class="monitor-plots">
            <AsyncPlot
              title="Measurement history (24h)"
              url={`/api/overview/history-plot?${comparison}`}
            />
            <AsyncPlot
              title="Latest readings by device"
              url={`/api/overview/latest-plot?${comparison}`}
            />
          </div>
        )}
      </section>
      {latest.data && (
        <MeasurementsList measurements={latest.data} showAllLink />
      )}
      {devices.data
        ? <DeviceGroups devices={devices.data} />
        : <LoadingState label="Devices" {...devices} />}
      <section class="monitor-sensors">
        <h2>Sensor types</h2>
        <div>
          {!sensors.data && <LoadingState label="Sensor types" {...sensors} />}
          {sensors.data?.map((sensor) => (
            <span key={sensor.id}>
              {sensor.name} <code>{sensor.unit}</code>
            </span>
          ))}
          {sensors.data?.length === 0 && <p>No sensor types registered.</p>}
        </div>
      </section>
    </>
  );
}
