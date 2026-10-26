/** HTTP failures are distinct from missing resources and successful empty data. */
export class BackendError extends Error {
  constructor(readonly status: number) {
    super(`Backend request failed: ${status}`);
    this.name = "BackendError";
  }
}

export async function backendRequest(
  path: string,
  init: RequestInit = {},
): Promise<Response> {
  const base = Deno.env.get("HEMRS_URL") ?? "http://localhost:65534/";
  const url = new URL(path.replace(/^\/+/, ""), `${base.replace(/\/+$/, "")}/`);
  const timeout = AbortSignal.timeout(30_000);
  const response = await fetch(url, {
    ...init,
    signal: init.signal ? AbortSignal.any([init.signal, timeout]) : timeout,
  });
  if (!response.ok) {
    await response.body?.cancel();
    throw new BackendError(response.status);
  }
  return response;
}

export async function backendJson<T>(
  path: string,
  signal?: AbortSignal,
): Promise<T> {
  const response = await backendRequest(path, { signal });
  return await response.json();
}

/** Only a 404 represents absence; transport/server failures must reach the caller. */
export async function optionalBackendJson<T>(path: string): Promise<T | null> {
  try {
    return await backendJson<T>(path);
  } catch (error) {
    if (error instanceof BackendError && error.status === 404) return null;
    throw error;
  }
}

/** Mutating endpoints acknowledge success with a plain-text `OK` body. */
export async function sendBackendCommand(
  path: string,
  method: "POST" | "PUT" | "DELETE",
  body: unknown,
  failureMessage: string,
): Promise<void> {
  const response = await backendRequest(path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (await response.text() !== "OK") {
    throw new Error(failureMessage);
  }
}
