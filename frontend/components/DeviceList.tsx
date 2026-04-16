import { Device } from "../lib/device.ts";

interface DeviceListProps {
  devices: Device[];
}

export default function DeviceList({ devices }: DeviceListProps) {
  return (
    <div class="bg-slate-100 p-4 rounded shadow-md mt-6">
      <table class="min-w-full bg-white">
        <thead>
          <tr class="w-full bg-gray-200">
            <th class="px-4 py-2 text-left">Device ID</th>
            <th class="px-4 py-2 text-left">Name</th>
            <th class="px-4 py-2 text-left">Location</th>
          </tr>
        </thead>
        <tbody>
          {devices.map((device) => (
            <tr key={device.id} class="border-b">
              <td class="px-4 py-2">{device.id}</td>
              <td class="px-4 py-2">{device.name}</td>
              <td class="px-4 py-2">{device.location}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
