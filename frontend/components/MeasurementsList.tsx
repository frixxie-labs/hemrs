import { Measurement } from "../lib/measurements.ts";

interface MeasuremetListProps {
  measurements: Measurement[];
}

export default function MeasuremetList({ measurements }: MeasuremetListProps) {
  return (
    <div class="bg-dark-card border border-dark-border rounded-xl p-6">
      <h2 class="text-xl font-bold text-text-primary mb-4">
        Measurements: {measurements.length}
      </h2>
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="border-b border-dark-border text-text-muted text-xs uppercase tracking-wider">
              <th class="px-4 py-3 text-left font-medium">Timestamp</th>
              <th class="px-4 py-3 text-left font-medium">Device name</th>
              <th class="px-4 py-3 text-left font-medium">Sensor name</th>
              <th class="px-4 py-3 text-left font-medium">Value</th>
              <th class="px-4 py-3 text-left font-medium">Unit</th>
            </tr>
          </thead>
          <tbody>
            {measurements.map((measurement) => (
              <tr
                key={measurement.timestamp}
                class="border-b border-dark-border hover:bg-table-row-hover transition-colors"
              >
                <td class="px-4 py-3 text-text-secondary">
                  {new Date(measurement.timestamp).toLocaleString()}
                </td>
                <td class="px-4 py-3 text-text-primary">
                  {measurement.device_name}
                </td>
                <td class="px-4 py-3 text-text-primary">
                  {measurement.sensor_name}
                </td>
                <td class="px-4 py-3 text-text-primary">
                  {measurement.value}
                </td>
                <td class="px-4 py-3 text-text-secondary">
                  {measurement.unit}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
