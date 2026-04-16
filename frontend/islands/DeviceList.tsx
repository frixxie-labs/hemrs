import { Device } from "../lib/device.ts";

interface DeviceListProps {
  devices: Device[];
}

export default function DeviceList({ devices }: DeviceListProps) {
  return (
    <div class="bg-slate-100 p-2 sm:p-4 rounded shadow-md mt-6 w-full">
      {/* Mobile-first card layout for small screens */}
      <div class="block sm:hidden space-y-3">
        {devices.map((device) => (
          <div
            key={device.id}
            class="bg-white p-4 rounded-lg border hover:bg-gray-50 cursor-pointer"
            onClick={() => {
              globalThis.location.href = `/devices/${device.id}`;
            }}
          >
            <div class="flex justify-between items-start mb-2">
              <h3 class="font-semibold text-lg">{device.name}</h3>
              <span class="text-sm text-gray-500">ID: {device.id}</span>
            </div>
            <p class="text-gray-600 text-sm">
              <span class="font-medium">Location:</span> {device.location}
            </p>
          </div>
        ))}
      </div>

      {/* Table layout for larger screens */}
      <div class="hidden sm:block overflow-x-auto">
        <table class="min-w-full bg-white">
          <thead>
            <tr class="w-full bg-gray-200">
              <th class="px-4 py-2 text-left">Device ID</th>
              <th class="px-4 py-2 text-left">Name</th>
              <th class="px-4 py-2 text-left">Location</th>
            </tr>
          </thead>
          <tbody
            onClick={(e) => {
              const row = (e.target as Element)?.closest("tr");
              const deviceId = row?.querySelector("td")?.textContent;
              if (deviceId) {
                globalThis.location.href = `/devices/${deviceId}`;
              }
            }}
          >
            {devices.map((device) => (
              <tr
                key={device.id}
                class="border-b hover:bg-gray-200 cursor-pointer"
              >
                <td class="px-4 py-2">{device.id}</td>
                <td class="px-4 py-2">{device.name}</td>
                <td class="px-4 py-2">{device.location}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
