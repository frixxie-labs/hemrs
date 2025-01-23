<script lang="ts">
    import { goto } from "$app/navigation";
    import {
        Table,
        TableBody,
        TableBodyCell,
        TableBodyRow,
        TableHead,
        TableHeadCell,
    } from "flowbite-svelte";

    let { data } = $props();
</script>

{#await data.sensors then sensors}
    <Table hoverable={true}>
        <TableHead>
            <TableHeadCell>id</TableHeadCell>
            <TableHeadCell>name</TableHeadCell>
            <TableHeadCell>unit</TableHeadCell>
        </TableHead>
        <TableBody>
            {#each sensors as sensor}
                <TableBodyRow
                    on:click={(_: MouseEvent) =>
                        goto(
                            `/devices/${data.device_id}/sensors/${sensor.id}/measurements`,
                        )}
                >
                    <TableBodyCell>{sensor.id}</TableBodyCell>
                    <TableBodyCell>{sensor.name}</TableBodyCell>
                    <TableBodyCell>{sensor.unit}</TableBodyCell>
                </TableBodyRow>
            {/each}
        </TableBody>
    </Table>
{/await}
