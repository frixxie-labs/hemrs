export interface Sensor {
  id: number;
  name: string;
  unit: string;
}

export async function getSensors(): Promise<Sensor[]> {
  const sensors: Sensor[] = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/sensors`,
  )
    .then((response) => response.json())
    .catch((error) => {
      console.error("Error fetching sensors:", error);
      return [];
    });
  return sensors;
}

export async function getSensorById(sensorId: number): Promise<Sensor | null> {
  if (!sensorId) {
    console.error("Sensor ID is not provided.");
    return null;
  }

  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/sensors/${sensorId}`,
  );

  if (!response.ok) {
    console.error("Failed to fetch sensor:", response.statusText);
    return null;
  }

  const sensor: Sensor = await response.json();
  return sensor;
}

export async function getSensorsByDeviceId(
  deviceId: number,
): Promise<Sensor[]> {
  const sensors: Sensor[] = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/devices/${deviceId}/sensors`,
  )
    .then((response) => response.json())
    .catch((error) => {
      console.error("Error fetching sensors by device ID:", error);
      return [];
    });
  return sensors;
}

export async function createSensor(
  name: string,
  unit: string,
) {
  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/sensors`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name, unit }),
    },
  );
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const text = await response.text();
  if (text !== "OK") {
    throw new Error("Failed to create sensor");
  }
}

export async function deleteSensor(sensor: Sensor) {
  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/sensors`,
    {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(sensor),
    },
  );
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const text = await response.text();
  if (text !== "OK") {
    throw new Error("Failed to delete sensor");
  }
}
