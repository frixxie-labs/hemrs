import { Device } from "../lib/device.ts";

interface DeviceListProps {
  devices: Device[];
  clickable?: boolean;
}

export default function DeviceList(
  { devices, clickable = false }: DeviceListProps,
) {
  return (
    <div class="bg-slate-100 p-2 sm:p-4 rounded shadow-md mt-6 w-full">
      {/* Mobile-first card layout for small screens */}
      <div class="block sm:hidden space-y-3">
        {devices.map((device) => {
          const content = (
            <div class="bg-white p-4 rounded-lg border hover:bg-gray-50">
              <div class="flex justify-between items-start mb-2">
                <h3 class="font-semibold text-lg">{device.name}</h3>
                <span class="text-sm text-gray-500">ID: {device.id}</span>
              </div>
              <p class="text-gray-600 text-sm">
                <span class="font-medium">Location:</span> {device.location}
              </p>
            </div>
          );
          return clickable
            ? (
              <a
                key={device.id}
                href={`/devices/${device.id}`}
                class="block cursor-pointer"
              >
                {content}
              </a>
            )
            : <div key={device.id}>{content}</div>;
        })}
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
          <tbody>
            {devices.map((device) => {
              const cells = (
                <>
                  <td class="px-4 py-2">{device.id}</td>
                  <td class="px-4 py-2">{device.name}</td>
                  <td class="px-4 py-2">{device.location}</td>
                </>
              );
              return clickable
                ? (
                  <tr key={device.id} class="border-b hover:bg-gray-200">
                    <td colSpan={3} class="p-0">
                      <a
                        href={`/devices/${device.id}`}
                        class="flex cursor-pointer"
                      >
                        <span class="px-4 py-2 flex-1">{device.id}</span>
                        <span class="px-4 py-2 flex-1">{device.name}</span>
                        <span class="px-4 py-2 flex-1">{device.location}</span>
                      </a>
                    </td>
                  </tr>
                )
                : (
                  <tr key={device.id} class="border-b">
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
