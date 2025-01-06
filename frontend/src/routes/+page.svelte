<script lang="ts">
    import { Card } from "flowbite-svelte";

    let { data } = $props();
</script>

<Card style="auto center">
    {#await data.devices}
        <p>Fetching devices</p>
    {:then devices}
        <p>Registered devices: {devices.length}</p>
    {/await}
    {#await data.sensors}
        <p>Fetching sensors</p>
    {:then sensors}
        <p>Registered sensors: {sensors.length}</p>
    {/await}
    {#await data.measurements}
        <p>Fetching measurements</p>
    {:then measurements}
        <p>Registered measurements: {measurements}</p>
    {/await}
</Card>
