import { Context } from "fresh";
import MeasuremetList from "../../components/MeasurementsList.tsx";
import PlotCard from "../../components/PlotCard.tsx";
import {
  getAllLatestMeasurements,
  Measurement,
} from "../../lib/measurements.ts";
import { getAllMeasurementsPlot } from "../../lib/plotter.ts";
import Button from "../../components/Button.tsx";

interface MeasurementsProps {
  measurements: Promise<Measurement[]>;
  plot: Promise<string | null>;
}

export const handler = {
  async GET(ctx: Context<MeasurementsProps>) {
    const measurements = getAllLatestMeasurements();
    const plot = getAllMeasurementsPlot();
    ctx.state.measurements = measurements;
    ctx.state.plot = plot;
    return await Measurements(ctx);
  },
};

export default async function Measurements(
  ctx: Context<MeasurementsProps>,
) {
  const measurements = await ctx.state.measurements;
  const plot = await ctx.state.plot;
  return (
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">Measurements</h1>
        <a href="/measurements/new">
          <Button type="button">New measurement</Button>
        </a>
        <PlotCard title="All Measurements Over Time" svg={plot} />
        <MeasuremetList
          measurements={measurements}
        />
      </div>
    </div>
  );
}
