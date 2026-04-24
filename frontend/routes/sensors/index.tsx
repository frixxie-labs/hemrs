import { page } from "fresh";
import { define } from "../../utils.ts";
import SensorList from "../../components/SensorList.tsx";
import { getSensors } from "../../lib/sensor.ts";
import Button from "../../components/Button.tsx";

export const handler = define.handlers({
  async GET(_ctx) {
    const sensors = await getSensors();
    return page({ sensors });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text-primary">Sensors</h1>
        <a href="/sensors/new">
          <Button type="button">New Sensor</Button>
        </a>
      </div>
      <SensorList sensors={data.sensors} />
    </div>
  );
});
