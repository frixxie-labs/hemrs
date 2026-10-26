import { getDeviceById, getDevices } from "../lib/device.ts";
import { getSensorById, getSensors } from "../lib/sensor.ts";
import { getMeasurementStats } from "../lib/measurement_stats.ts";
import {
  getAllLatestMeasurements,
  getLatestMeasurement,
  getMeasurementCount,
} from "../lib/measurements.ts";
import { installMockFetch } from "./mock_fetch.ts";
import { backendJson, sendBackendCommand } from "../lib/backend_client.ts";

Deno.test("backend helpers propagate HTTP failures instead of successful fallback data", async () => {
  const restore = installMockFetch({
    "api/": () => new Response("Unavailable", { status: 503 }),
  });
  try {
    for (
      const load of [
        getDevices,
        getSensors,
        getAllLatestMeasurements,
        getLatestMeasurement,
        getMeasurementCount,
        () => getMeasurementStats(1, 2),
        () => getDeviceById(1),
        () => getSensorById(2),
      ]
    ) {
      let failed = false;
      try {
        await load();
      } catch {
        failed = true;
      }
      if (!failed) throw new Error("Backend failure was hidden");
    }
  } finally {
    restore();
  }
});

Deno.test("backend helpers preserve empty data and distinguish missing resources", async () => {
  const restore = installMockFetch({
    "api/devices/1": () => new Response("Missing", { status: 404 }),
    "api/sensors/2": () => new Response("Missing", { status: 404 }),
    "api/devices": [],
    "api/measurements/count": 0,
  });
  try {
    if ((await getDevices()).length !== 0) {
      throw new Error("Expected empty data");
    }
    if (await getMeasurementCount() !== 0) {
      throw new Error("Expected zero count");
    }
    if (await getDeviceById(1) !== null) {
      throw new Error("Expected missing device");
    }
    if (await getSensorById(2) !== null) {
      throw new Error("Expected missing sensor");
    }
  } finally {
    restore();
  }
});

Deno.test("backend helpers normalize the base URL without losing its path prefix", async () => {
  const previous = Deno.env.get("HEMRS_URL");
  const restore = installMockFetch({
    "api/devices": (url: string) => {
      if (url !== "http://backend.test/hemrs/api/devices") {
        throw new Error(`Unexpected URL: ${url}`);
      }
      return new Response("[]");
    },
  });
  try {
    for (
      const base of ["http://backend.test/hemrs", "http://backend.test/hemrs/"]
    ) {
      Deno.env.set("HEMRS_URL", base);
      await getDevices();
    }
  } finally {
    if (previous === undefined) Deno.env.delete("HEMRS_URL");
    else Deno.env.set("HEMRS_URL", previous);
    restore();
  }
});

Deno.test("backend helpers propagate invalid JSON and connection failures", async () => {
  const restore = installMockFetch({
    "api/devices": () => new Response("not json"),
    "api/sensors": () => Promise.reject(new Error("Connection refused")),
  });
  try {
    for (const load of [getDevices, getSensors]) {
      let failed = false;
      try {
        await load();
      } catch {
        failed = true;
      }
      if (!failed) throw new Error("Expected request failure");
    }
  } finally {
    restore();
  }
});

Deno.test("backend requests honor caller cancellation", async () => {
  const controller = new AbortController();
  controller.abort();
  const restore = installMockFetch({
    "api/devices": (_url: string, init?: RequestInit) => {
      if (!init?.signal?.aborted) throw new Error("Caller signal was lost");
      return Promise.reject(init.signal.reason);
    },
  });
  try {
    let error: unknown;
    try {
      await backendJson("api/devices", controller.signal);
    } catch (failure) {
      error = failure;
    }
    if (!(error instanceof DOMException) || error.name !== "AbortError") {
      throw new Error("Expected cancellation to propagate");
    }
  } finally {
    restore();
  }
});

Deno.test("sendBackendCommand sends JSON with the given method", async () => {
  let received: RequestInit | undefined;
  const restore = installMockFetch({
    "PUT api/things": (_url: string, init?: RequestInit) => {
      received = init;
      return new Response("OK");
    },
  });
  try {
    await sendBackendCommand("api/things", "PUT", { id: 1 }, "Failed");
    if (received?.body !== JSON.stringify({ id: 1 })) {
      throw new Error("Expected JSON body");
    }
    const headers = new Headers(received?.headers);
    if (headers.get("Content-Type") !== "application/json") {
      throw new Error("Expected JSON content type");
    }
  } finally {
    restore();
  }
});

Deno.test("sendBackendCommand rejects responses other than OK", async () => {
  const restore = installMockFetch({
    "POST api/things": () => new Response("NOPE"),
  });
  try {
    let message = "";
    try {
      await sendBackendCommand("api/things", "POST", {}, "Failed to save");
    } catch (error) {
      message = (error as Error).message;
    }
    if (message !== "Failed to save") throw new Error(`Got "${message}"`);
  } finally {
    restore();
  }
});
