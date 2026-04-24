import { page } from "fresh";
import { define } from "../../utils.ts";
import Button from "../../components/Button.tsx";
import { getDevices } from "../../lib/device.ts";
import DeviceList from "../../components/DeviceList.tsx";

export const handler = define.handlers({
  async GET(_ctx) {
    const devices = await getDevices();
    return page({ devices });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text-primary">Devices</h1>
        <a href="/devices/new">
          <Button type="button">New Device</Button>
        </a>
      </div>
      <DeviceList devices={data.devices} />
    </div>
  );
});
