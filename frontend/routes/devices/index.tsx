import { page } from "fresh";
import { define } from "../../utils.ts";
import Button from "../../components/Button.tsx";
import { getDevices } from "../../lib/device.ts";
import DeviceGroups from "../../islands/DeviceGroups.tsx";

export const handler = define.handlers({
  async GET(_ctx) {
    const devices = await getDevices();
    return page({ devices });
  },
});

export default define.page<typeof handler>(({ data }) => {
  return (
    <div class="space-y-4">
      <div class="monitor-heading">
        <h1>Devices</h1>
        <a href="/devices/new">
          <Button type="button">New Device</Button>
        </a>
      </div>
      <DeviceGroups devices={data.devices} />
    </div>
  );
});
