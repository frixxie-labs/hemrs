import { useEffect, useState } from "preact/hooks";
import MeasurementStatCard from "../components/MeasurementStatCard.tsx";
import type { MeasurementStats } from "../lib/measurement_stats.ts";
import type { Measurement } from "../lib/measurements.ts";
import {
  measurementStreamPath,
  newestMeasurement,
  parseMeasurementUpdate,
} from "../lib/measurement_stream.ts";

interface LiveMeasurementStatCardProps {
  deviceId: number;
  sensorId: number;
  initialLatest: Measurement;
  measurementStats: MeasurementStats;
}

type ConnectionStatus = "connecting" | "live" | "reconnecting";

export default function LiveMeasurementStatCard(
  props: LiveMeasurementStatCardProps,
) {
  const [latest, setLatest] = useState(props.initialLatest);
  const [status, setStatus] = useState<ConnectionStatus>("connecting");

  useEffect(() => {
    const source = new EventSource(
      measurementStreamPath(props.deviceId, props.sensorId),
    );
    const handleMeasurement = (event: Event) => {
      const update = parseMeasurementUpdate((event as MessageEvent).data);
      if (
        update?.device_id === props.deviceId &&
        update.sensor_id === props.sensorId
      ) {
        setLatest((current) => newestMeasurement(current, update.measurement));
      }
    };

    source.addEventListener("measurement", handleMeasurement);
    source.onopen = () => setStatus("live");
    source.onerror = () => setStatus("reconnecting");

    return () => {
      source.removeEventListener("measurement", handleMeasurement);
      source.close();
    };
  }, [props.deviceId, props.sensorId]);

  return (
    <div class="space-y-2">
      <div class="flex justify-end" aria-live="polite">
        <span class="inline-flex items-center gap-2 rounded-full border border-dark-border bg-dark-card-inner px-3 py-1 text-xs text-text-secondary">
          <span
            class={`h-2 w-2 rounded-full ${
              status === "live" ? "bg-green-400" : "bg-yellow-400"
            }`}
          />
          {status === "live"
            ? "Live"
            : status === "connecting"
            ? "Connecting"
            : "Reconnecting"}
        </span>
      </div>
      <MeasurementStatCard
        measurement_stats={props.measurementStats}
        latest={latest}
      />
    </div>
  );
}
