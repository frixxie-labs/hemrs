<script lang="ts">
    import { Button, Input } from "flowbite-svelte";
</script>

<form method="POST">
    <Input placeholder="Device name" name="name" />
    <Input placeholder="Device location" name="location" />
    <Button type="submit">Submit</Button>
</form>
