import { define } from "../../../utils.ts";
import {
  getOverviewResource,
  isOverviewResource,
} from "../../../lib/overview_data.ts";
import {
  getLatestAllPlot,
  getRecentMeasurementsPlot,
} from "../../../lib/plotter.ts";

export const handler = define.handlers({
  async GET(ctx) {
    const resource = ctx.params.resource;
    const headers = { "Cache-Control": "no-store" };
    try {
      if (isOverviewResource(resource)) {
        return Response.json(
          await getOverviewResource(resource, ctx.req.signal),
          { headers },
        );
      }
      if (resource !== "history-plot" && resource !== "latest-plot") {
        return new Response("Not found", { status: 404 });
      }
      const sensor = ctx.url.searchParams.get("sensor");
      const page = Number(ctx.url.searchParams.get("page") ?? "1");
      if (!sensor || !Number.isSafeInteger(page) || page < 1) {
        return new Response("Invalid comparison", { status: 400 });
      }
      const comparison = { sensor, page };
      const svg = resource === "history-plot"
        ? await getRecentMeasurementsPlot(24, Date.now(), comparison)
        : await getLatestAllPlot(comparison);
      if (!svg) throw new Error("Plot unavailable");
      return Response.json(svg, { headers });
    } catch (error) {
      console.error(`Overview ${resource} request failed:`, error);
      return Response.json({ error: "Data unavailable" }, {
        status: 502,
        headers,
      });
    }
  },
});
