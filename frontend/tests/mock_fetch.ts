/**
 * A URL-pattern-based fetch mock for tests.
 *
 * Usage:
 *   const restore = installMockFetch({ "api/devices": [{ id: 1, name: "D", location: "L" }] });
 *   // ... run code that calls fetch ...
 *   restore();
 *
 * Routes are matched by checking if the request URL ends with the given key
 * (after stripping any leading "/"). For POST/DELETE requests, the route key
 * can optionally include the method prefix like "POST api/devices".
 */

type RouteHandler =
  | unknown // JSON body returned with 200
  | ((url: string, init?: RequestInit) => Response | Promise<Response>);

export function installMockFetch(
  routes: Record<string, RouteHandler>,
): () => void {
  const original = globalThis.fetch;

  globalThis.fetch = (
    input: string | URL | Request,
    init?: RequestInit,
  ): Promise<Response> => {
    const url = typeof input === "string"
      ? input
      : input instanceof URL
      ? input.toString()
      : input.url;
    const method = init?.method ?? "GET";

    // Try method-specific route first, then generic route
    for (const key of Object.keys(routes)) {
      const methodSpecific = key.includes(" ");
      if (methodSpecific) {
        const [routeMethod, routePath] = key.split(" ", 2);
        if (
          routeMethod.toUpperCase() === method.toUpperCase() &&
          url.includes(routePath)
        ) {
          const handler = routes[key];
          if (typeof handler === "function") {
            return Promise.resolve(handler(url, init));
          }
          return Promise.resolve(
            new Response(JSON.stringify(handler), {
              status: 200,
              headers: { "Content-Type": "application/json" },
            }),
          );
        }
      }
    }

    // Generic route matching (no method prefix)
    for (const key of Object.keys(routes)) {
      if (key.includes(" ")) continue;
      if (url.includes(key)) {
        const handler = routes[key];
        if (typeof handler === "function") {
          return Promise.resolve(handler(url, init));
        }
        return Promise.resolve(
          new Response(JSON.stringify(handler), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }
    }

    return Promise.reject(
      new Error(`Mock fetch: no route matched for ${method} ${url}`),
    );
  };

  return () => {
    globalThis.fetch = original;
  };
}

/** Helper to create a plain text "OK" response (used by create/delete endpoints). */
export function okTextResponse(): Response {
  return new Response("OK", { status: 200 });
}
