import { Context } from "fresh";
import SensorList from "../../components/SensorList.tsx";
import { getSensors, Sensor } from "../../lib/sensor.ts";
import Button from "../../components/Button.tsx";

interface SensorsProps {
  sensors: Promise<Sensor[]>;
}

export const handler = {
  async GET(ctx: Context<SensorsProps>) {
    const sensors = getSensors();

    ctx.state.sensors = sensors;
    return await Sensors(ctx);
  },
};

export default async function Sensors(ctx: Context<SensorsProps>) {
  const sensors = await ctx.state.sensors;
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">Sensors</h1>
        <a href="/sensors/new">
          <Button type="button">New Sensor</Button>
        </a>
        <SensorList
          sensors={sensors}
        />
      </div>
    </div>
  );
}
