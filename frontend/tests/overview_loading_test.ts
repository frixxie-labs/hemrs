import { App } from "fresh";
import { handler } from "../routes/api/overview/[resource].ts";
import type { State } from "../utils.ts";
import { installMockFetch } from "./mock_fetch.ts";

const app = new App<State>().get("/api/overview/:resource", handler.GET);
const handle = app.handler();
const request = (path: string) =>
  handle(new Request(`http://frontend.test/api/overview/${path}`));

Deno.test("overview data and latest plot finish while history plot is pending", async () => {
  const history = Promise.withResolvers<Response>();
  const started = Promise.withResolvers<void>();
  const restore = installMockFetch({
    "plot/measurements/range": () => {
      started.resolve();
      return history.promise;
    },
    "plot/measurements/latest/all": () => new Response("<svg/>"),
    "api/devices": [{ id: 1, name: "Office", location: "Upstairs" }],
  });
  let pending: Promise<Response> | undefined;
  try {
    pending = request("history-plot?sensor=Temperature&page=1");
    await started.promise;
    const devices = await request("devices");
    const plot = await request("latest-plot?sensor=Temperature&page=1");
    if (devices.status !== 200 || (await devices.json()).length !== 1) {
      throw new Error("Expected device data without waiting for history");
    }
    if (
      plot.status !== 200 ||
      !(await plot.json()).startsWith("data:image/svg+xml;base64,")
    ) {
      throw new Error("Expected latest plot without waiting for history");
    }
  } finally {
    history.resolve(new Response("<svg/>"));
    await (await pending)?.body?.cancel();
    restore();
  }
});

Deno.test("overview reports backend failures rather than empty or zero data", async () => {
  const restore = installMockFetch({
    "api/measurements/count": () =>
      new Response("Unavailable", { status: 503 }),
  });
  try {
    const response = await request("count");
    if (response.status !== 502) throw new Error("Expected an upstream error");
    await response.body?.cancel();
  } finally {
    restore();
  }
});

Deno.test("overview rejects unknown resources and invalid plot comparisons", async () => {
  const original = globalThis.fetch;
  globalThis.fetch = () => {
    throw new Error("Invalid requests must not reach upstream services");
  };
  try {
    for (
      const [path, status] of [
        ["unknown", 404],
        ["latest-plot", 400],
        ["history-plot?sensor=Temperature&page=-1", 400],
        ["history-plot?sensor=Temperature&page=1.5", 400],
      ] as const
    ) {
      const response = await request(path);
      if (response.status !== status) {
        throw new Error(`Unexpected status for ${path}`);
      }
      await response.body?.cancel();
    }
  } finally {
    globalThis.fetch = original;
  }
});
