import { handler } from "../routes/ping.ts";

Deno.test("Ping endpoint should return pong", async () => {
  const response = await handler.GET();
  if (response.status !== 200) {
    throw new Error(`Expected status 200, got ${response.status}`);
  }
  const text = await response.text();
  if (text !== "pong") {
    throw new Error(`Expected "pong", got "${text}"`);
  }
});
