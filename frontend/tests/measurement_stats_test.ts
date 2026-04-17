import { getMeasurementStats } from "../lib/measurement_stats.ts";
import { installMockFetch, okTextResponse } from "./mock_fetch.ts";

const MOCK_STATS = {
  min: 10.5,
  max: 20.3,
  count: 3,
  avg: 15.5,
  stddev: 4.0,
  variance: 16.0,
};

Deno.test("should get measurement stats", async () => {
  const restore = installMockFetch({
    "POST api/measurements": () => okTextResponse(),
    "measurements/stats": MOCK_STATS,
  });
  try {
    const stats = await getMeasurementStats(1, 1);

    if (!stats) {
      throw new Error("Failed to fetch measurement stats");
    }

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
  } finally {
    restore();
  }
});
