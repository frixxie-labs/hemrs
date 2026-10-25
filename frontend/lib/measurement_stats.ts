import { backendJson } from "./backend_client.ts";

export interface MeasurementStats {
  min: number | null;
  max: number | null;
  count: number;
  avg: number | null;
  stddev: number | null;
  variance: number | null;
}

export function getMeasurementStats(
  device_id: number,
  sensor_id: number,
): Promise<MeasurementStats> {
  return backendJson<MeasurementStats>(
    `api/devices/${device_id}/sensors/${sensor_id}/measurements/stats`,
  );
}
