import { Context } from "fresh";
import MeasuremetList from "../../components/MeasurementsList.tsx";
import {
  getAllLatestMeasurements,
  Measurement,
} from "../../lib/measurements.ts";
import Button from "../../components/Button.tsx";

interface MeasurementsProps {
  measurements: Promise<Measurement[]>;
}

export const handler = {
  async GET(ctx: Context<MeasurementsProps>) {
    const measurements = getAllLatestMeasurements();
    ctx.state.measurements = measurements;
    return await Measurements(ctx);
  },
};

export default async function Measurements(
  ctx: Context<MeasurementsProps>,
) {
  const measurements = await ctx.state.measurements;
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">Measurements</h1>
        <a href="/measurements/new">
          <Button type="button">New measurement</Button>
        </a>
        <MeasuremetList
          measurements={measurements}
        />
      </div>
    </div>
  );
}
