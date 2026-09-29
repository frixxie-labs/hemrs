import { HttpError, page } from "fresh";
import { define } from "../../../utils.ts";
import { getSensorsByDeviceId } from "../../../lib/sensor.ts";
import { getDeviceById } from "../../../lib/device.ts";
import SensorList from "../../../components/SensorList.tsx";

export const handler = define.handlers({
  async GET(ctx) {
    const device_id = parseInt(ctx.params.device_id);
    const [sensors, device] = await Promise.all([
      getSensorsByDeviceId(device_id),
      getDeviceById(device_id),
    ]);
    if (!device) {
      throw new HttpError(404, `Device with ID ${device_id} not found`);
    }
    return page({ sensors, device });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="space-y-4">
      <div class="monitor-heading">
        <div>
          <h1>{data.device.name}</h1>
          <p class="text-text-secondary text-sm mt-2">
            {data.device.location || "Unassigned"} · #{data.device.id}
          </p>
        </div>
        <a class="text-sm text-text-secondary" href="/devices">All devices ↗</a>
      </div>
      <SensorList
        device_id={data.device.id}
        sensors={data.sensors}
      />
    </div>
  );
});
