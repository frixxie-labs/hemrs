import {
  backendJson,
  backendRequest,
  optionalBackendJson,
} from "./backend_client.ts";

export interface Sensor {
  id: number;
  name: string;
  unit: string;
}

export function getSensors(): Promise<Sensor[]> {
  return backendJson<Sensor[]>("api/sensors");
}

export function getSensorById(sensorId: number): Promise<Sensor | null> {
  return sensorId
    ? optionalBackendJson<Sensor>(`api/sensors/${sensorId}`)
    : Promise.resolve(null);
}

export function getSensorsByDeviceId(deviceId: number): Promise<Sensor[]> {
  return backendJson<Sensor[]>(`api/devices/${deviceId}/sensors`);
}

export async function createSensor(name: string, unit: string) {
  const response = await backendRequest("api/sensors", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name, unit }),
  });
  if (await response.text() !== "OK") {
    throw new Error("Failed to create sensor");
  }
}

export async function deleteSensor(sensor: Sensor) {
  const response = await backendRequest("api/sensors", {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(sensor),
  });
  if (await response.text() !== "OK") {
    throw new Error("Failed to delete sensor");
  }
}
