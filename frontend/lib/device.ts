export interface Device {
  id: number;
  name: string;
  location: string;
}

export async function getDeviceById(device_id: number): Promise<Device | null> {
  if (!device_id) {
    return null;
  }

  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/devices/${device_id}`,
  );

  if (!response.ok) {
    console.error("Failed to fetch device:", response.statusText);
    return null;
  }

  const device: Device = await response.json();
  return device;
}

export async function getDevices(): Promise<Device[]> {
  const devices: Device[] = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/devices`,
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
  return devices;
}

export async function createDevice(name: string, location: string) {
  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/devices`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name, location }),
    },
  );
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const text = await response.text();
  if (text !== "OK") {
    throw new Error("Failed to create device");
  }
}

export async function deleteDevice(device: Device) {
  const response = await fetch(
    `${Deno.env.get("HEMRS_URL")}api/devices`,
    {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(device),
    },
  );
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const text = await response.text();
  if (text !== "OK") {
    throw new Error("Failed to delete device");
  }
}
