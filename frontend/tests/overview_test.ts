import { getOverviewSelection, overviewPageUrl } from "../lib/overview.ts";
import type { Measurement } from "../lib/measurements.ts";

const readings: Measurement[] = Array.from({ length: 14 }, (_, i) => ({
  timestamp: "2026-09-29T12:00:00Z",
  value: i,
  unit: "°C",
  device_name: `room-${i}`,
  device_location: "Home",
  sensor_name: "temperature",
}));
readings.push({ ...readings[0], sensor_name: "BTC/USD & price", unit: "USD" });

Deno.test("overview defaults to a comparable sensor and bounds device pages", () => {
  const selection = getOverviewSelection(readings, null, null);
  if (selection.selected?.name !== "temperature" || selection.pages !== 3) {
    throw new Error("Expected the most widely shared sensor and three pages");
  }
  for (
    const [input, expected] of [["-1", 1], ["bad", 1], ["2", 2], [
      "999",
      3,
    ]] as const
  ) {
    if (
      getOverviewSelection(readings, "temperature", input).page !== expected
    ) {
      throw new Error(`Unexpected page for ${input}`);
    }
  }
  const other = getOverviewSelection(readings, "BTC/USD & price", "2");
  if (other.selected?.unit !== "USD" || other.page !== 1 || other.pages !== 1) {
    throw new Error("Sensor changes must reset an out-of-range page");
  }
});

Deno.test("overview handles empty data and duplicate device names", () => {
  if (getOverviewSelection([], null, null).selected !== undefined) {
    throw new Error("Empty data must not select a sensor");
  }
  const selection = getOverviewSelection(
    [
      readings[0],
      readings[0],
      { ...readings[0], device_location: "Office" },
    ],
    "invalid",
    null,
  );
  if (selection.selected?.count !== 2) {
    throw new Error(
      "Count unique name/location pairs and recover invalid sensors",
    );
  }
});

Deno.test("overview paging preserves sensor names with reserved characters", () => {
  const url = new URL(
    overviewPageUrl("BTC/USD & price", 2),
    "http://localhost",
  );
  if (
    url.searchParams.get("sensor") !== "BTC/USD & price" ||
    url.searchParams.get("plot_page") !== "2" ||
    url.hash !== "#overview-plots"
  ) {
    throw new Error("Invalid comparison page link");
  }
});
