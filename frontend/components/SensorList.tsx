import { Sensor } from "../lib/sensor.ts";

interface SensorListProps {
  sensors: Sensor[];
  device_id?: number;
}

export default function SensorList(
  { sensors, device_id }: SensorListProps,
) {
  const clickable = device_id !== undefined;

  return (
    <div class="bg-slate-100 p-2 sm:p-4 rounded shadow-md mt-6 w-full">
      {/* Mobile-first card layout for small screens */}
      <div class="block sm:hidden space-y-3">
        {sensors.map((sensor) => {
          const content = (
            <div class="bg-white p-4 rounded-lg border hover:bg-gray-50">
              <div class="flex justify-between items-start mb-2">
                <h3 class="font-semibold text-lg">{sensor.name}</h3>
                <span class="text-sm text-gray-500">ID: {sensor.id}</span>
              </div>
              <p class="text-gray-600 text-sm">
                <span class="font-medium">Unit:</span> {sensor.unit}
              </p>
            </div>
          );
          return clickable
            ? (
              <a
                key={sensor.id}
                href={`/devices/${device_id}/sensors/${sensor.id}`}
                class="block cursor-pointer"
              >
                {content}
              </a>
            )
            : <div key={sensor.id}>{content}</div>;
        })}
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
          <tbody>
            {sensors.map((sensor) => {
              const cells = (
                <>
                  <td class="px-4 py-2">{sensor.id}</td>
                  <td class="px-4 py-2">{sensor.name}</td>
                  <td class="px-4 py-2">{sensor.unit}</td>
                </>
              );
              return clickable
                ? (
                  <tr key={sensor.id} class="border-b hover:bg-gray-100">
                    <td colSpan={3} class="p-0">
                      <a
                        href={`/devices/${device_id}/sensors/${sensor.id}`}
                        class="flex cursor-pointer"
                      >
                        <span class="px-4 py-2 flex-1">{sensor.id}</span>
                        <span class="px-4 py-2 flex-1">{sensor.name}</span>
                        <span class="px-4 py-2 flex-1">{sensor.unit}</span>
                      </a>
                    </td>
                  </tr>
                )
                : (
                  <tr key={sensor.id} class="border-b">
                    {cells}
                  </tr>
                );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
