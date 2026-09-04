import { define } from "../../../../../../../utils.ts";
import { fetchMeasurementStream } from "../../../../../../../lib/measurement_stream.ts";

const STREAM_HEADERS = {
  "Cache-Control": "no-cache",
  "Content-Type": "text/event-stream",
  "X-Accel-Buffering": "no",
};

export async function proxyMeasurementStream(
  request: Request,
  deviceIdParam: string,
  sensorIdParam: string,
  backendUrl?: string,
): Promise<Response> {
  const deviceId = Number(deviceIdParam);
  const sensorId = Number(sensorIdParam);
  if (
    !Number.isSafeInteger(deviceId) || deviceId <= 0 ||
    !Number.isSafeInteger(sensorId) || sensorId <= 0
  ) {
    return new Response("Invalid device or sensor ID", { status: 400 });
  }
  if (!backendUrl) {
    return new Response("Backend URL is not configured", { status: 503 });
  }

  let upstream: Response;
  try {
    upstream = await fetchMeasurementStream(
      deviceId,
      sensorId,
      backendUrl,
      request.signal,
    );
  } catch (error) {
    if (request.signal.aborted) {
      return new Response(null, { status: 499 });
    }
    console.error("Failed to connect to measurement stream:", error);
    return new Response("Failed to connect to backend stream", { status: 502 });
  }

  if (!upstream.ok) {
    return new Response(upstream.body, {
      status: upstream.status,
      statusText: upstream.statusText,
      headers: upstream.headers,
    });
  }

  return new Response(upstream.body, {
    status: upstream.status,
    headers: STREAM_HEADERS,
  });
}

export const handler = define.handlers({
  GET(ctx) {
    return proxyMeasurementStream(
      ctx.req,
      ctx.params.device_id,
      ctx.params.sensor_id,
      Deno.env.get("HEMRS_URL"),
    );
  },
});
