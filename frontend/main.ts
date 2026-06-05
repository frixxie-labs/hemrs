import { App, staticFiles } from "fresh";
import type { State } from "./utils.ts";

export const app = new App<State>();

app.use(async (ctx) => {
  const url = new URL(ctx.req.url);
  const timestamp = new Date().toISOString();
  const started = performance.now();

  try {
    const res = await ctx.next();
    const elapsed = performance.now() - started;
    console.log(
      `${timestamp} ${ctx.req.method} ${url.pathname} ${res.status} ${
        elapsed.toFixed(1)
      }ms`,
    );
    return res;
  } catch (err) {
    const elapsed = performance.now() - started;
    console.log(
      `${timestamp} ${ctx.req.method} ${url.pathname} error ${
        elapsed.toFixed(1)
      }ms`,
    );
    throw err;
  }
});

app.use(staticFiles());
app.fsRoutes();
