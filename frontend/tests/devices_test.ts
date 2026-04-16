import { createDevice, deleteDevice, getDevices } from "../lib/device.ts";

Deno.test("Should get devices", async () => {
  const devices = await getDevices();
  if (devices.length === 0) {
    throw new Error("No devices found");
  }
  devices.forEach((device) => {
    if (!device.id || !device.name) {
      throw new Error("Device is missing id or name");
    }
  });
});

Deno.test("Should get a device by ID", async () => {
  const devices = await getDevices();
  if (devices.length === 0) {
    throw new Error("No devices found to test");
  }

  const deviceId = devices[0].id;
  const device = await getDevices().then((d) =>
    d.find((d) => d.id === deviceId)
  );

  if (!device) {
    throw new Error(`Device with ID ${deviceId} not found`);
  }
  if (!device.name || !device.location) {
    throw new Error("Device is missing name or location");
  }
});

Deno.test("Should create and delete a device", async () => {
  const deviceName = `Test Device ${Date.now()}`;
  const deviceLocation = "Test Location";

  // Create a new device
  await createDevice(deviceName, deviceLocation);

  const devices = await getDevices();

  const createdDevice = devices.find(
    (device) =>
      device.name === deviceName && device.location === deviceLocation,
  );
  if (!createdDevice) {
    throw new Error("Device was not created successfully");
  }
  // Delete the created device
  await deleteDevice(createdDevice);
});
