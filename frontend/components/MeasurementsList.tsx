import { Measurement } from "../lib/measurements.ts";

interface MeasuremetListProps {
  measurements: Measurement[];
}

export default function MeasuremetList({ measurements }: MeasuremetListProps) {
  return (
    <div class="bg-slate-100 p-4 rounded shadow-md mt-6">
      <h2 class="text-xl font-semibold mb-4">
        Measurements: {measurements.length}
      </h2>
      <table class="min-w-full bg-white">
        <thead>
          <tr class="w-full bg-gray-200">
            <th class="px-4 py-2 text-left">Timestamp</th>
            <th class="px-4 py-2 text-left">Device name</th>
            <th class="px-4 py-2 text-left">Sensor name</th>
            <th class="px-4 py-2 text-left">Value</th>
            <th class="px-4 py-2 text-left">Unit</th>
          </tr>
        </thead>
        <tbody>
          {measurements.map((measurement) => (
            <tr key={measurement.timestamp} class="border-b">
              <td class="px-4 py-2">
                {new Date(measurement.timestamp).toLocaleString()}
              </td>
              <td class="px-4 py-2">{measurement.device_name}</td>
              <td class="px-4 py-2">{measurement.sensor_name}</td>
              <td class="px-4 py-2">{measurement.value}</td>
              <td class="px-4 py-2">{measurement.unit}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
