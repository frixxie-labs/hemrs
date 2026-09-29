import type { Measurement } from "../lib/measurements.ts";

interface MeasurementsListProps {
  measurements: Measurement[];
  showAllLink?: boolean;
}

function timestamp(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : date.toISOString().slice(0, 19).replace("T", " ");
}

export default function MeasurementsList(
  { measurements, showAllLink = false }: MeasurementsListProps,
) {
  return (
    <section>
      <div class="monitor-section-heading">
        <h2>Latest readings</h2>
        {showAllLink
          ? <a href="/measurements">All measurements ↗</a>
          : <span>{measurements.length} readings</span>}
      </div>
      <div class="monitor-table-wrap">
        <table class="monitor-table">
          <thead>
            <tr>
              <th scope="col">Device</th>
              <th scope="col">Location</th>
              <th scope="col">Sensor</th>
              <th scope="col" class="monitor-numeric">Value</th>
              <th scope="col">Recorded (UTC)</th>
            </tr>
          </thead>
          <tbody>
            {measurements.map((reading, index) => (
              <tr
                key={`${reading.device_name}-${reading.sensor_name}-${index}`}
              >
                <td>{reading.device_name}</td>
                <td class="monitor-secondary">{reading.device_location}</td>
                <td>{reading.sensor_name}</td>
                <td class="monitor-numeric monitor-value">
                  {reading.value} <span>{reading.unit}</span>
                </td>
                <td class="monitor-time">{timestamp(reading.timestamp)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {!measurements.length && (
        <p class="monitor-empty">No readings available.</p>
      )}
    </section>
  );
}
