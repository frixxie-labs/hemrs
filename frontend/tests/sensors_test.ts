import {
  createSensor,
  deleteSensor,
  getSensors,
  getSensorsByDeviceId,
} from "../lib/sensor.ts";
import { installMockFetch, okTextResponse } from "./mock_fetch.ts";

const MOCK_SENSORS = [
  { id: 1, name: "Temperature", unit: "°C" },
  { id: 2, name: "Humidity", unit: "%" },
];

Deno.test("getSensors returns an array of sensors", async () => {
  const restore = installMockFetch({ "api/sensors": MOCK_SENSORS });
  try {
    const sensors = await getSensors();
    if (!Array.isArray(sensors)) {
      throw new Error("Expected an array of sensors");
    }
    if (sensors.length === 0) {
      throw new Error("Expected at least one sensor");
    }
    sensors.forEach((sensor) => {
      if (
        typeof sensor.id !== "number" || typeof sensor.name !== "string" ||
        typeof sensor.unit !== "string"
      ) {
        throw new Error("Sensor object does not have the expected structure");
      }
    });
  } finally {
    restore();
  }
});

Deno.test("Should get a sensor by ID", async () => {
  const restore = installMockFetch({ "api/sensors": MOCK_SENSORS });
  try {
    const sensors = await getSensors();
    if (sensors.length === 0) {
      throw new Error("No sensors found to test");
    }

    const sensorId = sensors[0].id;
    const sensor = sensors.find((s) => s.id === sensorId);

    if (!sensor) {
      throw new Error(`Sensor with ID ${sensorId} not found`);
    }
    if (!sensor.name || !sensor.unit) {
      throw new Error("Sensor is missing name or unit");
    }
  } finally {
    restore();
  }
});

Deno.test("Should get sensors by device ID", async () => {
  const restore = installMockFetch({
    "api/devices/1/sensors": MOCK_SENSORS,
  });
  try {
    const deviceId = 1;
    const sensors = await getSensorsByDeviceId(deviceId);
    if (!Array.isArray(sensors)) {
      throw new Error("Expected an array of sensors");
    }
    if (sensors.length === 0) {
      throw new Error("Expected at least one sensor for the given device ID");
    }
    sensors.forEach((sensor) => {
      if (
        typeof sensor.id !== "number" || typeof sensor.name !== "string" ||
        typeof sensor.unit !== "string"
      ) {
        throw new Error("Sensor object does not have the expected structure");
      }
    });
  } finally {
    restore();
  }
});

Deno.test("Should create and delete a sensor", async () => {
  const sensorName = `Test Sensor ${Date.now()}`;
  const sensorUnit = "Test Unit";

  const sensorsAfterCreate = [
    ...MOCK_SENSORS,
    { id: 3, name: sensorName, unit: sensorUnit },
  ];

  const restore = installMockFetch({
    "POST api/sensors": () => okTextResponse(),
    "DELETE api/sensors": () => okTextResponse(),
    "api/sensors": sensorsAfterCreate,
  });
  try {
    await createSensor(sensorName, sensorUnit);

    const sensors = await getSensors();
    const createdSensor = sensors.find(
      (sensor) => sensor.name === sensorName && sensor.unit === sensorUnit,
    );
    if (!createdSensor) {
      throw new Error("Sensor was not created successfully");
    }

    await deleteSensor(createdSensor);
  } finally {
    restore();
  }
});
