import {
  createMeasurement,
  getAllLatestMeasurements,
  getLatestMeasurement,
  getLatestMeasurementByDeviceAndSensorId,
  getMeasurementCount,
} from "../lib/measurements.ts";
import { getDevices } from "../lib/device.ts";
import { getSensorsByDeviceId } from "../lib/sensor.ts";

Deno.test("getLatestMeasurement returns a measurement object", async () => {
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
    throw new Error("Measurement object does not have the expected structure");
  }
});

Deno.test("getLatestMeasurementByDeviceAndSensorId returns a measurement object", async () => {
  // Get actual devices and sensors from the database
  const devices = await getDevices();
  if (devices.length === 0) {
    throw new Error("No devices found to test");
  }

  const deviceId = devices[0].id;
  const sensors = await getSensorsByDeviceId(deviceId);
  if (sensors.length === 0) {
    throw new Error("No sensors found for the device");
  }

  const sensorId = sensors[0].id;

  // Create a test measurement to ensure data exists
  await createMeasurement(deviceId, sensorId, 25.5);

  const measurement = await getLatestMeasurementByDeviceAndSensorId(
    deviceId,
    sensorId,
  );
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
    throw new Error("Measurement object does not have the expected structure");
  }
});

Deno.test("getAllLatestMeasurements returns an array of measurement objects", async () => {
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
});

Deno.test("getMeasurementCount returns a number", async () => {
  const count = await getMeasurementCount();
  if (typeof count !== "number") {
    throw new Error("Expected a number for measurement count");
  }
  if (count < 0) {
    throw new Error("Measurement count should not be negative");
  }
});
