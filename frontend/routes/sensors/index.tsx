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
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">Sensors</h1>
        <a href="/sensors/new">
          <Button type="button">New Sensor</Button>
        </a>
        <SensorList sensors={data.sensors} />
      </div>
    </div>
  );
});
