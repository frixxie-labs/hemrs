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
    <div class="bg-dark-card border border-dark-border rounded-xl p-6">
      <div class="block sm:hidden space-y-3">
        {sensors.map((sensor) => {
          const content = (
            <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
              <div class="flex justify-between items-start mb-2">
                <h3 class="font-semibold text-text-primary">{sensor.name}</h3>
                <span class="text-sm text-text-muted">#{sensor.id}</span>
              </div>
              <p class="text-text-secondary text-sm">
                <span class="font-medium">Unit:</span> {sensor.unit}
              </p>
            </div>
          );
          return clickable
            ? (
              <a
                key={sensor.id}
                href={`/devices/${device_id}/sensors/${sensor.id}`}
                class="block"
              >
                {content}
              </a>
            )
            : <div key={sensor.id}>{content}</div>;
        })}
      </div>

      <div class="hidden sm:block overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="border-b border-dark-border text-text-muted text-xs uppercase tracking-wider">
              <th class="px-4 py-3 text-left font-medium">Sensor ID</th>
              <th class="px-4 py-3 text-left font-medium">Name</th>
              <th class="px-4 py-3 text-left font-medium">Unit</th>
            </tr>
          </thead>
          <tbody>
            {sensors.map((sensor) =>
              clickable
                ? (
                  <tr
                    key={sensor.id}
                    class="border-b border-dark-border hover:bg-table-row-hover transition-colors"
                  >
                    <td colSpan={3} class="p-0">
                      <a
                        href={`/devices/${device_id}/sensors/${sensor.id}`}
                        class="flex cursor-pointer"
                      >
                        <span class="px-4 py-3 text-text-muted flex-1">
                          #{sensor.id}
                        </span>
                        <span class="px-4 py-3 text-text-primary flex-1">
                          {sensor.name}
                        </span>
                        <span class="px-4 py-3 text-text-secondary flex-1">
                          {sensor.unit}
                        </span>
                      </a>
                    </td>
                  </tr>
                )
                : (
                  <tr
                    key={sensor.id}
                    class="border-b border-dark-border hover:bg-table-row-hover transition-colors"
                  >
                    <td class="px-4 py-3 text-text-muted">#{sensor.id}</td>
                    <td class="px-4 py-3 text-text-primary">{sensor.name}</td>
                    <td class="px-4 py-3 text-text-secondary">{sensor.unit}</td>
                  </tr>
                )
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
