export interface MeasurementStats {
  min: number;
  max: number;
  count: number;
  avg: number;
  stddev: number;
  variance: number;
}

export async function getMeasurementStats(
  device_id: number,
  sensor_id: number,
): Promise<MeasurementStats> {
  const stats: MeasurementStats = await fetch(
    `${
      Deno.env.get("HEMRS_URL")
    }api/devices/${device_id}/sensors/${sensor_id}/measurements/stats`,
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
  return stats;
}
