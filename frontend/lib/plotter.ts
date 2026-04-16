const PLOTTER_URL = Deno.env.get("PLOTTER_URL") || "http://localhost:8000/";

export async function fetchPlotSvg(path: string): Promise<string | null> {
  try {
    const response = await fetch(`${PLOTTER_URL}${path}`);
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

export function getLatestAllPlot(): Promise<string | null> {
  return fetchPlotSvg("plot/measurements/latest/all");
}

export function getAllMeasurementsPlot(): Promise<string | null> {
  return fetchPlotSvg("plot/measurements");
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
