import type { Measurement } from "./measurements.ts";

export interface MeasurementUpdate {
  device_id: number;
  sensor_id: number;
  measurement: Measurement;
}

export function measurementStreamPath(
  deviceId: number,
  sensorId: number,
): string {
  return `/api/devices/${deviceId}/sensors/${sensorId}/measurements/stream`;
}

export function parseMeasurementUpdate(data: string): MeasurementUpdate | null {
  try {
    const update: unknown = JSON.parse(data);
    if (!isRecord(update) || !isMeasurement(update.measurement)) return null;
    if (
      !Number.isInteger(update.device_id) || !Number.isInteger(update.sensor_id)
    ) {
      return null;
    }

    return update as unknown as MeasurementUpdate;
  } catch {
    return null;
  }
}

export function newestMeasurement(
  current: Measurement,
  candidate: Measurement,
): Measurement {
  const currentTimestamp = Date.parse(current.timestamp);
  const candidateTimestamp = Date.parse(candidate.timestamp);
  return Number.isNaN(candidateTimestamp) ||
      candidateTimestamp < currentTimestamp
    ? current
    : candidate;
}

function isMeasurement(value: unknown): boolean {
  return isRecord(value) &&
    typeof value.timestamp === "string" &&
    typeof value.value === "number" &&
    typeof value.unit === "string" &&
    typeof value.device_name === "string" &&
    typeof value.device_location === "string" &&
    typeof value.sensor_name === "string";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
