import {
  fetchPlotSvg,
  getTodayDeviceSensorMeasurementsPlot,
} from "../lib/plotter.ts";

const SVG_CONTENT =
  '<svg xmlns="http://www.w3.org/2000/svg"><circle r="10"/></svg>';

function mockFetch(
  response: { ok: boolean; status?: number; body?: string },
): () => void {
  const original = globalThis.fetch;
  globalThis.fetch = (_input: string | URL | Request) => {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(response.body ?? "");
    return Promise.resolve(
      new Response(bytes, {
        status: response.status ?? (response.ok ? 200 : 500),
      }),
    );
  };
  return () => {
    globalThis.fetch = original;
  };
}

Deno.test("fetchPlotSvg returns base64 data URI for valid SVG", async () => {
  const restore = mockFetch({ ok: true, body: SVG_CONTENT });
  try {
    const result = await fetchPlotSvg("plot/test");
    if (!result) {
      throw new Error("Expected a data URI, got null");
    }
    if (!result.startsWith("data:image/svg+xml;base64,")) {
      throw new Error(
        `Expected data URI prefix, got: ${result.substring(0, 40)}`,
      );
    }
    const base64 = result.replace("data:image/svg+xml;base64,", "");
    const decoded = atob(base64);
    if (decoded !== SVG_CONTENT) {
      throw new Error(`Decoded content mismatch: ${decoded}`);
    }
  } finally {
    restore();
  }
});

Deno.test("fetchPlotSvg returns null on non-ok response", async () => {
  const restore = mockFetch({ ok: false, status: 404 });
  try {
    const result = await fetchPlotSvg("plot/missing");
    if (result !== null) {
      throw new Error(`Expected null, got: ${result}`);
    }
  } finally {
    restore();
  }
});

Deno.test("fetchPlotSvg returns null on fetch error", async () => {
  const original = globalThis.fetch;
  globalThis.fetch = () => Promise.reject(new Error("network error"));
  try {
    const result = await fetchPlotSvg("plot/fail");
    if (result !== null) {
      throw new Error(`Expected null, got: ${result}`);
    }
  } finally {
    globalThis.fetch = original;
  }
});

Deno.test("getTodayDeviceSensorMeasurementsPlot passes today start param", async () => {
  let capturedUrl = "";
  const original = globalThis.fetch;
  const encoder = new TextEncoder();
  const svgBody = '<svg xmlns="http://www.w3.org/2000/svg"></svg>';
  globalThis.fetch = (input: string | URL | Request) => {
    capturedUrl = typeof input === "string"
      ? input
      : input instanceof URL
      ? input.toString()
      : input.url;
    return Promise.resolve(
      new Response(encoder.encode(svgBody), { status: 200 }),
    );
  };
  try {
    const result = await getTodayDeviceSensorMeasurementsPlot(3, 7);
    // Should return a valid data URI
    if (!result || !result.startsWith("data:image/svg+xml;base64,")) {
      throw new Error(`Expected data URI, got: ${result}`);
    }
    // URL should target the correct device/sensor path
    if (!capturedUrl.includes("/plot/devices/3/sensors/7/measurements")) {
      throw new Error(`Unexpected URL path: ${capturedUrl}`);
    }
    // URL should contain a start query param within the last 24 hours
    const url = new URL(capturedUrl, "http://localhost");
    const startParam = url.searchParams.get("start");
    if (!startParam) {
      throw new Error(`Expected start query param, got: ${capturedUrl}`);
    }
    const startTime = new Date(startParam).getTime();
    const expected = Date.now() - 24 * 60 * 60 * 1000;
    if (Math.abs(startTime - expected) > 5000) {
      throw new Error(
        `Expected start param ~24h ago, got: ${startParam}`,
      );
    }
  } finally {
    globalThis.fetch = original;
  }
});
