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
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-3xl font-bold mb-4">
          Device: {data.device.name}
        </h1>
        <p class="text-lg mb-6">
          Device Location: {data.device.location}
        </p>
        <SensorList
          device_id={data.device.id}
          sensors={data.sensors}
        />
      </div>
    </div>
  );
});
