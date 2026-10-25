import {
  backendJson,
  backendRequest,
  optionalBackendJson,
} from "./backend_client.ts";

export interface Device {
  id: number;
  name: string;
  location: string;
}

export function getDeviceById(deviceId: number): Promise<Device | null> {
  return deviceId
    ? optionalBackendJson<Device>(`api/devices/${deviceId}`)
    : Promise.resolve(null);
}

export function getDevices(): Promise<Device[]> {
  return backendJson<Device[]>("api/devices");
}

export async function createDevice(name: string, location: string) {
  const response = await backendRequest("api/devices", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name, location }),
  });
  if (await response.text() !== "OK") {
    throw new Error("Failed to create device");
  }
}

export async function deleteDevice(device: Device) {
  const response = await backendRequest("api/devices", {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(device),
  });
  if (await response.text() !== "OK") {
    throw new Error("Failed to delete device");
  }
}
