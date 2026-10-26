import {
  backendJson,
  optionalBackendJson,
  sendBackendCommand,
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

export function createSensor(name: string, unit: string): Promise<void> {
  return sendBackendCommand(
    "api/sensors",
    "POST",
    { name, unit },
    "Failed to create sensor",
  );
}

export function deleteSensor(sensor: Sensor): Promise<void> {
  return sendBackendCommand(
    "api/sensors",
    "DELETE",
    sensor,
    "Failed to delete sensor",
  );
}
