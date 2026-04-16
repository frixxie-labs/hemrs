import { Sensor } from "../lib/sensor.ts";

interface SensorListProps {
  sensors: Sensor[];
}

export default function SensorList({ sensors }: SensorListProps) {
  return (
    <div class="bg-slate-100 p-4 rounded shadow-md mt-6">
      <table class="min-w-full bg-white">
        <thead>
          <tr class="w-full bg-gray-200">
            <th class="px-4 py-2 text-left">Sensor ID</th>
            <th class="px-4 py-2 text-left">Name</th>
            <th class="px-4 py-2 text-left">Unit</th>
          </tr>
        </thead>
        <tbody>
          {sensors.map((sensor) => (
            <tr key={sensor.id} class="border-b">
              <td class="px-4 py-2">{sensor.id}</td>
              <td class="px-4 py-2">{sensor.name}</td>
              <td class="px-4 py-2">{sensor.unit}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
