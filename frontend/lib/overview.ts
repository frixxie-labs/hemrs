import type { Measurement } from "./measurements.ts";

// Matches the plotter's bounded comparison layout.
export const OVERVIEW_PAGE_SIZE = 6;

export function getOverviewSelection(
  measurements: Measurement[],
  requestedSensor: string | null,
  requestedPage: string | null,
) {
  const groups = new Map<
    string,
    { name: string; unit: string; devices: Set<string> }
  >();
  for (const m of measurements) {
    const group = groups.get(m.sensor_name) ?? {
      name: m.sensor_name,
      unit: m.unit,
      devices: new Set<string>(),
    };
    group.devices.add(JSON.stringify([m.device_name, m.device_location]));
    groups.set(m.sensor_name, group);
  }
  const options = [...groups.values()].map(({ name, unit, devices }) => ({
    name,
    unit,
    count: devices.size,
  })).sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);
  const selected = options.find((option) => option.name === requestedSensor) ??
    [...options].sort((a, b) => b.count - a.count)[0];
  const pages = Math.max(
    1,
    Math.ceil((selected?.count ?? 0) / OVERVIEW_PAGE_SIZE),
  );
  const parsedPage = Number(requestedPage);
  const page = Number.isSafeInteger(parsedPage)
    ? Math.min(pages, Math.max(1, parsedPage))
    : 1;
  return { options, selected, page, pages };
}

export function overviewPageUrl(sensor: string, page: number): string {
  return `/?${new URLSearchParams({
    sensor,
    plot_page: String(page),
  })}#overview-plots`;
}
