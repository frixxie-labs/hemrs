import {
  backendJson,
  optionalBackendJson,
  sendBackendCommand,
} from "./backend_client.ts";

export interface Coordinates {
  latitude: number;
  longitude: number;
}

export interface Device {
  id: number;
  name: string;
  location: string;
  coordinates?: Coordinates | null;
}

export function getDeviceById(deviceId: number): Promise<Device | null> {
  return deviceId
    ? optionalBackendJson<Device>(`api/devices/${deviceId}`)
    : Promise.resolve(null);
}

export function getDevices(): Promise<Device[]> {
  return backendJson<Device[]>("api/devices");
}

export function createDevice(name: string, location: string): Promise<void> {
  return sendBackendCommand(
    "api/devices",
    "POST",
    { name, location },
    "Failed to create device",
  );
}

export function deleteDevice(device: Device): Promise<void> {
  return sendBackendCommand(
    "api/devices",
    "DELETE",
    device,
    "Failed to delete device",
  );
}
