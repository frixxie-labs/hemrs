<script lang="ts">
    import {
        Spinner,
        Table,
        TableBody,
        TableBodyCell,
        TableBodyRow,
        TableHead,
        TableHeadCell,
    } from "flowbite-svelte";

    let { data } = $props();
</script>

{#await data.measurements}
    <div class="flex items-center justify-center">
        <p>Fetching measurements</p>
        <Spinner />
    </div>
{:then measurements}
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
            {#each measurements as measurement}
                <TableBodyRow>
                    <TableBodyCell>{measurement.timestamp}</TableBodyCell>
                    <TableBodyCell>{measurement.value}</TableBodyCell>
                    <TableBodyCell>{measurement.unit}</TableBodyCell>
                    <TableBodyCell>{measurement.device_name}</TableBodyCell>
                    <TableBodyCell>{measurement.device_location}</TableBodyCell>
                    <TableBodyCell>{measurement.sensor_name}</TableBodyCell>
                </TableBodyRow>
            {/each}
        </TableBody>
    </Table>
{/await}
