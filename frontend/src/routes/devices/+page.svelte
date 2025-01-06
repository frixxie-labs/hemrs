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

<Button on:click={(_e) => goto("/devices/new")}>New</Button>
{#await data.devices}
    <p>Fetching devices</p>
{:then devices}
    <Table>
        <TableHead>
            <TableHeadCell>id</TableHeadCell>
            <TableHeadCell>name</TableHeadCell>
            <TableHeadCell>location</TableHeadCell>
        </TableHead>
        <TableBody>
            {#each devices as device}
                <TableBodyRow>
                    <TableBodyCell>{device.id}</TableBodyCell>
                    <TableBodyCell>{device.name}</TableBodyCell>
                    <TableBodyCell>{device.location}</TableBodyCell>
                </TableBodyRow>
            {/each}
        </TableBody>
    </Table>
{/await}
