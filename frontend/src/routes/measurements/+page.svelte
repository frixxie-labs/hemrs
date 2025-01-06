<script lang="ts">
    import { goto } from "$app/navigation";
    import type { Measurement } from "$lib/measurement";
    import {
        Button,
        Table,
        TableBody,
        TableBodyCell,
        TableBodyRow,
        TableHead,
        TableHeadCell,
    } from "flowbite-svelte";

    let { data } = $props();
    let measurement: Promise<Measurement> = data.measurement;
</script>

<Button on:click={(_e) => goto("/measurements/new")}>New</Button>
{#await measurement}
    <p>Fetching measurements</p>
{:then measurement}
    <Table>
        <TableHead>
            <TableHeadCell>ts</TableHeadCell>
            <TableHeadCell>value</TableHeadCell>
            <TableHeadCell>unit</TableHeadCell>
            <TableHeadCell>device_name</TableHeadCell>
            <TableHeadCell>device_location</TableHeadCell>
            <TableHeadCell>sensor_name</TableHeadCell>
        </TableHead>
        <TableBody>
            <TableBodyRow>
                <TableBodyCell>{measurement.timestamp}</TableBodyCell>
                <TableBodyCell>{measurement.value}</TableBodyCell>
                <TableBodyCell>{measurement.unit}</TableBodyCell>
                <TableBodyCell>{measurement.device_name}</TableBodyCell>
                <TableBodyCell>{measurement.device_location}</TableBodyCell>
                <TableBodyCell>{measurement.sensor_name}</TableBodyCell>
            </TableBodyRow>
        </TableBody>
    </Table>
{/await}
