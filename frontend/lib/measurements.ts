export interface Measurement {
  timestamp: string;
  value: number;
  unit: string;
  device_name: string;
  device_location: string;
  sensor_name: string;
}

export async function getLatestMeasurement(): Promise<Measurement> {
  const measurement: Measurement = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/measurements/latest`,
  ).then(
    (response) => {
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      return response.json();
    },
  ).catch((error) => {
    console.error("There has been a problem with your fetch operation:", error);
    return null;
  });
  return measurement;
}

export async function getLatestMeasurementByDeviceAndSensorId(
  device_id: number,
  sensor_id: number,
): Promise<Measurement> {
  const measurement: Measurement = await fetch(
    `${
      Deno.env.get("HEMRS_URL")
    }api/devices/${device_id}/sensors/${sensor_id}/measurements/latest`,
  ).then(
    (response) => {
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      return response.json();
    },
  ).catch((error) => {
    console.error("There has been a problem with your fetch operation:", error);
    return null;
  });
  return measurement;
}

export async function getAllLatestMeasurements(): Promise<Measurement[]> {
  const measurements: Measurement[] = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/measurements/latest/all`,
  ).then(
    (response) => {
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      return response.json();
    },
  ).catch((error) => {
    console.error("There has been a problem with your fetch operation:", error);
    return [];
  });
  return measurements;
}

export async function getMeasurementCount(): Promise<number> {
  const count: number = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/measurements/count`,
  ).then(
    (response) => {
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      return response.json();
    },
  ).catch((error) => {
    console.error("There has been a problem with your fetch operation:", error);
    return 0;
  });
  return count;
}

export async function createMeasurement(
  device: number,
  sensor: number,
  measurement: number,
) {
  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/measurements`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ device, sensor, measurement }),
    },
  );
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  await response.text();
}
