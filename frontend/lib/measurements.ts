import { backendJson, backendRequest } from "./backend_client.ts";

export interface Measurement {
  timestamp: string;
  value: number;
  unit: string;
  device_name: string;
  device_location: string;
  sensor_name: string;
}

export function getLatestMeasurement(): Promise<Measurement> {
  return backendJson<Measurement>("api/measurements/latest");
}

export function getLatestMeasurementByDeviceAndSensorId(
  deviceId: number,
  sensorId: number,
): Promise<Measurement> {
  return backendJson<Measurement>(
    `api/devices/${deviceId}/sensors/${sensorId}/measurements/latest`,
  );
}

export function getAllLatestMeasurements(): Promise<Measurement[]> {
  return backendJson<Measurement[]>("api/measurements/latest/all");
}

export function getMeasurementCount(): Promise<number> {
  return backendJson<number>("api/measurements/count");
}

export async function createMeasurement(
  device: number,
  sensor: number,
  measurement: number,
) {
  const response = await backendRequest("api/measurements", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ device, sensor, measurement }),
  });
  await response.text();
}
