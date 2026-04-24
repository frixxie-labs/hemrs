import { page } from "fresh";
import { define } from "../../utils.ts";
import MeasurementList from "../../components/MeasurementsList.tsx";
import { getAllLatestMeasurements } from "../../lib/measurements.ts";
import Button from "../../components/Button.tsx";

export const handler = define.handlers({
  async GET(_ctx) {
    const measurements = await getAllLatestMeasurements();
    return page({ measurements });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text-primary">Measurements</h1>
        <a href="/measurements/new">
          <Button type="button">New measurement</Button>
        </a>
      </div>
      <MeasurementList measurements={data.measurements} />
    </div>
  );
});
