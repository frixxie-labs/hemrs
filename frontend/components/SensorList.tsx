import type { Sensor } from "../lib/sensor.ts";

interface SensorListProps {
  sensors: Sensor[];
  device_id?: number;
}

export default function SensorList({ sensors, device_id }: SensorListProps) {
  return (
    <section>
      <div class="monitor-section-heading">
        <h2>Sensor types</h2>
        <span>{sensors.length} types</span>
      </div>
      <div class="monitor-table-wrap">
        <table class="monitor-table">
          <thead>
            <tr>
              <th scope="col">ID</th>
              <th scope="col">Sensor</th>
              <th scope="col">Unit</th>
            </tr>
          </thead>
          <tbody>
            {sensors.map((sensor) => (
              <tr key={sensor.id}>
                <td class="monitor-id">#{sensor.id}</td>
                <td>
                  {device_id === undefined
                    ? sensor.name
                    : (
                      <a href={`/devices/${device_id}/sensors/${sensor.id}`}>
                        {sensor.name} ↗
                      </a>
                    )}
                </td>
                <td class="monitor-value">{sensor.unit}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {!sensors.length && <p class="monitor-empty">No sensors registered.</p>}
    </section>
  );
}
