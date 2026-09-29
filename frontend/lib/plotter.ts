const PLOTTER_URL = Deno.env.get("PLOTTER_URL") || "http://localhost:8000/";

export async function fetchPlotSvg(path: string): Promise<string | null> {
  try {
    const response = await fetch(`${PLOTTER_URL}${path}`, {
      signal: AbortSignal.timeout(45_000),
    });
    if (!response.ok) {
      console.error(`Plotter request failed: ${response.status} for ${path}`);
      return null;
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    let binary = "";
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    const base64 = btoa(binary);
    return `data:image/svg+xml;base64,${base64}`;
  } catch (error) {
    console.error("Failed to fetch plot:", error);
    return null;
  }
}

export interface PlotComparison {
  sensor: string;
  page: number;
}

export function getLatestAllPlot(
  comparison?: PlotComparison,
): Promise<string | null> {
  const params = comparison
    ? `?${new URLSearchParams({
      sensor: comparison.sensor,
      page: String(comparison.page),
    })}`
    : "";
  return fetchPlotSvg(`plot/measurements/latest/all${params}`);
}

export function getAllMeasurementsPlot(): Promise<string | null> {
  return fetchPlotSvg("plot/measurements");
}

const MINUTE_MS = 60 * 1000;

/**
 * Compare devices for one sensor over the last `hours` hours. The start is floored
 * to the minute so repeated page loads share the plotter cache entry.
 */
export function getRecentMeasurementsPlot(
  hours = 24,
  now = Date.now(),
  comparison?: PlotComparison,
): Promise<string | null> {
  const start = new Date(
    Math.floor((now - hours * 60 * MINUTE_MS) / MINUTE_MS) * MINUTE_MS,
  ).toISOString();
  const params = new URLSearchParams({ start });
  if (comparison) {
    params.set("sensor", comparison.sensor);
    params.set("page", String(comparison.page));
  }
  return fetchPlotSvg(`plot/measurements/range?${params}`);
}

export function getDeviceMeasurementsPlot(
  deviceId: number,
): Promise<string | null> {
  return fetchPlotSvg(`plot/devices/${deviceId}/measurements`);
}

export function getDeviceSensorMeasurementsPlot(
  deviceId: number,
  sensorId: number,
): Promise<string | null> {
  return fetchPlotSvg(
    `plot/devices/${deviceId}/sensors/${sensorId}/measurements`,
  );
}

export function getTodayDeviceSensorMeasurementsPlot(
  deviceId: number,
  sensorId: number,
): Promise<string | null> {
  const start = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
  return fetchPlotSvg(
    `plot/devices/${deviceId}/sensors/${sensorId}/measurements?start=${start}`,
  );
}
