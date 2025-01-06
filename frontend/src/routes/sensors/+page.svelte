<script lang="ts">
    import { goto } from "$app/navigation";
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
</script>

<Button on:click={(_e) => goto("/sensors/new")}>New</Button>
{#await data.sensors then sensors}
    <Table>
        <TableHead>
            <TableHeadCell>id</TableHeadCell>
            <TableHeadCell>name</TableHeadCell>
            <TableHeadCell>unit</TableHeadCell>
        </TableHead>
        <TableBody>
            {#each sensors as sensor}
                <TableBodyRow>
                    <TableBodyCell>{sensor.id}</TableBodyCell>
                    <TableBodyCell>{sensor.name}</TableBodyCell>
                    <TableBodyCell>{sensor.unit}</TableBodyCell>
                </TableBodyRow>
            {/each}
        </TableBody>
    </Table>
{/await}
