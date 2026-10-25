import type { Device } from "./device.ts";
import type { Measurement } from "./measurements.ts";
import type { Sensor } from "./sensor.ts";
import { backendJson } from "./backend_client.ts";

export interface OverviewResources {
  devices: Device[];
  sensors: Sensor[];
  count: number;
  latest: Measurement[];
}

const paths: Record<keyof OverviewResources, string> = {
  devices: "api/devices",
  sensors: "api/sensors",
  count: "api/measurements/count",
  latest: "api/measurements/latest/all",
};

export function isOverviewResource(
  value: string,
): value is keyof OverviewResources {
  return Object.hasOwn(paths, value);
}

export async function getOverviewResource<K extends keyof OverviewResources>(
  resource: K,
  signal: AbortSignal,
): Promise<OverviewResources[K]> {
  return await backendJson<OverviewResources[K]>(paths[resource], signal);
}
