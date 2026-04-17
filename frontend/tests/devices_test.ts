import { createDevice, deleteDevice, getDevices } from "../lib/device.ts";
import { installMockFetch, okTextResponse } from "./mock_fetch.ts";

const MOCK_DEVICES = [
  { id: 1, name: "Living Room Sensor", location: "Living Room" },
  { id: 2, name: "Kitchen Sensor", location: "Kitchen" },
];

Deno.test("Should get devices", async () => {
  const restore = installMockFetch({ "api/devices": MOCK_DEVICES });
  try {
    const devices = await getDevices();
    if (devices.length === 0) {
      throw new Error("No devices found");
    }
    devices.forEach((device) => {
      if (!device.id || !device.name) {
        throw new Error("Device is missing id or name");
      }
    });
  } finally {
    restore();
  }
});

Deno.test("Should get a device by ID", async () => {
  const restore = installMockFetch({ "api/devices": MOCK_DEVICES });
  try {
    const devices = await getDevices();
    if (devices.length === 0) {
      throw new Error("No devices found to test");
    }

    const deviceId = devices[0].id;
    const device = devices.find((d) => d.id === deviceId);

    if (!device) {
      throw new Error(`Device with ID ${deviceId} not found`);
    }
    if (!device.name || !device.location) {
      throw new Error("Device is missing name or location");
    }
  } finally {
    restore();
  }
});

Deno.test("Should create and delete a device", async () => {
  const deviceName = `Test Device ${Date.now()}`;
  const deviceLocation = "Test Location";

  // After creation, the mock returns the new device in the list
  const devicesAfterCreate = [
    ...MOCK_DEVICES,
    { id: 3, name: deviceName, location: deviceLocation },
  ];

  const restore = installMockFetch({
    "POST api/devices": () => okTextResponse(),
    "DELETE api/devices": () => okTextResponse(),
    "api/devices": devicesAfterCreate,
  });
  try {
    await createDevice(deviceName, deviceLocation);

    const devices = await getDevices();
    const createdDevice = devices.find(
      (device) =>
        device.name === deviceName && device.location === deviceLocation,
    );
    if (!createdDevice) {
      throw new Error("Device was not created successfully");
    }

    await deleteDevice(createdDevice);
  } finally {
    restore();
  }
});
