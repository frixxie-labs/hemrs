<script lang="ts">
    import {
        NavBrand,
        NavHamburger,
        NavLi,
        NavUl,
        Navbar,
    } from "flowbite-svelte";
    import "../app.css";
    interface Props {
        children?: import("svelte").Snippet;
    }

    let { children }: Props = $props();
</script>

<header>
    <Navbar>
        <NavBrand href="/" class="hover">
            <span class="self-center whitespace-nowrap text-xl font-semibold"
                >Hemrs</span
            >
        </NavBrand>
        <NavHamburger />
        <NavUl>
            <NavLi href="/" style="hover">Home</NavLi>
            <NavLi href="/devices">Devices</NavLi>
            <NavLi href="/sensors">Sensors</NavLi>
            <NavLi href="/measurements">Measurements</NavLi>
        </NavUl>
    </Navbar>
</header>

{@render children?.()}
