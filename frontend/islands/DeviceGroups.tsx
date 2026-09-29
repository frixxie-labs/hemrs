import { useState } from "preact/hooks";
import type { Device } from "../lib/device.ts";

export default function DeviceGroups({ devices }: { devices: Device[] }) {
  const [query, setQuery] = useState("");
  const filtered = devices.filter((device) =>
    `${device.name} ${device.location} ${device.id}`.toLowerCase().includes(
      query.trim().toLowerCase(),
    )
  );
  const groups = new Map<string, Device[]>();
  for (const device of filtered) {
    const location = device.location || "Unassigned";
    const group = groups.get(location) ?? [];
    group.push(device);
    groups.set(location, group);
  }

  return (
    <section>
      <div class="monitor-section-heading monitor-device-heading">
        <h2>
          Devices <span>{filtered.length} / {devices.length}</span>
        </h2>
        <label class="monitor-search">
          <span class="sr-only">Search devices by name, location or ID</span>
          <input
            type="search"
            placeholder="Search name, location or ID"
            value={query}
            onInput={(event) => setQuery(event.currentTarget.value)}
          />
        </label>
      </div>
      <div class="monitor-locations">
        {Array.from(
          groups,
          ([location, entries]) => (
            <section class="monitor-location" key={location}>
              <h3>{location}</h3>
              <div class="monitor-table-wrap">
                <table class="monitor-table">
                  <thead>
                    <tr>
                      <th scope="col">ID</th>
                      <th scope="col">Device</th>
                    </tr>
                  </thead>
                  <tbody>
                    {entries.map((device) => (
                      <tr key={device.id}>
                        <td class="monitor-id">#{device.id}</td>
                        <td>
                          <a href={`/devices/${device.id}`}>{device.name} ↗</a>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          ),
        )}
      </div>
      <p class="monitor-search-status" role="status">
        {filtered.length
          ? `${filtered.length} devices in ${groups.size} locations`
          : devices.length
          ? "No devices match your search."
          : "No devices registered."}
      </p>
    </section>
  );
}
