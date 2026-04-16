import { fetchPlotSvg } from "../lib/plotter.ts";

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
