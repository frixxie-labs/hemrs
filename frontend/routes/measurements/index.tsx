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
    <div class="px-4 py-8 mx-auto">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
        <h1 class="text-2xl font-bold mb-4">Measurements</h1>
        <a href="/measurements/new">
          <Button type="button">New measurement</Button>
        </a>
        <MeasurementList measurements={data.measurements} />
      </div>
    </div>
  );
});
