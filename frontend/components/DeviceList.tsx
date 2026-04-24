import { Device } from "../lib/device.ts";

interface DeviceListProps {
  devices: Device[];
  clickable?: boolean;
  searchable?: boolean;
}

export default function DeviceList(
  { devices, clickable = false, searchable = false }: DeviceListProps,
) {
  return (
    <div class="bg-dark-card border border-dark-border rounded-xl p-6">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h2 class="text-xl font-bold text-text-primary">Devices</h2>
          <p class="text-text-secondary text-sm">
            {devices.length} devices shown
          </p>
        </div>
        {searchable && (
          <input
            type="text"
            placeholder="Search name, location, type, status or ID"
            class="bg-dark-card-inner border border-dark-border rounded-full px-4 py-2 text-sm text-text-primary placeholder-text-muted w-80 focus:outline-none focus:border-accent-green"
          />
        )}
      </div>

      {/* Mobile card layout */}
      <div class="block sm:hidden space-y-3">
        {devices.map((device) => {
          const content = (
            <div class="bg-dark-card-inner border border-dark-border rounded-lg p-4">
              <div class="flex justify-between items-start mb-2">
                <h3 class="font-semibold text-text-primary">{device.name}</h3>
                <span class="text-sm text-text-muted">#{device.id}</span>
              </div>
              <p class="text-text-secondary text-sm">
                <span class="font-medium">Location:</span> {device.location}
              </p>
            </div>
          );
          return clickable
            ? (
              <a
                key={device.id}
                href={`/devices/${device.id}`}
                class="block"
              >
                {content}
              </a>
            )
            : <div key={device.id}>{content}</div>;
        })}
      </div>

      {/* Table layout */}
      <div class="hidden sm:block overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="border-b border-dark-border text-text-muted text-xs uppercase tracking-wider">
              <th class="px-4 py-3 text-left font-medium">ID</th>
              <th class="px-4 py-3 text-left font-medium">Device</th>
              <th class="px-4 py-3 text-left font-medium">Location</th>
            </tr>
          </thead>
          <tbody>
            {devices.map((device) =>
              clickable
                ? (
                  <tr
                    key={device.id}
                    class="border-b border-dark-border hover:bg-table-row-hover transition-colors"
                  >
                    <td class="px-4 py-3 text-text-muted">
                      <a href={`/devices/${device.id}`} class="cursor-pointer">
                        #{device.id}
                      </a>
                    </td>
                    <td class="px-4 py-3 text-text-primary">
                      <a href={`/devices/${device.id}`} class="cursor-pointer">
                        {device.name}
                      </a>
                    </td>
                    <td class="px-4 py-3 text-text-secondary">
                      <a href={`/devices/${device.id}`} class="cursor-pointer">
                        {device.location}
                      </a>
                    </td>
                  </tr>
                )
                : (
                  <tr
                    key={device.id}
                    class="border-b border-dark-border hover:bg-table-row-hover transition-colors"
                  >
                    <td class="px-4 py-3 text-text-muted">#{device.id}</td>
                    <td class="px-4 py-3 text-text-primary">{device.name}</td>
                    <td class="px-4 py-3 text-text-secondary">
                      {device.location}
                    </td>
                  </tr>
                )
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
