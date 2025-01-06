<script lang="ts">
    import type { Device } from "$lib/device";
    import type { Sensor } from "$lib/sensor";
    import { Card } from "flowbite-svelte";

    let { data } = $props();

    let devices: Promise<Device[]> = data.devices;
    let sensors: Promise<Sensor[]> = data.sensors;
    let measurements: Promise<Number> = data.measurements;
</script>

<Card style="auto center">
    {#await devices}
        <p>Fetching devices</p>
    {:then devices}
        <p>Registered devices: {devices.length}</p>
    {/await}
    {#await sensors}
        <p>Fetching sensors</p>
    {:then sensors}
        <p>Registered sensors: {sensors.length}</p>
    {/await}
    {#await measurements}
        <p>Fetching measurements</p>
    {:then measurements}
        <p>Registered measurements: {measurements}</p>
    {/await}
</Card>
