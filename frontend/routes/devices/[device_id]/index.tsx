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
      <div class="bg-dark-card border border-dark-border rounded-xl p-6">
        <h1 class="text-2xl font-bold text-text-primary">
          Device: {data.device.name}
        </h1>
        <p class="text-text-secondary mt-1">
          Location: {data.device.location}
        </p>
      </div>
      <SensorList
        device_id={data.device.id}
        sensors={data.sensors}
      />
    </div>
  );
});
