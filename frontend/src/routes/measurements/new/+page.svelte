<script lang="ts">
    import { Button, Input, Select } from "flowbite-svelte";
    let { data } = $props();

    let devices = data.devices.map((device) => {
        return { name: device.name, value: device.id };
    });
    let sensors = data.sensors.map((sensor) => {
        return { name: sensor.name, value: sensor.id };
    });
</script>

<form method="POST">
    <Select placeholder="Device" name="device" items={devices} />
    <Select placeholder="Sensor" name="sensor" items={sensors} />
    <Input placeholder="Measurement" name="measurement" />
    <Button type="submit">Submit</Button>
</form>
