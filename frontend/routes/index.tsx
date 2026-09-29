import { define } from "../utils.ts";
import Overview from "../islands/Overview.tsx";

export default define.page((ctx) => (
  <>
    <Overview
      sensor={ctx.url.searchParams.get("sensor")}
      plotPage={ctx.url.searchParams.get("plot_page")}
    />
    <noscript>Enable JavaScript to load the overview data and plots.</noscript>
  </>
));
