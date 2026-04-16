import {
  createSensor,
  deleteSensor,
  getSensors,
  getSensorsByDeviceId,
} from "../lib/sensor.ts";

Deno.test("getSensors returns an array of sensors", async () => {
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
});

Deno.test("Should get a sensor by ID", async () => {
  const sensors = await getSensors();
  if (sensors.length === 0) {
    throw new Error("No sensors found to test");
  }

  const sensorId = sensors[0].id;
  const sensor = await getSensors().then((s) =>
    s.find((s) => s.id === sensorId)
  );

  if (!sensor) {
    throw new Error(`Sensor with ID ${sensorId} not found`);
  }
  if (!sensor.name || !sensor.unit) {
    throw new Error("Sensor is missing name or unit");
  }
});

Deno.test("Should get sensors by device ID", async () => {
  const deviceId = 1; // Replace with a valid device ID for your tests
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
});

Deno.test("Should create and delete a sensor", async () => {
  const sensorName = `Test Sensor ${Date.now()}`;
  const sensorUnit = "Test Unit";

  // Create a new sensor
  await createSensor(sensorName, sensorUnit);

  const sensors = await getSensors();

  const createdSensor = sensors.find(
    (sensor) => sensor.name === sensorName && sensor.unit === sensorUnit,
  );
  if (!createdSensor) {
    throw new Error("Sensor was not created successfully");
  }

  // Delete the created sensor
  await deleteSensor(createdSensor);
});
