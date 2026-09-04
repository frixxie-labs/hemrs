import {
  measurementStreamPath,
  newestMeasurement,
  parseMeasurementUpdate,
} from "../lib/measurement_stream.ts";
import { proxyMeasurementStream } from "../routes/api/devices/[device_id]/sensors/[sensor_id]/measurements/stream.ts";
import { installMockFetch } from "./mock_fetch.ts";

const UPDATE = {
  device_id: 3,
  sensor_id: 7,
  measurement: {
    timestamp: "2026-09-03T12:00:00Z",
    value: 21.5,
    unit: "C",
    device_name: "Office",
    device_location: "Upstairs",
    sensor_name: "Temperature",
  },
};

Deno.test("measurementStreamPath builds a same-origin URL", () => {
  const path = measurementStreamPath(3, 7);
  if (path !== "/api/devices/3/sensors/7/measurements/stream") {
    throw new Error(`Unexpected stream path: ${path}`);
  }
});

Deno.test("parseMeasurementUpdate validates the event payload", () => {
  const update = parseMeasurementUpdate(JSON.stringify(UPDATE));
  if (
    update?.device_id !== 3 || update.sensor_id !== 7 ||
    update.measurement.value !== 21.5
  ) {
    throw new Error("Expected a valid measurement update");
  }

  if (parseMeasurementUpdate("not JSON") !== null) {
    throw new Error("Expected malformed JSON to be rejected");
  }
  if (parseMeasurementUpdate(JSON.stringify({ ...UPDATE, sensor_id: "7" }))) {
    throw new Error("Expected an invalid sensor ID to be rejected");
  }
  if (parseMeasurementUpdate(JSON.stringify({ ...UPDATE, measurement: {} }))) {
    throw new Error("Expected an invalid measurement to be rejected");
  }
});

Deno.test("newestMeasurement ignores backfilled measurements", () => {
  const current = UPDATE.measurement;
  const older = { ...current, timestamp: "2026-09-02T12:00:00Z", value: 1 };
  const newer = { ...current, timestamp: "2026-09-04T12:00:00Z", value: 2 };

  if (newestMeasurement(current, older) !== current) {
    throw new Error("Expected an older measurement to be ignored");
  }
  if (newestMeasurement(current, newer) !== newer) {
    throw new Error(
      "Expected a newer measurement to replace the current value",
    );
  }
});

Deno.test("proxyMeasurementStream preserves the upstream SSE stream", async () => {
  const event = `event: measurement\ndata: ${JSON.stringify(UPDATE)}\n\n`;
  let requestedUrl = "";
  let accept = "";
  let upstreamSignal: AbortSignal | null | undefined;
  const encoder = new TextEncoder();
  const body = new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(encoder.encode(event));
    },
  });
  const restore = installMockFetch({
    "measurements/stream": (url: string, init?: RequestInit) => {
      requestedUrl = url;
      accept = new Headers(init?.headers).get("Accept") ?? "";
      upstreamSignal = init?.signal;
      return new Response(body, {
        headers: { "Content-Type": "text/event-stream" },
      });
    },
  });

  try {
    const request = new Request("http://frontend.test/api/stream");
    const response = await proxyMeasurementStream(
      request,
      "3",
      "7",
      "http://backend.test",
    );

    if (
      requestedUrl !==
        "http://backend.test/api/devices/3/sensors/7/measurements/stream"
    ) {
      throw new Error(`Unexpected upstream URL: ${requestedUrl}`);
    }
    if (accept !== "text/event-stream") {
      throw new Error(`Unexpected Accept header: ${accept}`);
    }
    if (response.headers.get("Content-Type") !== "text/event-stream") {
      throw new Error("Expected SSE content type");
    }
    if (response.headers.get("X-Accel-Buffering") !== "no") {
      throw new Error("Expected proxy buffering to be disabled");
    }
    if (upstreamSignal !== request.signal) {
      throw new Error("Expected downstream cancellation to be forwarded");
    }
    if (!response.body) {
      throw new Error("Expected a streaming response body");
    }
    const reader = response.body.getReader();
    const firstChunk = await reader.read();
    if (
      !firstChunk || firstChunk.done ||
      new TextDecoder().decode(firstChunk.value) !== event
    ) {
      throw new Error(
        "Expected the first event before the upstream stream closes",
      );
    }
    await reader.cancel();
  } finally {
    restore();
  }
});

Deno.test("proxyMeasurementStream validates configuration and IDs", async () => {
  const request = new Request("http://frontend.test/api/stream");
  const invalidId = await proxyMeasurementStream(
    request,
    "not-an-id",
    "7",
    "http://backend.test/",
  );
  if (invalidId.status !== 400) {
    throw new Error(`Expected 400, got ${invalidId.status}`);
  }

  const missingBackend = await proxyMeasurementStream(
    request,
    "3",
    "7",
    undefined,
  );
  if (missingBackend.status !== 503) {
    throw new Error(`Expected 503, got ${missingBackend.status}`);
  }
});
