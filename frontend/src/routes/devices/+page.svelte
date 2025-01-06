<script lang="ts">
    import { goto } from "$app/navigation";
    import type { Device } from "$lib/device";
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

    let devices: Promise<Device[]> = data.devices;
</script>

<Button on:click={(_e) => goto("/devices/new")}>New</Button>
{#await devices}
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
