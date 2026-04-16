import { Sensor } from "../lib/sensor.ts";

interface SensorListProps {
  device_id: number;
  sensors: Sensor[];
}

export default function SensorList({ device_id, sensors }: SensorListProps) {
  return (
    <div class="bg-slate-100 p-2 sm:p-4 rounded shadow-md mt-6 w-full">
      {/* Mobile-first card layout for small screens */}
      <div class="block sm:hidden space-y-3">
        {sensors.map((sensor) => (
          <div
            key={sensor.id}
            class="bg-white p-4 rounded-lg border hover:bg-gray-50 cursor-pointer"
            onClick={() => {
              globalThis.location.href =
                `/devices/${device_id}/sensors/${sensor.id}`;
            }}
          >
            <div class="flex justify-between items-start mb-2">
              <h3 class="font-semibold text-lg">{sensor.name}</h3>
              <span class="text-sm text-gray-500">ID: {sensor.id}</span>
            </div>
            <p class="text-gray-600 text-sm">
              <span class="font-medium">Unit:</span> {sensor.unit}
            </p>
          </div>
        ))}
      </div>

      {/* Table layout for larger screens */}
      <div class="hidden sm:block overflow-x-auto">
        <table class="min-w-full bg-white">
          <thead>
            <tr class="w-full bg-gray-200">
              <th class="px-4 py-2 text-left">Sensor ID</th>
              <th class="px-4 py-2 text-left">Name</th>
              <th class="px-4 py-2 text-left">Unit</th>
            </tr>
          </thead>
          <tbody
            onClick={(e) => {
              const row = (e.target as Element)?.closest("tr");
              const sensor_id = row?.querySelector("td")?.textContent;
              if (sensor_id) {
                globalThis.location.href =
                  `/devices/${device_id}/sensors/${sensor_id}`;
              }
            }}
          >
            {sensors.map((sensor) => (
              <tr
                key={sensor.id}
                class="border-b hover:bg-gray-100 cursor-pointer"
              >
                <td class="px-4 py-2">{sensor.id}</td>
                <td class="px-4 py-2">{sensor.name}</td>
                <td class="px-4 py-2">{sensor.unit}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
