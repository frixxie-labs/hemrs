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
    <div class="px-2 sm:px-4 py-4 sm:py-8 mx-auto">
      <div class="max-w-screen-lg mx-auto flex flex-col items-center justify-center space-y-4">
        <h1 class="text-2xl sm:text-3xl font-bold mb-4 text-gray-800">
          Devices
        </h1>
        <a href="/devices/new" class="mb-4">
          <Button type="button">New Device</Button>
        </a>
        <DeviceList devices={data.devices} />
      </div>
    </div>
  );
});
