import { getMeasurementStats } from "../lib/measurement_stats.ts";
import { getDevices } from "../lib/device.ts";
import { getSensorsByDeviceId } from "../lib/sensor.ts";
import { createMeasurement } from "../lib/measurements.ts";

Deno.test("should get measurement stats", async () => {
  // Get actual devices and sensors from the database
  const devices = await getDevices();
  if (devices.length === 0) {
    throw new Error("No devices found to test");
  }

  const device_id = devices[0].id;
  const sensors = await getSensorsByDeviceId(device_id);
  if (sensors.length === 0) {
    throw new Error("No sensors found for the device");
  }

  const sensor_id = sensors[0].id;

  // Create some test measurements to ensure data exists
  await createMeasurement(device_id, sensor_id, 10.5);
  await createMeasurement(device_id, sensor_id, 20.3);
  await createMeasurement(device_id, sensor_id, 15.7);

  const stats = await getMeasurementStats(device_id, sensor_id);

  if (!stats) {
    throw new Error("Failed to fetch measurement stats");
  }

  // Check that the stats object has the expected properties
  if (
    typeof stats.min !== "number" ||
    typeof stats.max !== "number" ||
    typeof stats.count !== "number" ||
    typeof stats.avg !== "number" ||
    typeof stats.stddev !== "number" ||
    typeof stats.variance !== "number"
  ) {
    throw new Error(
      "Measurement stats object does not have the expected structure",
    );
  }
});
