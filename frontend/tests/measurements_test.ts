import {
  createMeasurement,
  getAllLatestMeasurements,
  getLatestMeasurement,
  getLatestMeasurementByDeviceAndSensorId,
  getMeasurementCount,
} from "../lib/measurements.ts";
import { installMockFetch, okTextResponse } from "./mock_fetch.ts";

const MOCK_MEASUREMENT = {
  timestamp: "2025-01-15T12:00:00Z",
  value: 22.5,
  unit: "°C",
  device_name: "Living Room Sensor",
  device_location: "Living Room",
  sensor_name: "Temperature",
};

const MOCK_MEASUREMENTS = [
  MOCK_MEASUREMENT,
  {
    timestamp: "2025-01-15T12:00:00Z",
    value: 55.0,
    unit: "%",
    device_name: "Living Room Sensor",
    device_location: "Living Room",
    sensor_name: "Humidity",
  },
];

Deno.test("getLatestMeasurement returns a measurement object", async () => {
  const restore = installMockFetch({
    "api/measurements/latest": MOCK_MEASUREMENT,
  });
  try {
    const measurement = await getLatestMeasurement();
    if (measurement === null) {
      throw new Error("Expected a measurement object, but got null");
    }
    if (
      typeof measurement.timestamp !== "string" ||
      typeof measurement.value !== "number" ||
      typeof measurement.unit !== "string" ||
      typeof measurement.device_name !== "string" ||
      typeof measurement.device_location !== "string" ||
      typeof measurement.sensor_name !== "string"
    ) {
      throw new Error(
        "Measurement object does not have the expected structure",
      );
    }
  } finally {
    restore();
  }
});

Deno.test("getLatestMeasurementByDeviceAndSensorId returns a measurement object", async () => {
  const restore = installMockFetch({
    "POST api/measurements": () => okTextResponse(),
    "measurements/latest": MOCK_MEASUREMENT,
  });
  try {
    await createMeasurement(1, 1, 25.5);

    const measurement = await getLatestMeasurementByDeviceAndSensorId(1, 1);
    if (measurement === null) {
      throw new Error("Expected a measurement object, but got null");
    }
    if (
      typeof measurement.timestamp !== "string" ||
      typeof measurement.value !== "number" ||
      typeof measurement.unit !== "string" ||
      typeof measurement.device_name !== "string" ||
      typeof measurement.device_location !== "string" ||
      typeof measurement.sensor_name !== "string"
    ) {
      throw new Error(
        "Measurement object does not have the expected structure",
      );
    }
  } finally {
    restore();
  }
});

Deno.test("getAllLatestMeasurements returns an array of measurement objects", async () => {
  const restore = installMockFetch({
    "api/measurements/latest/all": MOCK_MEASUREMENTS,
  });
  try {
    const measurements = await getAllLatestMeasurements();
    if (!Array.isArray(measurements)) {
      throw new Error("Expected an array of measurements");
    }
    if (measurements.length === 0) {
      throw new Error("Expected at least one measurement");
    }
    measurements.forEach((measurement) => {
      if (
        typeof measurement.timestamp !== "string" ||
        typeof measurement.value !== "number" ||
        typeof measurement.unit !== "string" ||
        typeof measurement.device_name !== "string" ||
        typeof measurement.device_location !== "string" ||
        typeof measurement.sensor_name !== "string"
      ) {
        throw new Error(
          "Measurement object does not have the expected structure",
        );
      }
    });
  } finally {
    restore();
  }
});

Deno.test("getMeasurementCount returns a number", async () => {
  const restore = installMockFetch({
    "api/measurements/count": 42,
  });
  try {
    const count = await getMeasurementCount();
    if (typeof count !== "number") {
      throw new Error("Expected a number for measurement count");
    }
    if (count < 0) {
      throw new Error("Measurement count should not be negative");
    }
  } finally {
    restore();
  }
});
