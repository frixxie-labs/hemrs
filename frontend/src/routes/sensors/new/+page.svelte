<script lang="ts">
    import { Button, Input } from "flowbite-svelte";
</script>

<form method="POST">
    <Input placeholder="Sensor Name" name="name" />
    <Input placeholder="Sensor Unit" name="unit" />
    <Button type="submit">Submit</Button>
</form>
